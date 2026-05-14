use serde::{Deserialize, Serialize};

use crate::errors::{KernelError, Result};
use crate::projection::{DrainPolicy, ProjectionEnvelope, ProjectionStatus, SettlementRule};
use crate::types::{RecordId, Vector};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementOutcome {
    NoChange,
    Gain { amount: Vector },
    Loss { amount: Vector },
    PartialRelease { amount: Vector },
    StagedRelease {
        released: Vector,
        remaining_locked: Vector,
        remaining_phases: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconstructionRequest {
    pub projection: ProjectionEnvelope,
    pub outcome: SettlementOutcome,
    pub reconstructed_record_id: RecordId,
    pub settlement_reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconstructionReceipt {
    pub projection_id: crate::types::ProjectionId,
    pub prior_record_id: RecordId,
    pub settlement_record_id: RecordId,
    pub resulting_vector: Vector,
    pub returned_to_live: Vector,
    pub locked_remaining: Vector,
    pub status: ProjectionStatus,
    pub settlement_reference: String,
    pub outcome_summary: String,
    pub drain_policy: DrainPolicy,
}

fn apply_vector_delta(base: &Vector, add: Option<&Vector>, subtract: Option<&Vector>) -> Result<Vector> {
    let mut result = base.clone();
    if let Some(add_vec) = add {
        result = result.checked_add_vector(add_vec)?;
    }
    if let Some(sub_vec) = subtract {
        result = result.checked_sub_vector(sub_vec)?;
    }
    Ok(result)
}

pub fn reconstruct_projection(request: ReconstructionRequest, settlement_rule: &SettlementRule) -> Result<(ProjectionEnvelope, ReconstructionReceipt)> {
    let mut projection = request.projection;
    projection.ensure_settleable()?;

    let (returned_to_live, locked_remaining, status, summary) = match request.outcome.clone() {
        SettlementOutcome::NoChange => {
            (projection.locked_vector.clone(), Vector::zero(projection.locked_vector.len()), ProjectionStatus::Settled, "no change".to_string())
        }
        SettlementOutcome::Gain { amount } => {
            if !settlement_rule.allow_gain {
                return Err(KernelError::InvalidState("gains are disallowed by settlement rule".to_string()));
            }
            let live = apply_vector_delta(&projection.locked_vector, Some(&amount), None)?;
            (live, Vector::zero(projection.locked_vector.len()), ProjectionStatus::Settled, "gain settled".to_string())
        }
        SettlementOutcome::Loss { amount } => {
            if !settlement_rule.allow_loss {
                return Err(KernelError::InvalidState("losses are disallowed by settlement rule".to_string()));
            }
            let live = apply_vector_delta(&projection.locked_vector, None, Some(&amount))?;
            (live, Vector::zero(projection.locked_vector.len()), ProjectionStatus::Settled, "loss settled".to_string())
        }
        SettlementOutcome::PartialRelease { amount } => {
            if !settlement_rule.allow_partial_release {
                return Err(KernelError::InvalidState("partial release is disallowed by settlement rule".to_string()));
            }
            let remaining_locked = projection.locked_vector.checked_sub_vector(&amount)?;
            (amount, remaining_locked, ProjectionStatus::Settled, "partial release settled".to_string())
        }
        SettlementOutcome::StagedRelease { released, remaining_locked, remaining_phases } => {
            if !settlement_rule.allow_staged_release {
                return Err(KernelError::InvalidState("staged settlement is disallowed by settlement rule".to_string()));
            }
            if remaining_phases == 0 {
                return Err(KernelError::InvalidParameter("remaining_phases must be non-zero for staged release".to_string()));
            }
            if remaining_phases > settlement_rule.max_phases {
                return Err(KernelError::InvalidParameter("remaining_phases exceeds settlement rule cap".to_string()));
            }
            let projected_status = if remaining_phases == 1 {
                ProjectionStatus::Settled
            } else {
                ProjectionStatus::Staged { remaining_phases: remaining_phases - 1 }
            };
            (released, remaining_locked, projected_status, "staged settlement applied".to_string())
        }
    };

    projection.escrow_vector = locked_remaining.clone();
    projection.locked_vector = locked_remaining.clone();
    projection.remaining_vector = projection.remaining_vector.checked_add_vector(&returned_to_live)?;
    projection.status = status.clone();
    projection.settlement_record_id = Some(request.reconstructed_record_id.clone());

    let receipt = ReconstructionReceipt {
        projection_id: projection.projection_id.clone(),
        prior_record_id: projection.created_record_id.clone(),
        settlement_record_id: request.reconstructed_record_id,
        resulting_vector: projection.remaining_vector.clone(),
        returned_to_live,
        locked_remaining,
        status,
        settlement_reference: request.settlement_reference,
        outcome_summary: summary,
        drain_policy: projection.drain_policy,
    };

    Ok((projection, receipt))
}
