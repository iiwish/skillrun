use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct Schemas {
    pub input: Value,
    pub output: Value,
}

pub fn validate_value(schema: &Value, value: &Value) -> Result<(), String> {
    validate_at(schema, value, "$")
}

fn validate_at(schema: &Value, value: &Value, path: &str) -> Result<(), String> {
    if let Some(allowed) = schema.get("enum").and_then(Value::as_array) {
        if !allowed.iter().any(|item| item == value) {
            return Err(format!("{path} must be one of {}", render_enum(allowed)));
        }
    }

    if let Some(type_value) = schema.get("type") {
        validate_type(type_value, value, path)?;
    }

    if schema_type_allows_object(schema) && value.is_object() {
        validate_object(schema, value, path)?;
    }

    if schema_type_allows_array(schema) && value.is_array() {
        validate_array(schema, value, path)?;
    }

    Ok(())
}

fn validate_object(schema: &Value, value: &Value, path: &str) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("{path} must be an object"))?;

    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        for item in required {
            let Some(key) = item.as_str() else {
                continue;
            };
            if !object.contains_key(key) {
                return Err(format!("{path} is missing required property {key}"));
            }
        }
    }

    let properties = schema.get("properties").and_then(Value::as_object);
    if schema.get("additionalProperties") == Some(&Value::Bool(false)) {
        for key in object.keys() {
            if !properties.is_some_and(|properties| properties.contains_key(key)) {
                return Err(format!("{path} contains unsupported property {key}"));
            }
        }
    }

    if let Some(properties) = properties {
        for (key, property_schema) in properties {
            if let Some(child) = object.get(key) {
                validate_at(property_schema, child, &format!("{path}.{key}"))?;
            }
        }
    }

    Ok(())
}

fn validate_array(schema: &Value, value: &Value, path: &str) -> Result<(), String> {
    let Some(items_schema) = schema.get("items") else {
        return Ok(());
    };
    let items = value
        .as_array()
        .ok_or_else(|| format!("{path} must be an array"))?;
    for (index, item) in items.iter().enumerate() {
        validate_at(items_schema, item, &format!("{path}[{index}]"))?;
    }
    Ok(())
}

fn validate_type(type_value: &Value, value: &Value, path: &str) -> Result<(), String> {
    let allowed = match type_value {
        Value::String(single) => vec![single.as_str()],
        Value::Array(items) => items.iter().filter_map(Value::as_str).collect(),
        _ => return Ok(()),
    };

    if allowed
        .iter()
        .any(|expected| value_matches_type(value, expected))
    {
        Ok(())
    } else {
        Err(format!("{path} must be {}", allowed.join(" or ")))
    }
}

fn value_matches_type(value: &Value, expected: &str) -> bool {
    match expected {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "boolean" => value.is_boolean(),
        "number" => value.is_number(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "null" => value.is_null(),
        _ => true,
    }
}

fn schema_type_allows_object(schema: &Value) -> bool {
    schema_type_allows(schema, "object")
}

fn schema_type_allows_array(schema: &Value) -> bool {
    schema_type_allows(schema, "array")
}

fn schema_type_allows(schema: &Value, expected: &str) -> bool {
    match schema.get("type") {
        Some(Value::String(value)) => value == expected,
        Some(Value::Array(values)) => values.iter().any(|value| value.as_str() == Some(expected)),
        _ => false,
    }
}

fn render_enum(values: &[Value]) -> String {
    values
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
