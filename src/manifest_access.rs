use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;

pub struct ManifestView<'a> {
    value: &'a YamlValue,
}

impl<'a> ManifestView<'a> {
    pub fn new(value: &'a YamlValue) -> Self {
        Self { value }
    }

    pub fn skill_name(&self) -> Option<&'a str> {
        string_at(self.value, &["skill", "name"])
    }

    pub fn tool_name(&self) -> Option<&'a str> {
        string_at(self.value, &["tool", "name"])
    }

    pub fn tool_description(&self) -> Option<&'a str> {
        string_at(self.value, &["tool", "description"])
    }

    pub fn runtime_adapter(&self) -> Option<&'a str> {
        string_at(self.value, &["runtime", "adapter"])
    }

    pub fn runtime_entrypoint(&self) -> Option<&'a str> {
        string_at(self.value, &["runtime", "entrypoint"])
    }

    pub fn runtime_timeout(&self) -> Option<&'a str> {
        string_at(self.value, &["runtime", "timeout"])
    }

    pub fn runtime_command(&self) -> Result<Option<Vec<String>>, String> {
        string_array_at(self.value, &["runtime", "command"])
    }

    pub fn permissions_json(&self) -> Option<JsonValue> {
        json_value_at(self.value, &["permissions"])
    }

    pub fn input_schema_json(&self) -> Option<JsonValue> {
        json_value_at(self.value, &["schemas", "input"])
    }

    pub fn output_schema_json(&self) -> Option<JsonValue> {
        json_value_at(self.value, &["schemas", "output"])
    }

    pub fn source_path(&self, source: &str) -> Option<&'a str> {
        string_at(self.value, &["sources", source, "path"])
    }

    pub fn source_sha256(&self, source: &str) -> Option<&'a str> {
        string_at(self.value, &["sources", source, "sha256"])
    }

    pub fn skill_sha256(&self) -> Option<&'a str> {
        self.source_sha256("skill")
            .or_else(|| string_at(self.value, &["skill", "skill_hash"]))
    }

    pub fn action_sha256(&self) -> Option<&'a str> {
        self.source_sha256("action")
    }

    pub fn examples(&self) -> Option<&'a [YamlValue]> {
        value_at(self.value, &["examples"])
            .and_then(YamlValue::as_sequence)
            .map(Vec::as_slice)
    }
}

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
    use super::{json_value_at, string_array_at, string_at, ManifestView};
    use serde_json::json;
    use serde_yaml::Value as YamlValue;

    fn manifest() -> YamlValue {
        serde_yaml::from_str(
            r#"
runtime:
  adapter: command
  entrypoint: action.py
  timeout: 1s
  command:
    - python
    - action.py
sources:
  skill:
    path: SKILL.md
    sha256: skill-hash
  action:
    path: action.py
    sha256: action-hash
skill:
  name: command_hello
tool:
  name: command_hello
  description: Execute command hello.
schemas:
  input:
    type: object
permissions:
  env:
    read:
      - API_KEY
examples:
  - id: default
    input: examples/default.input.json
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

    #[test]
    fn exposes_domain_fields() {
        let manifest = manifest();
        let view = ManifestView::new(&manifest);

        assert_eq!(view.skill_name(), Some("command_hello"));
        assert_eq!(view.tool_name(), Some("command_hello"));
        assert_eq!(view.tool_description(), Some("Execute command hello."));
        assert_eq!(view.runtime_adapter(), Some("command"));
        assert_eq!(view.runtime_entrypoint(), Some("action.py"));
        assert_eq!(view.runtime_timeout(), Some("1s"));
        assert_eq!(view.source_path("skill"), Some("SKILL.md"));
        assert_eq!(view.skill_sha256(), Some("skill-hash"));
        assert_eq!(view.action_sha256(), Some("action-hash"));
        assert_eq!(view.examples().map(|items| items.len()), Some(1));
        assert_eq!(
            view.permissions_json(),
            Some(json!({ "env": { "read": ["API_KEY"] } }))
        );
    }
}
