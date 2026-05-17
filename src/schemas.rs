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

pub fn validate_schemas(schemas: &Schemas) -> Result<(), String> {
    validate_schema_contract(&schemas.input)
        .map_err(|error| format!("input schema contract invalid: {error}"))?;
    validate_schema_contract(&schemas.output)
        .map_err(|error| format!("output schema contract invalid: {error}"))
}

pub fn validate_schema_contract(schema: &Value) -> Result<(), String> {
    validate_schema_contract_at(schema, "$")
}

fn validate_schema_contract_at(schema: &Value, path: &str) -> Result<(), String> {
    let object = schema
        .as_object()
        .ok_or_else(|| format!("{path} schema must be an object"))?;

    if let Some(type_value) = object.get("type") {
        allowed_schema_types(type_value, path)?;
    }

    if let Some(enum_value) = object.get("enum") {
        let values = enum_value
            .as_array()
            .ok_or_else(|| format!("{path}.enum must be an array"))?;
        if values.is_empty() {
            return Err(format!("{path}.enum must not be empty"));
        }
    }

    if let Some(required_value) = object.get("required") {
        let required = required_value
            .as_array()
            .ok_or_else(|| format!("{path}.required must be an array of strings"))?;
        for item in required {
            if !item.is_string() {
                return Err(format!("{path}.required must contain only strings"));
            }
        }
    }

    if let Some(properties_value) = object.get("properties") {
        let properties = properties_value
            .as_object()
            .ok_or_else(|| format!("{path}.properties must be an object"))?;
        for (key, property_schema) in properties {
            validate_schema_contract_at(property_schema, &format!("{path}.properties.{key}"))?;
        }
    }

    if let Some(items_schema) = object.get("items") {
        validate_schema_contract_at(items_schema, &format!("{path}.items"))?;
    }

    if let Some(min_length) = object.get("minLength") {
        if min_length.as_u64().is_none() {
            return Err(format!("{path}.minLength must be a non-negative integer"));
        }
    }

    if let Some(minimum) = object.get("minimum") {
        if minimum.as_f64().is_none() {
            return Err(format!("{path}.minimum must be a number"));
        }
    }

    if let Some(additional_properties) = object.get("additionalProperties") {
        if !additional_properties.is_boolean() {
            return Err(format!("{path}.additionalProperties must be a boolean"));
        }
    }

    Ok(())
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

    validate_string_constraints(schema, value, path)?;
    validate_number_constraints(schema, value, path)?;

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
    let allowed = allowed_schema_types(type_value, path)?;

    if allowed
        .iter()
        .any(|expected| value_matches_type(value, expected))
    {
        Ok(())
    } else {
        Err(format!("{path} must be {}", allowed.join(" or ")))
    }
}

fn allowed_schema_types<'a>(type_value: &'a Value, path: &str) -> Result<Vec<&'a str>, String> {
    let allowed = match type_value {
        Value::String(single) => vec![single.as_str()],
        Value::Array(items) => {
            if items.is_empty() {
                return Err(format!("{path} schema type array must not be empty"));
            }
            let mut allowed = Vec::with_capacity(items.len());
            for item in items {
                let Some(item) = item.as_str() else {
                    return Err(format!(
                        "{path} schema type array must contain only strings"
                    ));
                };
                allowed.push(item);
            }
            allowed
        }
        _ => {
            return Err(format!(
                "{path} schema type must be a string or string array"
            ))
        }
    };

    for expected in &allowed {
        if !is_supported_type(expected) {
            return Err(format!("{path} uses unsupported schema type {expected}"));
        }
    }

    Ok(allowed)
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
        _ => false,
    }
}

fn is_supported_type(expected: &str) -> bool {
    matches!(
        expected,
        "object" | "array" | "string" | "boolean" | "number" | "integer" | "null"
    )
}

fn validate_string_constraints(schema: &Value, value: &Value, path: &str) -> Result<(), String> {
    let Some(min_length) = schema.get("minLength").and_then(Value::as_u64) else {
        return Ok(());
    };
    let Some(text) = value.as_str() else {
        return Ok(());
    };
    if text.chars().count() < min_length as usize {
        return Err(format!(
            "{path} string length must be at least {min_length}"
        ));
    }
    Ok(())
}

fn validate_number_constraints(schema: &Value, value: &Value, path: &str) -> Result<(), String> {
    let Some(minimum) = schema.get("minimum").and_then(Value::as_f64) else {
        return Ok(());
    };
    let Some(number) = value.as_f64() else {
        return Ok(());
    };
    if number < minimum {
        return Err(format!("{path} number must be at least {minimum}"));
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::{validate_schema_contract, validate_schemas, validate_value, Schemas};
    use serde_json::json;

    #[test]
    fn validates_min_length() {
        let schema = json!({ "type": "string", "minLength": 2 });

        assert!(validate_value(&schema, &json!("ab")).is_ok());
        assert_eq!(
            validate_value(&schema, &json!("a")).unwrap_err(),
            "$ string length must be at least 2"
        );
    }

    #[test]
    fn validates_minimum_for_numbers_and_integers() {
        let schema = json!({ "type": "integer", "minimum": 2 });

        assert!(validate_value(&schema, &json!(2)).is_ok());
        assert_eq!(
            validate_value(&schema, &json!(1)).unwrap_err(),
            "$ number must be at least 2"
        );
    }

    #[test]
    fn rejects_unknown_schema_types() {
        let schema = json!({ "type": "date" });

        assert_eq!(
            validate_value(&schema, &json!("2026-05-15")).unwrap_err(),
            "$ uses unsupported schema type date"
        );
    }

    #[test]
    fn rejects_invalid_schema_type_shapes() {
        assert_eq!(
            validate_value(&json!({ "type": 42 }), &json!("value")).unwrap_err(),
            "$ schema type must be a string or string array"
        );
        assert_eq!(
            validate_value(&json!({ "type": [] }), &json!("value")).unwrap_err(),
            "$ schema type array must not be empty"
        );
        assert_eq!(
            validate_value(&json!({ "type": ["string", 42] }), &json!("value")).unwrap_err(),
            "$ schema type array must contain only strings"
        );
    }

    #[test]
    fn validates_schema_contract_without_values() {
        let schema = json!({
            "type": "object",
            "required": ["name"],
            "properties": {
                "name": { "type": "string", "minLength": 1 },
                "items": {
                    "type": "array",
                    "items": { "type": "integer", "minimum": 1 }
                }
            },
            "additionalProperties": false
        });

        assert!(validate_schema_contract(&schema).is_ok());
    }

    #[test]
    fn rejects_invalid_schema_contract_shapes() {
        assert_eq!(
            validate_schema_contract(&json!({ "type": 42 })).unwrap_err(),
            "$ schema type must be a string or string array"
        );
        assert_eq!(
            validate_schema_contract(&json!({ "type": "object", "required": ["name", 42] }))
                .unwrap_err(),
            "$.required must contain only strings"
        );
        assert_eq!(
            validate_schema_contract(&json!({ "type": "object", "properties": [] })).unwrap_err(),
            "$.properties must be an object"
        );
        assert_eq!(
            validate_schema_contract(&json!({ "type": "array", "items": [] })).unwrap_err(),
            "$.items schema must be an object"
        );
        assert_eq!(
            validate_schema_contract(&json!({ "type": "string", "minLength": -1 })).unwrap_err(),
            "$.minLength must be a non-negative integer"
        );
    }

    #[test]
    fn validates_schema_pair_contract() {
        let schemas = Schemas {
            input: json!({ "type": "object" }),
            output: json!({ "type": [] }),
        };

        assert_eq!(
            validate_schemas(&schemas).unwrap_err(),
            "output schema contract invalid: $ schema type array must not be empty"
        );
    }
}
