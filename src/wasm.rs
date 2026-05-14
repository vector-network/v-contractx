//! Shared WASM-facing helpers.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmProjectionInput {
    pub projection_id: String,
    pub locked_components: Vec<u128>,
    pub rule_environment: String,
    pub settlement_rule: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmSettlementOutput {
    pub accepted: bool,
    pub projected_components: Vec<u128>,
    pub narrative: String,
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn settle_projection(input_json: &str) -> String {
    let input: WasmProjectionInput = serde_json::from_str(input_json)
        .unwrap_or(WasmProjectionInput {
            projection_id: "invalid".to_string(),
            locked_components: vec![],
            rule_environment: "invalid".to_string(),
            settlement_rule: "invalid".to_string(),
        });

    let output = WasmSettlementOutput {
        accepted: !input.locked_components.is_empty(),
        projected_components: input.locked_components,
        narrative: format!(
            "projection {} settled inside environment {}",
            input.projection_id, input.rule_environment
        ),
    };

    serde_json::to_string(&output).unwrap_or_else(|_| "{\"accepted\":false}".to_string())
}
