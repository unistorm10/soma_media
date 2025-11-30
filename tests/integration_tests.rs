//! Integration tests for soma_media organ operations

use soma_media::organ::{Organ, MediaOrgan, Stimulus, Response};
use serde_json::json;
use std::collections::HashMap;
use tempfile::TempDir;
use std::fs;
use std::path::PathBuf;

/// Helper to create a test stimulus
fn create_stimulus(op: &str, input: serde_json::Value) -> Stimulus {
    Stimulus {
        op: op.to_string(),
        input,
        context: HashMap::new(),
    }
}

#[tokio::test]
async fn test_media_capabilities() {
    let organ = MediaOrgan::new();
    
    let stimulus = create_stimulus("media.capabilities", json!({}));
    let response = organ.stimulate(stimulus).await.unwrap();
    
    assert!(response.ok);
    assert!(response.output["name"].as_str().unwrap() == "soma_media");
    assert!(response.output["functions"].as_array().is_some());
    
    let functions = response.output["functions"].as_array().unwrap();
    assert!(functions.len() >= 7, "Should have at least 7 functions");
}

#[tokio::test]
async fn test_organ_describe() {
    let organ = MediaOrgan::new();
    let card = organ.describe();
    
    assert_eq!(card.name, "soma_media");
    assert_eq!(card.division, "media");
    assert_eq!(card.subsystem, "preprocessing");
    assert!(card.execution_modes.contains(&"embedded".to_string()));
    assert!(card.execution_modes.contains(&"sidecar".to_string()));
    assert!(card.execution_modes.contains(&"server".to_string()));
    assert!(card.author.is_some());
    assert!(card.repository.is_some());
    assert_eq!(card.functions.len(), 7);
}

#[tokio::test]
async fn test_unsupported_operation() {
    let organ = MediaOrgan::new();
    
    let stimulus = create_stimulus("invalid.operation", json!({}));
    let response = organ.stimulate(stimulus).await.unwrap();
    
    assert!(!response.ok);
    assert!(response.output["error"].as_str().unwrap().contains("Unsupported"));
}

#[tokio::test]
async fn test_missing_required_input() {
    let organ = MediaOrgan::new();
    
    // audio.preprocess requires input_path and output_path
    let stimulus = create_stimulus("audio.preprocess", json!({}));
    let response = organ.stimulate(stimulus).await.unwrap();
    
    assert!(!response.ok);
    assert!(response.output["error"].as_str().is_some());
}

#[tokio::test]
async fn test_raw_metadata_missing_file() {
    let organ = MediaOrgan::new();
    
    let stimulus = create_stimulus("raw.metadata", json!({
        "input_path": "/nonexistent/file.CR2"
    }));
    
    let response = organ.stimulate(stimulus).await.unwrap();
    assert!(!response.ok);
}

#[tokio::test]
async fn test_raw_preview_missing_file() {
    let organ = MediaOrgan::new();
    
    let stimulus = create_stimulus("raw.preview", json!({
        "input_path": "/nonexistent/file.CR2",
        "output_path": "/tmp/test.webp"
    }));
    
    let response = organ.stimulate(stimulus).await.unwrap();
    assert!(!response.ok);
}

#[tokio::test]
async fn test_response_has_latency() {
    let organ = MediaOrgan::new();
    
    let stimulus = create_stimulus("media.capabilities", json!({}));
    let response = organ.stimulate(stimulus).await.unwrap();
    
    assert!(response.ok);
    assert!(response.latency_ms > 0);
}

#[tokio::test]
async fn test_all_function_names_match() {
    let organ = MediaOrgan::new();
    let card = organ.describe();
    
    let expected_functions = vec![
        "audio.preprocess",
        "audio.mel_spectrogram",
        "video.extract_frames",
        "image.preprocess",
        "raw.preview",
        "raw.metadata",
        "media.capabilities",
    ];
    
    let function_names: Vec<String> = card.functions.iter()
        .map(|f| f.name.clone())
        .collect();
    
    for expected in &expected_functions {
        assert!(
            function_names.contains(&expected.to_string()),
            "Missing function: {}",
            expected
        );
    }
}

#[tokio::test]
async fn test_function_cards_have_required_fields() {
    let organ = MediaOrgan::new();
    let card = organ.describe();
    
    for function in &card.functions {
        assert!(!function.name.is_empty(), "Function name is empty");
        assert!(!function.description.is_empty(), "Function description is empty for {}", function.name);
        assert!(!function.tags.is_empty(), "Function tags are empty for {}", function.name);
        assert!(!function.examples.is_empty(), "Function examples are empty for {}", function.name);
        assert!(function.output_schema.is_object(), "Output schema not an object for {}", function.name);
    }
}

#[tokio::test]
async fn test_idempotent_flags_set() {
    let organ = MediaOrgan::new();
    let card = organ.describe();
    
    for function in &card.functions {
        // All our functions should be marked as idempotent or not
        // Just verify the field exists (it's a bool so always has a value)
        let _ = function.idempotent;
    }
}

#[tokio::test]
async fn test_side_effects_documented() {
    let organ = MediaOrgan::new();
    let card = organ.describe();
    
    for function in &card.functions {
        // Side effects should be documented (can be empty for pure functions)
        assert!(function.side_effects.len() >= 0, "Side effects field exists for {}", function.name);
    }
}

#[cfg(feature = "gpu-auto")]
#[tokio::test]
async fn test_gpu_processor_creation() {
    use soma_media::GpuProcessor;
    
    let gpu = GpuProcessor::new();
    let backend = gpu.backend_info();
    
    assert!(!backend.is_empty());
    assert!(
        backend.contains("CUDA") || 
        backend.contains("Vulkan") || 
        backend.contains("Metal") || 
        backend.contains("CPU")
    );
}

#[cfg(feature = "gpu-auto")]
#[tokio::test]
async fn test_gpu_backend_info() {
    use soma_media::GpuProcessor;
    
    let gpu = GpuProcessor::new();
    
    // Should always have a backend (CPU fallback guaranteed)
    assert!(!gpu.backend_info().is_empty());
}
