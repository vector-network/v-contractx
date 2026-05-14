use serde::{Deserialize, Serialize};

use crate::access::{is_authorized, AccessContext, AccessPolicy, Role};
use crate::certification::{CertificationReport, CertificationState};
use crate::errors::{KernelError, Result};
use crate::projection::SettlementRule;
use crate::reconstruction::SettlementOutcome;
use crate::types::{ContractId, EntityId, ProjectionId, SpaceId, Vector};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractMetadata {
    pub contract_id: ContractId,
    pub name: String,
    pub version: String,
    pub description: String,
    pub owner_pk: String,
    pub access_policy: AccessPolicy,
    pub settlement_rule: SettlementRule,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractContext {
    pub contract_id: ContractId,
    pub space_id: SpaceId,
    pub projection_id: ProjectionId,
    pub entity_id: EntityId,
    pub caller_pk: String,
    pub roles: Vec<Role>,
    pub certification: CertificationReport,
    pub projected_vector: Vector,
    pub locked_vector: Vector,
    pub live_remaining: Vector,
    pub environment: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractDecision {
    pub allow: bool,
    pub reason: Option<String>,
    pub modified_settlement_rule: Option<SettlementRule>,
    pub settlement_outcome: Option<SettlementOutcome>,
    pub emitted_metadata: serde_json::Value,
}

pub trait Contract: Send + Sync {
    fn metadata(&self) -> &ContractMetadata;
    fn evaluate(&self, ctx: &ContractContext) -> Result<ContractDecision>;
}

#[derive(Default)]
pub struct ContractRegistry {
    contracts: std::collections::BTreeMap<String, Box<dyn Contract>>,
}

impl ContractRegistry {
    pub fn register<C: Contract + 'static>(&mut self, contract: C) {
        self.contracts
            .insert(contract.metadata().contract_id.0.clone(), Box::new(contract));
    }

    pub fn get(&self, contract_id: &ContractId) -> Option<&dyn Contract> {
        self.contracts.get(&contract_id.0).map(|b| b.as_ref())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StakeDirective {
    pub staked_vector: Vector,
    pub directed_allocation: Vector,
    pub reward_vector: Vector,
    pub loss_vector: Vector,
    pub access_allowed: bool,
}

pub struct DefaultStakeContract {
    metadata: ContractMetadata,
}

impl DefaultStakeContract {
    pub fn new(metadata: ContractMetadata) -> Self {
        Self { metadata }
    }
}

impl Contract for DefaultStakeContract {
    fn metadata(&self) -> &ContractMetadata {
        &self.metadata
    }

    fn evaluate(&self, ctx: &ContractContext) -> Result<ContractDecision> {
        let access_ctx = AccessContext {
            caller_pk: ctx.caller_pk.clone(),
            roles: ctx.roles.clone(),
            owner_pk: self.metadata.owner_pk.clone(),
            certification_required: true,
        };

        let authorized = is_authorized(&access_ctx, &self.metadata.access_policy);
        if !authorized {
            return Ok(ContractDecision {
                allow: false,
                reason: Some("access policy denied the caller".to_string()),
                modified_settlement_rule: None,
                settlement_outcome: None,
                emitted_metadata: serde_json::json!({"allow": false}),
            });
        }

        if ctx.certification.state != CertificationState::Certified {
            return Ok(ContractDecision {
                allow: false,
                reason: Some("caller vector is not certified".to_string()),
                modified_settlement_rule: None,
                settlement_outcome: None,
                emitted_metadata: serde_json::json!({"allow": false, "reason": "uncertified"}),
            });
        }

        let reward = ctx.locked_vector.scale_bps(25_000);
        let allocation = ctx.locked_vector.scale_bps(10_000);
        let rule = self.metadata.settlement_rule.clone();
        let outcome = SettlementOutcome::StagedRelease {
            released: ctx.locked_vector.clone(),
            remaining_locked: Vector::zero(ctx.locked_vector.len()),
            remaining_phases: 1,
        };

        Ok(ContractDecision {
            allow: true,
            reason: Some("projection accepted by default stake contract".to_string()),
            modified_settlement_rule: Some(rule),
            settlement_outcome: Some(outcome),
            emitted_metadata: serde_json::json!({
                "staked": ctx.locked_vector,
                "directed_allocation": allocation,
                "reward": reward,
                "space": ctx.space_id.0,
                "projection": ctx.projection_id.0,
            }),
        })
    }
}

pub fn require_contract(contract: Option<&dyn Contract>, ctx: &ContractContext) -> Result<ContractDecision> {
    match contract {
        Some(contract) => contract.evaluate(ctx),
        None => Err(KernelError::ContractError("missing contract binding".to_string())),
    }
}
