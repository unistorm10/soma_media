//! Input validation against JSON schemas

use crate::error::MediaError;
use serde_json::Value;

pub type Result<T> = std::result::Result<T, MediaError>;

/// Validate input against a JSON schema
pub fn validate_input(input: &Value, schema: &Value) -> Result<()> {
    // Get required fields from schema
    if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
        for field_name in required {
            let field_str = field_name.as_str()
                .ok_or_else(|| MediaError::ValidationError("Invalid schema: required field not a string".to_string()))?;
            
            if !input.get(field_str).is_some() {
                return Err(MediaError::ValidationError(
                    format!("Missing required field: {}", field_str)
                ));
            }
        }
    }
    
    // Validate property types
    if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
        if let Some(input_obj) = input.as_object() {
            for (key, value) in input_obj {
                if let Some(prop_schema) = properties.get(key) {
                    validate_type(value, prop_schema)?;
                }
            }
        }
    }
    
    Ok(())
}

/// Validate that a value matches the expected type
fn validate_type(value: &Value, schema: &Value) -> Result<()> {
    if let Some(expected_type) = schema.get("type").and_then(|t| t.as_str()) {
        let valid = match expected_type {
            "string" => value.is_string(),
            "integer" | "number" => value.is_number(),
            "boolean" => value.is_boolean(),
            "array" => value.is_array(),
            "object" => value.is_object(),
            "null" => value.is_null(),
            _ => true, // Unknown types pass validation
        };
        
        if !valid {
            return Err(MediaError::ValidationError(
                format!("Type mismatch: expected {}, got {:?}", expected_type, value)
            ));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_validate_required_fields() {
        let schema = json!({
            "type": "object",
            "required": ["input_path", "output_path"]
        });
        
        let valid_input = json!({
            "input_path": "/tmp/input.mp3",
            "output_path": "/tmp/output.wav"
        });
        
        assert!(validate_input(&valid_input, &schema).is_ok());
        
        let invalid_input = json!({
            "input_path": "/tmp/input.mp3"
        });
        
        assert!(validate_input(&invalid_input, &schema).is_err());
    }
    
    #[test]
    fn test_validate_types() {
        let schema = json!({
            "type": "object",
            "properties": {
                "quality": { "type": "integer" },
                "force": { "type": "boolean" }
            }
        });
        
        let valid_input = json!({
            "quality": 92,
            "force": true
        });
        
        assert!(validate_input(&valid_input, &schema).is_ok());
        
        let invalid_input = json!({
            "quality": "ninety-two",
            "force": true
        });
        
        assert!(validate_input(&invalid_input, &schema).is_err());
    }
}
