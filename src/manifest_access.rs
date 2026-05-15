use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;

pub fn value_at<'a>(value: &'a YamlValue, path: &[&str]) -> Option<&'a YamlValue> {
    let mut current = value;
    for segment in path {
        let key = YamlValue::String((*segment).to_string());
        current = current.as_mapping()?.get(&key)?;
    }
    Some(current)
}

pub fn string_at<'a>(value: &'a YamlValue, path: &[&str]) -> Option<&'a str> {
    value_at(value, path)?.as_str()
}

pub fn string_array_at(value: &YamlValue, path: &[&str]) -> Result<Option<Vec<String>>, String> {
    let Some(value) = value_at(value, path) else {
        return Ok(None);
    };
    let sequence = value
        .as_sequence()
        .ok_or_else(|| format!("{} must be an array of strings", path.join(".")))?;
    let mut strings = Vec::with_capacity(sequence.len());
    for item in sequence {
        let Some(string) = item.as_str() else {
            return Err(format!("{} must contain only strings", path.join(".")));
        };
        strings.push(string.to_string());
    }
    Ok(Some(strings))
}

pub fn json_value_at(value: &YamlValue, path: &[&str]) -> Option<JsonValue> {
    let value = value_at(value, path)?;
    serde_json::to_value(value).ok()
}

#[cfg(test)]
mod tests {
    use super::{json_value_at, string_array_at, string_at};
    use serde_json::json;
    use serde_yaml::Value as YamlValue;

    fn manifest() -> YamlValue {
        serde_yaml::from_str(
            r#"
runtime:
  adapter: command
  command:
    - python
    - action.py
schemas:
  input:
    type: object
"#,
        )
        .expect("manifest should parse")
    }

    #[test]
    fn reads_nested_strings() {
        assert_eq!(
            string_at(&manifest(), &["runtime", "adapter"]),
            Some("command")
        );
    }

    #[test]
    fn reads_string_arrays() {
        assert_eq!(
            string_array_at(&manifest(), &["runtime", "command"]).unwrap(),
            Some(vec!["python".to_string(), "action.py".to_string()])
        );
    }

    #[test]
    fn converts_yaml_subtrees_to_json() {
        assert_eq!(
            json_value_at(&manifest(), &["schemas", "input"]),
            Some(json!({ "type": "object" }))
        );
    }
}
