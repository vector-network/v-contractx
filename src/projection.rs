use serde::{Deserialize, Serialize};

use crate::certification::CertificationThreshold;
use crate::errors::{KernelError, Result};
use crate::types::{ContractId, EntityId, ProjectionId, RecordId, Vector, VectorState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectionLockKind {
    Stake,
    DirectedAllocation,
    RiskEscrow,
    TaskLock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementTrigger {
    Manual,
    HeightReached(u64),
    TimeReached(u64),
    ContractSignal,
    OracleSignal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrainPolicy {
    None,
    FixedBps(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementRule {
    pub allow_gain: bool,
    pub allow_loss: bool,
    pub allow_partial_release: bool,
    pub allow_staged_release: bool,
    pub max_phases: u32,
}

impl SettlementRule {
    pub fn immediate() -> Self {
        Self {
            allow_gain: true,
            allow_loss: true,
            allow_partial_release: true,
            allow_staged_release: false,
            max_phases: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionRequest {
    pub projection_id: ProjectionId,
    pub source_entity_id: EntityId,
    pub source_space_id: crate::types::SpaceId,
    pub contract_id: Option<ContractId>,
    pub source_state: VectorState,
    pub locked_amount: Vector,
    pub lock_kind: ProjectionLockKind,
    pub rule_environment: String,
    pub settlement_rule: SettlementRule,
    pub settlement_trigger: SettlementTrigger,
    pub drain_policy: DrainPolicy,
    pub certification_threshold: CertificationThreshold,
    pub created_by_pk: String,
    pub parent_record_ids: Vec<RecordId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectionStatus {
    Active,
    Settling,
    Settled,
    Staged { remaining_phases: u32 },
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionEnvelope {
    pub projection_id: ProjectionId,
    pub source_entity_id: EntityId,
    pub source_space_id: crate::types::SpaceId,
    pub contract_id: Option<ContractId>,
    pub original_vector: Vector,
    pub remaining_vector: Vector,
    pub locked_vector: Vector,
    pub escrow_vector: Vector,
    pub lock_kind: ProjectionLockKind,
    pub rule_environment: String,
    pub settlement_rule: SettlementRule,
    pub settlement_trigger: SettlementTrigger,
    pub drain_policy: DrainPolicy,
    pub certification_threshold: CertificationThreshold,
    pub status: ProjectionStatus,
    pub created_record_id: RecordId,
    pub settlement_record_id: Option<RecordId>,
    pub created_by_pk: String,
}

impl ProjectionEnvelope {
    pub fn is_active(&self) -> bool {
        matches!(self.status, ProjectionStatus::Active | ProjectionStatus::Staged { .. })
    }

    pub fn ensure_settleable(&self) -> Result<()> {
        match self.status {
            ProjectionStatus::Active | ProjectionStatus::Staged { .. } => Ok(()),
            ProjectionStatus::Settled => Err(KernelError::DoubleReconstruction(
                format!("projection {} already settled", self.projection_id.0),
            )),
            ProjectionStatus::Settling => Err(KernelError::InvalidState(
                format!("projection {} is already settling", self.projection_id.0),
            )),
            ProjectionStatus::Revoked => Err(KernelError::InvalidState(
                format!("projection {} is revoked", self.projection_id.0),
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionResult {
    pub envelope: ProjectionEnvelope,
    pub record_payload: serde_json::Value,
}

pub fn build_projection_envelope(request: &ProjectionRequest, projection_record_id: RecordId) -> Result<ProjectionEnvelope> {
    request.source_state.vector.ensure_same_dim(&request.locked_amount)?;
    let remaining = request.source_state.vector.checked_sub_vector(&request.locked_amount)?;
    if request.locked_amount.is_zero() {
        return Err(KernelError::ZeroVectorViolation(
            "projection locked amount cannot be zero".to_string(),
        ));
    }
    let escrow_vector = request.locked_amount.clone();

    Ok(ProjectionEnvelope {
        projection_id: request.projection_id.clone(),
        source_entity_id: request.source_entity_id.clone(),
        source_space_id: request.source_space_id.clone(),
        contract_id: request.contract_id.clone(),
        original_vector: request.source_state.vector.clone(),
        remaining_vector: remaining,
        locked_vector: request.locked_amount.clone(),
        escrow_vector,
        lock_kind: request.lock_kind,
        rule_environment: request.rule_environment.clone(),
        settlement_rule: request.settlement_rule.clone(),
        settlement_trigger: request.settlement_trigger,
        drain_policy: request.drain_policy,
        certification_threshold: request.certification_threshold.clone(),
        status: ProjectionStatus::Active,
        created_record_id: projection_record_id,
        settlement_record_id: None,
        created_by_pk: request.created_by_pk.clone(),
    })
}
