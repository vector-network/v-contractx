use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmContractRequest {
    pub projection_id: String,
    pub locked_components: Vec<u128>,
    pub rule_environment: String,
    pub allow_gain: bool,
    pub allow_loss: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmContractResponse {
    pub accepted: bool,
    pub projected_components: Vec<u128>,
    pub note: String,
}

#[wasm_bindgen]
pub fn evaluate_contract(input_json: &str) -> String {
    let parsed: Result<WasmContractRequest, _> = serde_json::from_str(input_json);
    let response = match parsed {
        Ok(req) => WasmContractResponse {
            accepted: !req.locked_components.is_empty(),
            projected_components: req.locked_components,
            note: if req.allow_gain || req.allow_loss {
                format!("projection {} accepted in {}", req.projection_id, req.rule_environment)
            } else {
                format!("projection {} accepted with conservative settlement", req.projection_id)
            },
        },
        Err(err) => WasmContractResponse {
            accepted: false,
            projected_components: vec![],
            note: format!("invalid input: {}", err),
        },
    };
    serde_json::to_string(&response).unwrap_or_else(|_| "{\"accepted\":false}".to_string())
}
