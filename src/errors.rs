use serde_json::{json, Value};

pub const VALIDATION_ERROR: &str = "ValidationError";
pub const POLICY_VIOLATION: &str = "PolicyViolation";
pub const PROTOCOL_VIOLATION: &str = "ProtocolViolation";
pub const RUNTIME_ERROR: &str = "RuntimeError";

pub fn protocol_violation(message: impl Into<String>) -> Value {
    envelope(PROTOCOL_VIOLATION, message, false, None)
}

pub fn runtime_error(message: impl Into<String>) -> Value {
    envelope(RUNTIME_ERROR, message, false, None)
}

pub fn envelope(
    code: &str,
    message: impl Into<String>,
    recoverable: bool,
    llm_hint: Option<&str>,
) -> Value {
    let message = message.into();
    let mut error = json!({
        "code": code,
        "message": message,
        "recoverable": recoverable,
    });

    if let Some(llm_hint) = llm_hint {
        error["llm_hint"] = Value::String(llm_hint.to_string());
    }

    json!({
        "ok": false,
        "error": error,
        "display": {
            "markdown": message
        }
    })
}

pub fn validate_error_envelope(value: &Value) -> Result<(), String> {
    if value.get("ok").and_then(Value::as_bool) != Some(false) {
        return Err("error envelope must contain ok: false".to_string());
    }

    let error = value
        .get("error")
        .and_then(Value::as_object)
        .ok_or_else(|| "error envelope must contain an error object".to_string())?;
    let code = error
        .get("code")
        .and_then(Value::as_str)
        .ok_or_else(|| "error envelope must contain error.code".to_string())?;
    if ![
        VALIDATION_ERROR,
        POLICY_VIOLATION,
        PROTOCOL_VIOLATION,
        RUNTIME_ERROR,
    ]
    .contains(&code)
    {
        return Err(format!("unknown error code: {code}"));
    }
    if error.get("message").and_then(Value::as_str).is_none() {
        return Err("error envelope must contain error.message".to_string());
    }
    if error.get("recoverable").and_then(Value::as_bool).is_none() {
        return Err("error envelope must contain error.recoverable".to_string());
    }
    if value
        .get("display")
        .and_then(|display| display.get("markdown"))
        .and_then(Value::as_str)
        .is_none()
    {
        return Err("error envelope must contain display.markdown".to_string());
    }

    Ok(())
}
