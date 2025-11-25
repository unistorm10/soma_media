---
description: 'UMA + MCP interface requirements for Rust crates/modules in the platform'
---

# UMA + MCP Compliance Specification

**Purpose:** Defines interface contracts and capability exposure at the crate/module level  
**Scope:** How Rust crates expose functionality to orchestrators, agents, and each other  
**Related:** See SOMA_SPEC.instructions.md for build architecture requirements

**Application Context:**
- ✅ **DO apply** to Rust crates that expose platform capabilities as UMA modules
- ✅ **DO apply** when designing or implementing the crate's public API and UMA provider
- ✅ **DO apply** to crates that need to be discoverable or callable by other platform components
- ✅ **DOES apply** to: sigil, umbrafs, model_vault, soma_* crates, and any platform modules
- ❌ **DON'T apply** to target/ build directories (excluded by pattern)
- ❌ **DON'T apply** to tests, examples, or benchmarks (not part of crate interface)
- ❌ **DON'T apply** to simple utility crates that don't expose UMA interfaces

**Key Principle:** Each UMA-compliant organ (crate) implements the `Organ` trait at the crate level, exposing its capabilities through standardized Stimulus/Response patterns.

---

## 0) Core Concepts

**UMA (Universal Module Architecture)** is the interface contract layer that defines how SOMA organs expose capabilities across execution modes.

**MCP (Model Context Protocol) Compliance** means organs expose structured capability cards that enable the executive and agents to:
- Discover available functions dynamically
- Invoke specific operations by name
- Compose multiple organ functions together
- Route requests based on capabilities and SLOs

**Key Principle:** UMA defines the interface pattern (Stimulus/Response); MCP defines the capability exposure mechanism (cards).

---

## 1) UMA Interface Contract (Required)

Every SOMA organ MUST implement the Organ trait:

```rust
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// Universal stimulus structure - input to an organ
pub struct Stimulus {
    pub op: String,              // Function name (e.g., "vault.store", "search.vector")
    pub input: Value,            // JSON input matching function schema
    pub context: HashMap<String, String>, // Request metadata (trace_id, user_id, etc.)
}

/// Universal response structure - output from an organ
pub struct Response {
    pub ok: bool,                // Success indicator
    pub output: Value,           // JSON output matching function schema
    pub latency_ms: u64,         // Operation duration
    pub cost: Option<f64>,       // Optional cost tracking (tokens, compute, etc.)
}

/// Organ trait - every SOMA organ implements this
#[async_trait]
pub trait Organ: Send + Sync {
    /// Process a stimulus and produce a response
    async fn stimulate(&self, stimulus: Stimulus) -> Result<Response>;
    
    /// Return organ's capability card
    fn describe(&self) -> OrganCard;
}
```

### Implementation Pattern

```rust
pub struct MyOrgan {
    config: OrganConfig,
    // Internal implementation details
}

#[async_trait]
impl Organ for MyOrgan {
    async fn stimulate(&self, stimulus: Stimulus) -> Result<Response> {
        let start = Instant::now();
        
        // Route based on operation name
        let result = match request.op.as_str() {
            "function.name" => self.handle_function(request.input).await?,
            "submodule.operation" => {
                // Forward to sub-organ
                self.sub_organ.stimulate(Stimulus {
                    op: "operation".to_string(),
                    input: stimulus.input,
                    context: stimulus.context,
                }).await?
            }
            _ => {
                return Ok(Response {
                    ok: false,
                    output: json!({
                        "error": "UnsupportedOperation",
                        "op": stimulus.op
                    }),
                    latency_ms: start.elapsed().as_millis() as u64,
                    cost: None,
                })
            }
        };
        
        Ok(Response {
            ok: true,
            output: result,
            latency_ms: start.elapsed().as_millis() as u64,
            cost: None,
        })
    }
    
    fn describe(&self) -> OrganCard {
        // Return organ's capability card (see section 2)
        self.build_organ_card()
    }
}
```

---

## 2) MCP Capability Cards (Required)

Capability cards are structured metadata that describe what a module can do. They enable dynamic discovery and invocation.

### Organ Card

Top-level descriptor for the entire organ:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganCard {
    pub name: String,            // Organ identifier (e.g., "soma_vault")
    pub version: String,         // Semantic version
    pub description: String,     // Natural language description of organ purpose
    pub division: String,        // SOMA division (e.g., "storage", "ai", "semantic")
    pub subsystem: String,       // Subsystem name (e.g., "vault", "embeddings")
    pub tags: Vec<String>,       // Searchable tags for discovery (e.g., ["storage", "encryption", "blob"])
    pub execution_modes: Vec<String>, // Compiled modes: ["embedded", "sidecar", "server"]
    pub functions: Vec<FunctionCard>,  // Available operations
    pub sub_organs: Vec<SubOrganCard>, // Optional sub-components
    pub author: Option<String>,  // Organ maintainer/team
    pub repository: Option<String>, // Source repository URL
}
```

### Function Card

MCP-compliant descriptor for each operation:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCard {
    pub name: String,            // Fully qualified name (e.g., "vault.store")
    pub description: String,     // Natural language description of what the function does
    pub tags: Vec<String>,       // Searchable tags (e.g., ["storage", "write", "encryption"])
    pub examples: Vec<String>,   // Natural language usage examples
    pub input_schema: Value,     // JSON Schema for input validation
    pub output_schema: Value,    // JSON Schema for output validation
    pub slo: SloConfig,          // Service level objectives
    pub residency: ResidencyPolicy, // Data residency requirements
    pub idempotent: bool,        // Whether repeated calls with same input produce same result
    pub side_effects: Vec<String>, // List of side effects (e.g., ["writes to disk", "network call"])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SloConfig {
    pub p95_latency_ms: u64,     // 95th percentile latency target
    pub max_error_rate: f64,     // Maximum acceptable error rate (0.0-1.0)
    pub queue_cap: Option<usize>, // Optional queue depth limit
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResidencyPolicy {
    Local,                       // Must execute locally
    Regional,                    // Can execute in same region
    Global,                      // Can execute anywhere
}
```

### SubOrgan Card

For composite organs with multiple sub-components:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubOrganCard {
    pub name: String,            // Sub-organ identifier
    pub description: String,     // Natural language description
    pub tags: Vec<String>,       // Searchable tags for sub-organ
    #[serde(skip)]
    pub organ: Option<Box<dyn Organ>>, // Runtime organ instance
    pub functions: Vec<FunctionCard>, // Sub-organ's functions
}
```

### Example: Vault Organ Card

```rust
impl MyOrgan {
    fn build_organ_card(&self) -> OrganCard {
        OrganCard {
            name: "soma_vault".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Secure storage vault organ for encrypted blob management with metadata indexing and vector similarity search".to_string(),
            division: "storage".to_string(),
            subsystem: "vault".to_string(),
            tags: vec![
                "storage".to_string(),
                "vault".to_string(),
                "encryption".to_string(),
                "blob".to_string(),
                "metadata".to_string(),
                "vector-search".to_string(),
            ],
            execution_modes: self.get_available_modes(), // Implementation-specific
            author: Some("SOMA Storage Team".to_string()),
            repository: Some("https://github.com/org/soma_vault".to_string()),
            functions: vec![
                FunctionCard {
                    name: "vault.store".to_string(),
                    description: "Store a blob with optional metadata and automatic encryption. Returns confirmation and storage details.".to_string(),
                    tags: vec!["write".to_string(), "storage".to_string(), "encryption".to_string()],
                    examples: vec![
                        "Store user profile photo with metadata".to_string(),
                        "Archive encrypted document with tags".to_string(),
                        "Save binary artifact with version info".to_string(),
                    ],
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "key": {
                                "type": "string",
                                "description": "Unique identifier for the blob"
                            },
                            "data": {
                                "type": "string",
                                "description": "Base64-encoded blob data"
                            },
                            "metadata": {
                                "type": "object",
                                "description": "Optional key-value metadata"
                            }
                        },
                        "required": ["key", "data"]
                    }),
                    output_schema: json!({
                        "type": "object",
                        "properties": {
                            "stored": { "type": "boolean" },
                            "size_bytes": { "type": "integer" }
                        }
                    }),
                    slo: SloConfig {
                        p95_latency_ms: 50,
                        max_error_rate: 0.001,
                        queue_cap: Some(1000),
                    },
                    residency: ResidencyPolicy::Local,
                    idempotent: true,  // Same key+data = same result
                    side_effects: vec!["writes to disk".to_string(), "updates index".to_string()],
                },
                FunctionCard {
                    name: "vault.retrieve".to_string(),
                    description: "Retrieve a blob by its key with automatic decryption. Returns blob data and associated metadata.".to_string(),
                    tags: vec!["read".to_string(), "storage".to_string(), "decryption".to_string()],
                    examples: vec![
                        "Fetch user avatar by ID".to_string(),
                        "Load encrypted configuration file".to_string(),
                        "Retrieve artifact by version hash".to_string(),
                    ],
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "key": { "type": "string" }
                        },
                        "required": ["key"]
                    }),
                    output_schema: json!({
                        "type": "object",
                        "properties": {
                            "data": { "type": "string" },
                            "metadata": { "type": "object" }
                        }
                    }),
                    slo: SloConfig {
                        p95_latency_ms: 30,
                        max_error_rate: 0.001,
                        queue_cap: Some(2000),
                    },
                    residency: ResidencyPolicy::Local,
                    idempotent: true,  // Same key = same result
                    side_effects: vec!["reads from disk".to_string()],
                },
                // ... more functions
            ],
            sub_organs: vec![
                // Optional: if organ has sub-components
                SubOrganCard {
                    name: "similarity".to_string(),
                    description: "Vector similarity search sub-organ for semantic blob discovery".to_string(),
                    tags: vec!["search".to_string(), "vector".to_string(), "similarity".to_string()],
                    organ: None, // Set at runtime
                    functions: vec![
                        FunctionCard {
                            name: "similarity.search".to_string(),
                            description: "Find similar blobs using vector embeddings".to_string(),
                            tags: vec!["search".to_string(), "vector".to_string(), "semantic".to_string()],
                            examples: vec![
                                "Find images similar to a reference photo".to_string(),
                                "Discover related documents by content".to_string(),
                            ],
                            idempotent: true,
                            side_effects: vec![],
                            // ... schemas
                        },
                    ],
                },
            ],
        }
    }
}
```

---

## 3) Exporting Capability Cards

Organs MUST export capability cards as TOML/JSON files in the `cards/` directory for static discovery:

### Why Cards Are Critical for MCP

Capability cards enable:
- **Natural language search** - Tags and descriptions allow semantic discovery
- **Agent planning** - LLMs can read cards to understand available functions
- **Dynamic composition** - The executive chains functions based on descriptions
- **Schema validation** - Input/output schemas ensure type safety
- **Intent matching** - Examples help match user intent to functions
- **Side effect awareness** - Agents know what functions modify state

### Card Export Format

```toml
# cards/organ.toml
[organ]
name = "soma_vault"
version = "0.1.0"
description = "Secure storage vault organ for encrypted blob management with metadata indexing and vector similarity search"
division = "storage"
subsystem = "vault"
tags = ["storage", "vault", "encryption", "blob", "metadata", "vector-search"]
execution_modes = ["embedded", "sidecar", "server"]
author = "SOMA Storage Team"
repository = "https://github.com/org/soma_vault"

[[functions]]
name = "vault.store"
description = "Store a blob with optional metadata and automatic encryption. Returns confirmation and storage details."
tags = ["write", "storage", "encryption"]
examples = [
    "Store user profile photo with metadata",
    "Archive encrypted document with tags",
    "Save binary artifact with version info"
]
idempotent = true
side_effects = ["writes to disk", "updates index"]
residency = "Local"

[functions.input_schema]
type = "object"

[functions.input_schema.properties.key]
type = "string"
description = "Unique identifier for the blob"

[functions.input_schema.properties.data]
type = "string"
description = "Base64-encoded blob data"

[functions.input_schema.properties.metadata]
type = "object"
description = "Optional key-value metadata"

[functions.input_schema.required]
values = ["key", "data"]

[functions.output_schema]
type = "object"

[functions.output_schema.properties.stored]
type = "boolean"

[functions.output_schema.properties.size_bytes]
type = "integer"

[functions.slo]
p95_latency_ms = 50
max_error_rate = 0.001
queue_cap = 1000

[[functions]]
name = "vault.retrieve"
description = "Retrieve a blob by its key with automatic decryption. Returns blob data and associated metadata."
tags = ["read", "storage", "decryption"]
examples = [
    "Fetch user avatar by ID",
    "Load encrypted configuration file",
    "Retrieve artifact by version hash"
]
idempotent = true
side_effects = ["reads from disk"]
# ... rest of schema
```

### Best Practices for Tags

**Organ Tags (3-8 tags):**
- Primary domain (e.g., "storage", "ai", "networking")
- Key capabilities (e.g., "encryption", "vector-search", "caching")
- Use cases (e.g., "blob-storage", "embeddings", "routing")
- Technology markers (e.g., "rocksdb", "gpu", "distributed")

**Function Tags (2-5 tags):**
- Operation type (e.g., "read", "write", "delete", "search")
- Domain area (e.g., "storage", "compute", "index")
- Special properties (e.g., "async", "batch", "streaming")

**Tag Naming:**
- Use lowercase with hyphens (e.g., "vector-search", not "VectorSearch")
- Be specific but not verbose (e.g., "blob" not "binary-large-object")
- Include common synonyms (e.g., both "ai" and "machine-learning" if applicable)

### Best Practices for Descriptions

**Organ Description (1-2 sentences):**
- What the organ does (purpose)
- Key differentiators (e.g., "with automatic encryption")
- Primary use case

Example: "Secure storage vault organ for encrypted blob management with metadata indexing and vector similarity search"

**Function Description (1-2 sentences):**
- What the function does
- What it returns
- Key behavior notes

Example: "Store a blob with optional metadata and automatic encryption. Returns confirmation and storage details."

**Examples (2-4 natural language use cases):**
- Real-world scenarios
- Different parameter combinations
- Common user intents

Example:
- "Store user profile photo with metadata"
- "Archive encrypted document with tags"
- "Save binary artifact with version info"

---

## 4) Executive Orchestration Support

### Discovery Pattern

The executive discovers available capabilities using tags and descriptions:

```rust
pub struct Executive {
    organs: HashMap<String, Box<dyn Organ>>,
}

impl Executive {
    /// Discover all available functions across all organs
    pub fn list_functions(&self) -> Vec<(String, FunctionCard)> {
        self.organs
            .iter()
            .flat_map(|(organ_name, organ)| {
                let card = organ.describe();
                card.functions
                    .into_iter()
                    .map(move |func| (organ_name.clone(), func))
            })
            .collect()
    }
    
    /// Search functions by natural language query using tags and descriptions
    pub fn search_functions(&self, query: &str) -> Vec<(String, FunctionCard)> {
        let query_lower = query.to_lowercase();
        
        self.list_functions()
            .into_iter()
            .filter(|(_, func)| {
                // Match against description
                func.description.to_lowercase().contains(&query_lower) ||
                // Match against tags
                func.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower)) ||
                // Match against examples
                func.examples.iter().any(|ex| ex.to_lowercase().contains(&query_lower)) ||
                // Match against function name
                func.name.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
    
    /// Find functions by tag
    pub fn functions_with_tag(&self, tag: &str) -> Vec<(String, FunctionCard)> {
        self.list_functions()
            .into_iter()
            .filter(|(_, func)| func.tags.contains(&tag.to_string()))
            .collect()
    }
    
    /// Get specific organ's capabilities
    pub fn get_organ_card(&self, organ: &str) -> Option<OrganCard> {
        self.organs
            .get(organ)
            .map(|organ| organ.describe())
    }
}
```

### Invocation Pattern

The executive invokes specific functions by sending stimuli:

```rust
impl Executive {
    /// Send a stimulus to invoke a specific function on an organ
    pub async fn stimulate_organ(
        &self,
        organ_name: &str,
        function: &str,
        input: Value,
        context: HashMap<String, String>,
    ) -> Result<Response> {
        let organ = self.organs
            .get(organ_name)
            .ok_or_else(|| Error::OrganNotFound(organ_name.to_string()))?;
        
        organ.stimulate(Stimulus {
            op: function.to_string(),
            input,
            context,
        }).await
    }
    
    /// Validate input against function schema before sending stimulus
    pub fn validate_stimulus(
        &self,
        organ_name: &str,
        function: &str,
        input: &Value,
    ) -> Result<()> {
        let card = self.get_organ_card(organ_name)
            .ok_or_else(|| Error::OrganNotFound(organ_name.to_string()))?;
        
        let func_card = card.functions
            .iter()
            .find(|f| f.name == function)
            .ok_or_else(|| Error::FunctionNotFound(function.to_string()))?;
        
        // Validate input against JSON schema
        validate_json_schema(input, &func_card.input_schema)?;
        
        Ok(())
    }
}
```

### Composition Pattern

Chain multiple organ functions together in a pathway:

```rust
impl Executive {
    /// Execute a pathway of stimuli across multiple organs
    pub async fn pathway(
        &self,
        steps: Vec<PathwayStep>,
        initial_input: Value,
    ) -> Result<Response> {
        let mut current_output = initial_input;
        
        for step in steps {
            let response = self.stimulate_organ(
                &step.organ,
                &step.function,
                current_output,
                step.context,
            ).await?;
            
            if !response.ok {
                return Ok(response); // Propagate error
            }
            
            current_output = response.output;
        }
        
        Ok(Response {
            ok: true,
            output: current_output,
            latency_ms: 0, // Aggregate separately
            cost: None,
        })
    }
}

pub struct PathwayStep {
    pub organ: String,
    pub function: String,
    pub context: HashMap<String, String>,
}
```

---

## 5) Testing Requirements

All Organ implementations MUST pass the following tests:

### Capability Discovery Tests

```rust
#[test]
fn test_organ_description_complete() {
    let organ = MyOrgan::new(config);
    let card = organ.describe();
    
    // Verify all expected functions are present
    assert!(card.functions.iter().any(|f| f.name == "expected.function"));
    
    // Verify schemas are valid JSON Schema
    for func in &card.functions {
        assert!(is_valid_json_schema(&func.input_schema));
        assert!(is_valid_json_schema(&func.output_schema));
    }
}
```

### Organ Contract Tests

```rust
#[tokio::test]
async fn test_stimulus_response_contract() {
    let organ = MyOrgan::new(config);
    
    // Test successful operation
    let response = organ.stimulate(Stimulus {
        op: "function.name".to_string(),
        input: json!({"param": "value"}),
        context: HashMap::new(),
    }).await.unwrap();
    
    assert!(response.ok);
    assert!(response.latency_ms > 0);
    
    // Test unsupported operation
    let response = organ.stimulate(Stimulus {
        op: "nonexistent.function".to_string(),
        input: json!({}),
        context: HashMap::new(),
    }).await.unwrap();
    
    assert!(!response.ok);
    assert!(response.output["error"].as_str().is_some());
}
```

### Function Schema Validation Tests

```rust
#[test]
fn test_function_schemas() {
    let organ = MyOrgan::new(config);
    let card = organ.describe();
    
    for func in &card.functions {
        // Valid input should pass
        let valid_input = generate_valid_input(&func.input_schema);
        assert!(validate_json_schema(&valid_input, &func.input_schema).is_ok());
        
        // Invalid input should fail
        let invalid_input = json!({"wrong": "schema"});
        assert!(validate_json_schema(&invalid_input, &func.input_schema).is_err());
    }
}
```

### SubOrgan Routing Tests

```rust
#[tokio::test]
async fn test_sub_organ_routing() {
    let organ = MyOrgan::new(config);
    
    // Call sub-organ function via main organ
    let response = organ.stimulate(Stimulus {
        op: "sub_organ.operation".to_string(),
        input: json!({"param": "value"}),
        context: HashMap::new(),
    }).await.unwrap();
    
    assert!(response.ok);
}
```

### Feature Combination Tests

```bash
# Test that Organ interface works across all execution modes
# (See SOMA_SPEC.instructions.md for feature definition details)
cargo test --all-features
cargo test --no-default-features --features embedded
# Additional feature tests in SOMA_SPEC.instructions.md
```

---

## 7) Implementation Checklist

For each SOMA module:

- [ ] Implement `UmaProvider` trait with `ask()` method
- [ ] Implement `capabilities()` method returning complete `ModuleCard`
- [ ] Add module-level description (1-2 sentences, clear purpose)
- [ ] Add 3-8 relevant tags for module discoverability
- [ ] Create `FunctionCard` for every public operation
- [ ] Add function-level descriptions (what it does, what it returns)
- [ ] Add 2-5 tags per function for searchability
- [ ] Add 2-4 natural language examples per function
- [ ] Mark `idempotent` flag correctly for each function
- [ ] List all `side_effects` explicitly
- [ ] Define JSON Schema for all input/output payloads
- [ ] Set appropriate SLO targets for each function
- [ ] If composite module: add `SubmoduleCard` entries with descriptions and tags
- [ ] Export capability cards as TOML/JSON in `cards/` directory
- [ ] Add tests validating Ask/Reply contract
- [ ] Add tests validating schema compliance
- [ ] Add tests for all function operations
- [ ] Test natural language search works with tags and descriptions
- [ ] Document function examples in README
- [ ] Verify fine-grained function invocation works
- [ ] Test submodule routing (if applicable)
- [ ] Validate capability discovery from exported cards

---

## 8) Key Principles

1. **UMA is the interface** - Ask/Reply is how modules communicate, regardless of implementation
2. **Cards are self-description** - Modules advertise their capabilities via structured metadata
3. **MCP is the discovery protocol** - Orchestrators use cards to find and invoke functions
4. **Implementation-agnostic** - UMA doesn't care about execution modes, features, or internal structure (see SOMA_SPEC for that)
5. **Fine-grained invocation** - Orchestrators can call individual functions, not just whole modules
6. **Schema-driven validation** - JSON Schema ensures type safety across module boundaries
7. **Composition over monoliths** - Submodules enable clean decomposition of complex functionality
8. **Natural language first** - Tags, descriptions, and examples enable semantic discovery by LLMs and agents
9. **Transparency** - Side effects and idempotency flags help agents reason about function behavior

### Why Tags & Descriptions Are Critical

**For LLM Agents:**
- Read descriptions to understand what functions do
- Match user intent to function examples
- Use tags to filter relevant capabilities
- Reason about side effects before execution

**For Orchestrators:**
- Build dynamic execution plans based on descriptions
- Search functions semantically, not just by exact name
- Compose pipelines using natural language goals
- Validate safety using idempotency and side effects

**For Developers:**
- Discover modules through tag-based search
- Understand API surface without reading code
- Find similar/related functions via tags
- Learn usage patterns from examples

---

## 9) Example: Complete Implementation

See `soma_vault` for reference implementation:
- `src/uma.rs` - UmaProvider trait implementation
- `src/lib.rs` - Module integration with feature gates
- `cards/module.toml` - Exported capability cards
- `tests/uma_contract.rs` - Contract validation tests
- `examples/mcp_usage.rs` - Orchestrator usage examples

---

## 10) Relationship to SOMA Spec

**SOMA_SPEC.instructions.md** defines:
- Single-crate architecture with Cargo features
- Execution mode compilation targets
- Directory structure and build requirements
- Configuration and deployment mechanics

**UMA_MCP_COMPLIANCE.instructions.md** (this file) defines:
- Interface contracts (Ask/Reply pattern)
- Capability exposure (cards and schemas)
- Orchestration support (discovery and invocation)
- Runtime behavior contracts

Both specifications work together:
- SOMA spec ensures modules are built correctly
- UMA/MCP spec ensures modules expose capabilities correctly
- Modules must comply with both to be fully SOMA-compliant

---

## Questions or Issues?

For questions about:
- **Build architecture**: See SOMA_SPEC.instructions.md
- **Interface contracts**: See this file (UMA_MCP_COMPLIANCE.instructions.md)
- **Reference implementation**: Check soma_vault module
- **Agent operations**: See .github/copilot-instructions.md
