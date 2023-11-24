pub mod input;

use std::fmt::Display;

use input::{BoolInput, Input, Select, UserInput};

use serde::{Deserialize, Serialize};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Value {
    Array(Vec<Value>),
    InputString(UserInput<String>),
    InputBool(BoolInput),
    InputInt(UserInput<i64>),
    InputFloat(UserInput<f64>),
    Object(Map<Value>),
    String(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    Null,
}

impl Default for Value {
    fn default() -> Self {
        Self::new()
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<BoolInput> for Value {
    fn from(value: BoolInput) -> Self {
        Self::InputBool(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<UserInput<i64>> for Value {
    fn from(value: UserInput<i64>) -> Self {
        Self::InputInt(value)
    }
}

impl From<Input<i64>> for Value {
    fn from(value: Input<i64>) -> Self {
        Self::InputInt(UserInput::Input(value))
    }
}

impl From<Select<i64>> for Value {
    fn from(value: Select<i64>) -> Self {
        Self::InputInt(UserInput::Select(value))
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<UserInput<f64>> for Value {
    fn from(value: UserInput<f64>) -> Self {
        Self::InputFloat(value)
    }
}

impl From<Input<f64>> for Value {
    fn from(value: Input<f64>) -> Self {
        Self::InputFloat(UserInput::Input(value))
    }
}

impl From<Select<f64>> for Value {
    fn from(value: Select<f64>) -> Self {
        Self::InputFloat(UserInput::Select(value))
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<UserInput<String>> for Value {
    fn from(value: UserInput<String>) -> Self {
        Self::InputString(value)
    }
}

impl From<Input<String>> for Value {
    fn from(value: Input<String>) -> Self {
        Self::InputString(UserInput::Input(value))
    }
}

impl From<Select<String>> for Value {
    fn from(value: Select<String>) -> Self {
        Self::InputString(UserInput::Select(value))
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl<const N: usize, S: Into<String>, V: Into<Value>> From<[(S, V); N]> for Value {
    fn from(value: [(S, V); N]) -> Self {
        Self::Object(Map::from(value.map(|(k, v)| (k.into(), v.into()))))
    }
}

impl<const N: usize, T: Into<Value>> From<[T; N]> for Value {
    fn from(value: [T; N]) -> Self {
        Self::Array(Vec::from(value.map(|v| v.into())))
    }
}

impl Value {
    /// Create a new empty object
    pub fn new() -> Self {
        Self::Object(Map::new())
    }

    /// Initialize the value
    ///
    /// If the value is an input value, try to get the value from user input and set it to the value.
    /// If the value is an array or an object, initialize all the values in it recursively.
    pub fn init(&mut self) -> Result<(), TryFromError> {
        match self {
            Self::InputString(v) => *self = Self::String(v.get()?),
            Self::InputBool(v) => *self = Self::Bool(v.get()?),
            Self::InputInt(v) => *self = Self::Int(v.get()?),
            Self::InputFloat(v) => *self = Self::Float(v.get()?),
            Self::Object(map) => {
                for value in map.values_mut() {
                    value.init()?;
                }
            }
            Self::Array(array) => {
                for value in array {
                    value.init()?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Get value of given key
    ///
    /// If the value is an object and the key exists, the value will be returned.
    /// If the key is not exist, `None` will be returned.
    /// Otherwise, the panic will be raised.
    pub fn get(&self, key: &str) -> Option<&Self> {
        if let Self::Object(map) = self {
            map.get(key)
        } else {
            panic!("value is not an object");
        }
    }

    /// Get value of given key or return default value
    ///
    /// Get value of key by calling `get`. If the key is not exist, the default value will be returned.
    /// Otherwise the value will be converted to the type of the default value.
    pub fn get_or<'a, T>(&'a self, key: &str, default: T) -> Result<T, T::Error>
    where
        T: TryFrom<&'a Self>,
    {
        match self.get(key) {
            Some(value) => value.try_into(),
            None => Ok(default),
        }
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<Self>) {
        if let Self::Object(map) = self {
            map.insert(key.into(), value.into());
        } else {
            panic!("value is not an object");
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_) | Self::InputBool(_))
    }

    pub fn as_bool(&self) -> TryFromResult<bool> {
        match self {
            Self::InputBool(v) => Ok(v.get()?),
            Self::Bool(v) => Ok(*v),
            _ => Err(TryFromError::TypeMismatch),
        }
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(_) | Self::InputInt(_))
    }

    pub fn as_int(&self) -> TryFromResult<i64> {
        match self {
            Self::InputInt(v) => Ok(v.get()?),
            Self::Int(v) => Ok(*v),
            _ => Err(TryFromError::TypeMismatch),
        }
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_) | Self::InputFloat(_))
    }

    pub fn as_float(&self) -> TryFromResult<f64> {
        match self {
            Self::InputFloat(v) => Ok(v.get()?),
            Self::Float(v) => Ok(*v),
            _ => Err(TryFromError::TypeMismatch),
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_) | Self::InputString(_))
    }

    pub fn as_string(&self) -> TryFromResult<String> {
        match self {
            Self::InputString(v) => Ok(v.get()?),
            Self::String(v) => Ok(v.clone()),
            _ => Err(TryFromError::TypeMismatch),
        }
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    pub fn merge(&self, other: &Self) -> Self {
        let mut ret = self.clone();
        ret.merge_mut(other);
        ret
    }

    pub fn merge_mut(&mut self, other: &Self) {
        match (self, other) {
            (Self::Object(self_map), Self::Object(other_map)) => {
                for (key, value) in other_map {
                    if let Some(self_value) = self_map.get_mut(key) {
                        self_value.merge_mut(value);
                    } else {
                        self_map.insert(key.clone(), value.clone());
                    }
                }
            }
            (s, o) => *s = o.clone(),
        }
    }
}

#[macro_export]
macro_rules! object {
    () => {
        Value::new()
    };
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut value = Value::new();
        $(value.insert($key, $value);)*
        value
    }};
}

impl TryFrom<&Value> for bool {
    type Error = TryFromError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        value.as_bool()
    }
}

impl TryFrom<&Value> for i64 {
    type Error = TryFromError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        value.as_int()
    }
}

impl TryFrom<&Value> for f64 {
    type Error = TryFromError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        value.as_float()
    }
}

impl TryFrom<&Value> for String {
    type Error = TryFromError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        value.as_string()
    }
}

type TryFromResult<T> = Result<T, TryFromError>;

#[derive(Debug)]
pub enum TryFromError {
    TypeMismatch,
    InputError(input::Error),
}

impl From<input::Error> for TryFromError {
    fn from(error: input::Error) -> Self {
        Self::InputError(error)
    }
}

impl Display for TryFromError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TryFromError::TypeMismatch => write!(f, "type mismatch"),
            TryFromError::InputError(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for TryFromError {}

pub type Map<T> = std::collections::BTreeMap<String, T>;

#[cfg(test)]
mod tests {
    use super::*;

    use super::input::Input;

    mod serde {
        use super::*;

        use serde_test::{assert_de_tokens, assert_tokens, Token};

        #[test]
        fn input() {
            let value = object!(
                "input_bool" => BoolInput {
                    default: Some(true),
                    description: None,
                },
                "input_int" => Input {
                    default: Some(1),
                    description: None,
                },
                "input_float" => Input {
                    default: Some(1.0),
                    description: None,
                },
                "input_string" => Input {
                    default: Some("string".to_string()),
                    description: None,
                },
                "input_no_default" => Input::<String> {
                    default: None,
                    description: None,
                },
                "select_int" => Select {
                    alternatives: vec![1, 2],
                    description: None,
                },
                "select_float" => Select {
                    alternatives: vec![1.0, 2.0],
                    description: None,
                },
                "select_string" => Select {
                    alternatives: vec!["string1".to_string(), "string2".to_string()],
                    description: None,
                },
            );

            assert_de_tokens(
                &value,
                &[
                    Token::Map { len: Some(8) },
                    Token::Str("input_bool"),
                    Token::Map { len: Some(1) },
                    Token::Str("default"),
                    Token::Bool(true),
                    Token::MapEnd,
                    Token::Str("input_int"),
                    Token::Map { len: Some(1) },
                    Token::Str("default"),
                    Token::I64(1),
                    Token::MapEnd,
                    Token::Str("input_float"),
                    Token::Map { len: Some(1) },
                    Token::Str("default"),
                    Token::F64(1.0),
                    Token::MapEnd,
                    Token::Str("input_string"),
                    Token::Map { len: Some(1) },
                    Token::Str("default"),
                    Token::Str("string"),
                    Token::MapEnd,
                    Token::Str("input_no_default"),
                    Token::Map { len: Some(0) },
                    Token::MapEnd,
                    Token::Str("select_int"),
                    Token::Map { len: Some(1) },
                    Token::Str("alternatives"),
                    Token::Seq { len: Some(2) },
                    Token::I64(1),
                    Token::I64(2),
                    Token::SeqEnd,
                    Token::MapEnd,
                    Token::Str("select_float"),
                    Token::Map { len: Some(1) },
                    Token::Str("alternatives"),
                    Token::Seq { len: Some(2) },
                    Token::F64(1.0),
                    Token::F64(2.0),
                    Token::SeqEnd,
                    Token::MapEnd,
                    Token::Str("select_string"),
                    Token::Map { len: Some(1) },
                    Token::Str("alternatives"),
                    Token::Seq { len: Some(2) },
                    Token::Str("string1"),
                    Token::Str("string2"),
                    Token::SeqEnd,
                    Token::MapEnd,
                    Token::MapEnd,
                ],
            )
        }

        #[test]
        fn value() {
            let value = object!(
                "array" => [1, 2],
                "bool" => true,
                "float" => 1.0,
                "int" => 1,
                "null" => Value::Null,
                "object" => [("key", "value")],
                "string" => "string",
            );

            assert_tokens(
                &value,
                &[
                    Token::Map { len: Some(7) },
                    Token::Str("array"),
                    Token::Seq { len: Some(2) },
                    Token::I64(1),
                    Token::I64(2),
                    Token::SeqEnd,
                    Token::Str("bool"),
                    Token::Bool(true),
                    Token::Str("float"),
                    Token::F64(1.0),
                    Token::Str("int"),
                    Token::I64(1),
                    Token::Str("null"),
                    Token::Unit,
                    Token::Str("object"),
                    Token::Map { len: Some(1) },
                    Token::Str("key"),
                    Token::Str("value"),
                    Token::MapEnd,
                    Token::Str("string"),
                    Token::Str("string"),
                    Token::MapEnd,
                ],
            )
        }

        #[test]
        fn array() {
            let value = Value::Array(vec![
                Value::Bool(true),
                Value::Int(1_i64),
                Value::Float(1.0),
                Value::String("string".to_string()),
                Value::from([1, 2]),
                Value::from([("key", "value")]),
            ]);

            assert_tokens(
                &value,
                &[
                    Token::Seq { len: Some(6) },
                    Token::Bool(true),
                    Token::I64(1),
                    Token::F64(1.0),
                    Token::Str("string"),
                    Token::Seq { len: Some(2) },
                    Token::I64(1),
                    Token::I64(2),
                    Token::SeqEnd,
                    Token::Map { len: Some(1) },
                    Token::Str("key"),
                    Token::Str("value"),
                    Token::MapEnd,
                    Token::SeqEnd,
                ],
            )
        }

        #[test]
        fn bool() {
            let boolean = Value::Bool(true);
            assert_tokens(&boolean, &[Token::Bool(true)]);
        }

        #[test]
        fn int() {
            let integer = Value::Int(1);
            assert_tokens(&integer, &[Token::I64(1)]);
        }

        #[test]
        fn float() {
            let float = Value::Float(1.0);
            assert_tokens(&float, &[Token::F64(1.0)]);
        }

        #[test]
        fn string() {
            let string = Value::String("string".to_string());
            assert_tokens(&string, &[Token::Str("string")]);
        }

        #[test]
        fn null() {
            let null = Value::Null;
            assert_tokens(&null, &[Token::Unit]);
        }
    }

    #[test]
    fn init() {
        let input_bool = BoolInput {
            default: Some(true),
            description: None,
        };
        let input_int = UserInput::Input(Input {
            default: Some(1),
            description: None,
        });
        let input_float = UserInput::Input(Input {
            default: Some(1.0),
            description: None,
        });
        let input_string = UserInput::Input(Input {
            default: Some("string".to_string()),
            description: None,
        });

        let mut value = object!(
            "null" => Value::Null,
            "bool" => input_bool.clone(),
            "int" => input_int.clone(),
            "float" => input_float.clone(),
            "string" => input_string.clone(),
            "array" => [input_int.clone()],
            "object" => [("int", input_int.clone())],
        );

        assert_eq!(value.get("null").unwrap(), &Value::Null);
        assert_eq!(value.get("bool").unwrap(), &Value::InputBool(input_bool));
        assert_eq!(
            value.get("int").unwrap(),
            &Value::InputInt(input_int.clone())
        );
        assert_eq!(value.get("float").unwrap(), &Value::InputFloat(input_float));
        assert_eq!(
            value.get("string").unwrap(),
            &Value::InputString(input_string)
        );
        assert_eq!(
            value.get("array").unwrap(),
            &Value::Array(vec![Value::InputInt(input_int.clone())])
        );
        assert_eq!(
            value.get("object").unwrap(),
            &Value::Object(Map::from([(
                "int".to_string(),
                Value::InputInt(input_int.clone())
            )]))
        );

        value.init().unwrap();

        assert_eq!(value.get("null").unwrap(), &Value::Null);
        assert_eq!(value.get("bool").unwrap(), &Value::Bool(true));
        assert_eq!(value.get("int").unwrap(), &Value::Int(1));
        assert_eq!(value.get("float").unwrap(), &Value::Float(1.0));
        assert_eq!(
            value.get("string").unwrap(),
            &Value::String("string".to_string())
        );
        assert_eq!(
            value.get("array").unwrap(),
            &Value::Array(vec![Value::Int(1)])
        );
        assert_eq!(
            value.get("object").unwrap(),
            &Value::Object(Map::from([("int".to_string(), Value::Int(1))]))
        );
    }

    #[test]
    fn get() {
        let value = Value::from([("int", 1)]);

        assert_eq!(value.get("int").unwrap(), &Value::Int(1.into()));
        assert!(value.get("float").is_none());

        assert_eq!(value.get_or("int", 2).unwrap(), 1);
        assert_eq!(value.get_or("float", 2.0).unwrap(), 2.0);
    }

    #[test]
    #[should_panic]
    fn get_panic() {
        let value = Value::Null;
        value.get("int");
    }

    #[test]
    fn insert() {
        let mut value = Value::Object(Map::new());
        assert_eq!(value.get_or("int", 2).unwrap(), 2);
        value.insert("int", 1);
        assert_eq!(value.get_or("int", 2).unwrap(), 1);
    }

    #[test]
    #[should_panic]
    fn insert_panic() {
        let mut value = Value::Null;
        value.insert("int", 1);
    }

    #[test]
    fn is_sth() {
        assert!(Value::Null.is_null());
        assert!(Value::from(true).is_bool());
        assert!(Value::from(1).is_int());
        assert!(Value::from(1.0).is_float());
        assert!(Value::from(String::from("string")).is_string());
        assert!(Value::from("string").is_string());
        assert!(Value::from([1, 2]).is_array());
        assert!(Value::from([("key", "value")]).is_object());
    }

    #[test]
    #[allow(clippy::bool_assert_comparison)]
    fn try_from() {
        // Bool
        let bool_value = Value::from(true);
        assert_eq!(bool::try_from(&bool_value).unwrap(), true);
        assert!(matches!(
            i64::try_from(&bool_value),
            Err(TryFromError::TypeMismatch)
        ));
        let bool_input_value = Value::InputBool(BoolInput {
            default: Some(true),
            description: None,
        });
        assert_eq!(bool::try_from(&bool_input_value).unwrap(), true);
        assert!(matches!(
            i64::try_from(&bool_input_value),
            Err(TryFromError::TypeMismatch)
        ));

        // Int
        let int_value = Value::from(1);
        assert_eq!(i64::try_from(&int_value).unwrap(), 1);
        assert!(matches!(
            f64::try_from(&int_value),
            Err(TryFromError::TypeMismatch)
        ));
        let int_input_value = Value::InputInt(UserInput::Input(Input {
            default: Some(1),
            description: None,
        }));
        assert_eq!(i64::try_from(&int_input_value).unwrap(), 1);
        assert!(matches!(
            f64::try_from(&int_input_value),
            Err(TryFromError::TypeMismatch)
        ));

        // Float
        let float_value = Value::from(1.0);
        assert_eq!(f64::try_from(&float_value).unwrap(), 1.0);
        assert!(matches!(
            String::try_from(&float_value),
            Err(TryFromError::TypeMismatch)
        ));
        let float_input_value = Value::InputFloat(UserInput::Input(Input {
            default: Some(1.0),
            description: None,
        }));
        assert_eq!(f64::try_from(&float_input_value).unwrap(), 1.0);
        assert!(matches!(
            String::try_from(&float_input_value),
            Err(TryFromError::TypeMismatch)
        ));

        // String
        let string_value = Value::from("string");
        assert_eq!(String::try_from(&string_value).unwrap(), "string");
        assert!(matches!(
            bool::try_from(&string_value),
            Err(TryFromError::TypeMismatch)
        ));
        let string_input_value = Value::InputString(UserInput::Input(Input {
            default: Some("string".to_string()),
            description: None,
        }));
        assert_eq!(String::try_from(&string_input_value).unwrap(), "string");
        assert!(matches!(
            bool::try_from(&string_input_value),
            Err(TryFromError::TypeMismatch)
        ));
    }

    #[test]
    fn merge() {
        let value = object!(
            "bool" => true,
            "int" => 1,
            "float" => 1.0,
            "string" => "string",
            "array" => [1, 2],
            "object" => [("key1", "value1"), ("key2", "value2")],
        );

        let value2 = object!(
            "bool" => false,
            "int" => 2,
            "array" => [3, 4],
            "object" => [("key2", "value2_2"), ("key3", "value3")],
        );

        assert_eq!(
            value.merge(&value2),
            object!(
                "bool" => false,
                "int" => 2,
                "float" => 1.0,
                "string" => "string",
                "array" => [3, 4], // array will be replaced instead of merged
                "object" => [("key1", "value1"), ("key2", "value2_2"), ("key3", "value3")],
            ),
        );
    }
}
