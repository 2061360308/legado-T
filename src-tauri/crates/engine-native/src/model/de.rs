use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

pub fn default_true() -> bool {
    true
}

pub fn i32_or_string<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(match value {
        None | Some(Value::Null) => 0,
        Some(Value::Number(number)) => number.as_i64().unwrap_or_default() as i32,
        Some(Value::String(text)) => text.trim().parse().unwrap_or_default(),
        Some(Value::Bool(value)) => i32::from(value),
        _ => 0,
    })
}

pub fn bool_or_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(match value {
        None | Some(Value::Null) => true,
        Some(Value::Bool(value)) => value,
        Some(Value::Number(number)) => number.as_i64().unwrap_or_default() != 0,
        Some(Value::String(text)) => {
            let text = text.trim();
            !(text.is_empty()
                || text.eq_ignore_ascii_case("false")
                || text == "0"
                || text.eq_ignore_ascii_case("null"))
        }
        _ => true,
    })
}

pub fn opt_bool_or_string<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(match value {
        None | Some(Value::Null) => None,
        Some(Value::Bool(value)) => Some(value),
        Some(Value::Number(number)) => Some(number.as_i64().unwrap_or_default() != 0),
        Some(Value::String(text)) => {
            let text = text.trim();
            if text.is_empty() || text.eq_ignore_ascii_case("null") {
                None
            } else {
                Some(!(text.eq_ignore_ascii_case("false") || text == "0"))
            }
        }
        _ => None,
    })
}

pub fn opt_i64_or_string<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(match value {
        None | Some(Value::Null) => None,
        Some(Value::Number(number)) => number.as_i64(),
        Some(Value::String(text)) => {
            let text = text.trim();
            if text.is_empty() || text.eq_ignore_ascii_case("null") {
                None
            } else {
                text.parse().ok()
            }
        }
        _ => None,
    })
}

pub fn opt_string_or_json<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(match value {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) => Some(text),
        Some(other) => Some(other.to_string()),
    })
}

pub fn rule_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => {
            let text = text.trim();
            if text.is_empty() || text.eq_ignore_ascii_case("null") {
                Ok(None)
            } else {
                serde_json::from_str(text)
                    .map(Some)
                    .map_err(serde::de::Error::custom)
            }
        }
        Some(Value::Array(_)) => Ok(None),
        Some(other) => serde_json::from_value(other)
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}
