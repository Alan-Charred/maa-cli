// This file is used to download and extract prebuilt packages of maa-cli.

use crate::dirs::{Dirs, Ensure};

use super::{
    download::{download, Checker},
    extract::Archive,
};

use std::env::{consts::EXE_SUFFIX, current_exe};
use std::{env::var_os, path::Path};

use anyhow::{bail, Context, Ok, Result};
use semver::Version;
use serde::Deserialize;
use tokio::runtime::Runtime;

const MAA_CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn name() -> String {
    format!("maa{}", EXE_SUFFIX)
}

pub fn version() -> Result<Version> {
    Version::parse(MAA_CLI_VERSION).context("Failed to parse maa-cli version")
}

pub fn update(dirs: &Dirs) -> Result<()> {
    let version_json = get_metadata()?;
    let asset = version_json.get_asset()?;
    let cur_version = asset.version();

    let cache_dir = dirs.cache().ensure()?;

    let last_version = version()?;
    if *cur_version > last_version {
        println!(
            "Found newer {} version v{} (current: v{}), updating...",
            name(),
            cur_version,
            last_version
        );

        let bin_name = name();
        let bin_path = current_exe()?;

        asset.download(cache_dir)?.extract(|path| {
            if path.ends_with(&bin_name) {
                Some(bin_path.clone())
            } else {
                None
            }
        })?;
    } else {
        println!("Up to date: {} v{}.", name(), last_version);
    }

    Ok(())
}

fn get_metadata() -> Result<VersionJSON> {
    let metadata_url = if let Some(url) = var_os("MAA_CLI_API") {
        url.into_string().unwrap()
    } else {
        String::from("https://github.com/MaaAssistantArknights/maa-cli/raw/version/version.json")
    };
    let metadata: VersionJSON = reqwest::blocking::get(metadata_url)?.json()?;
    Ok(metadata)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
/// The version.json file from the server.
///
/// Example:
/// ```json
/// {
///    "maa-cli": {
///      "universal-apple-darwin": {
///        "version": "0.1.0",
///        "name": "maa_cli-v0.1.0-universal-apple-darwin.tar.gz",
///        "size": 123456,
///        "sha256sum": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
///      },
///      "x86_64-unknown-linux-gnu": {
///        ...
///      }
///   },
///   "maa-run": {
///     "universal-apple-darwin": {
///       ...
///     },
///     ...
///   }
/// }
/// ```
struct VersionJSON {
    pub maa_cli: Targets,
}

impl VersionJSON {
    pub fn get_asset(&self) -> Result<&Asset> {
        let targets = &self.maa_cli;

        if cfg!(target_os = "macos") {
            Ok(&targets.universal_macos)
        } else if cfg!(target_os = "linux")
            && cfg!(target_arch = "x86_64")
            && cfg!(target_env = "gnu")
        {
            Ok(&targets.x64_linux_gnu)
        } else {
            bail!("Unsupported platform")
        }
    }
}

#[derive(Deserialize)]
pub struct Targets {
    #[serde(rename = "universal-apple-darwin")]
    universal_macos: Asset,
    #[serde(rename = "x86_64-unknown-linux-gnu")]
    x64_linux_gnu: Asset,
}

#[derive(Deserialize)]
pub struct Asset {
    version: Version,
    tag: String,
    name: String,
    size: u64,
    sha256sum: String,
}

impl Asset {
    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn download(&self, dir: &Path) -> Result<Archive> {
        let path = dir.join(&self.name);
        let size = self.size;

        if path.exists() {
            let file_size = path.metadata()?.len();
            if file_size == size {
                println!("Found existing file: {}", path.display());
                return Ok(Archive::try_from(path)?);
            }
        }

        let url = format_url(&self.tag, &self.name);

        let client = reqwest::Client::new();
        Runtime::new()
            .context("Failed to create tokio runtime")?
            .block_on(download(
                &client,
                &url,
                &path,
                size,
                Some(Checker::Sha256(&self.sha256sum)),
            ))
            .context("Failed to download maa-cli")?;

        Ok(Archive::try_from(path)?)
    }
}

fn format_url(tag: &str, name: &str) -> String {
    if let Some(url) = var_os("MAA_CLI_DOWNLOAD") {
        format!("{}/{}/{}", url.into_string().unwrap(), tag, name)
    } else {
        format!(
            "https://github.com/MaaAssistantArknights/maa-cli/releases/download/{}/{}",
            tag, name
        )
    }
}