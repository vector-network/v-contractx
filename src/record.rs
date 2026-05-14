use serde::{Deserialize, Serialize};

use crate::certification::{AuthRatio, CertificationState};
use crate::errors::{KernelError, Result};
use crate::types::{EntityId, LogicalClock, OperationKind, ProjectionId, RecordId, RegionId, SignatureReference, SpaceId, Timestamp, Vector};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub record_id: RecordId,
    pub parent_record_ids: Vec<RecordId>,
    pub event_kind: OperationKind,
    pub space_id: SpaceId,
    pub region_id: RegionId,
    pub entity_id: EntityId,
    pub projection_id: Option<ProjectionId>,
    pub v_before: Vector,
    pub v_after: Vector,
    pub parameters: serde_json::Value,
    pub certification_state: CertificationState,
    pub auth_ratio: AuthRatio,
    pub timestamp: Timestamp,
    pub logical_clock: LogicalClock,
    pub proof: Option<String>,
    pub initiator_pk: String,
    pub signature: Option<SignatureReference>,
    pub hash: String,
    pub version: String,
    pub settlement_ref: Option<String>,
    pub is_failure: bool,
    pub failure_reason: Option<String>,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RecordMaterial {
    pub record_id: RecordId,
    pub parent_record_ids: Vec<RecordId>,
    pub event_kind: OperationKind,
    pub space_id: SpaceId,
    pub region_id: RegionId,
    pub entity_id: EntityId,
    pub projection_id: Option<ProjectionId>,
    pub v_before: Vector,
    pub v_after: Vector,
    pub parameters: serde_json::Value,
    pub certification_state: CertificationState,
    pub auth_ratio: AuthRatio,
    pub timestamp: Timestamp,
    pub logical_clock: LogicalClock,
    pub proof: Option<String>,
    pub initiator_pk: String,
    pub signature: Option<SignatureReference>,
    pub version: String,
    pub settlement_ref: Option<String>,
    pub is_failure: bool,
    pub failure_reason: Option<String>,
}

impl From<&Record> for RecordMaterial {
    fn from(value: &Record) -> Self {
        Self {
            record_id: value.record_id.clone(),
            parent_record_ids: value.parent_record_ids.clone(),
            event_kind: value.event_kind.clone(),
            space_id: value.space_id.clone(),
            region_id: value.region_id.clone(),
            entity_id: value.entity_id.clone(),
            projection_id: value.projection_id.clone(),
            v_before: value.v_before.clone(),
            v_after: value.v_after.clone(),
            parameters: value.parameters.clone(),
            certification_state: value.certification_state,
            auth_ratio: value.auth_ratio.clone(),
            timestamp: value.timestamp,
            logical_clock: value.logical_clock,
            proof: value.proof.clone(),
            initiator_pk: value.initiator_pk.clone(),
            signature: value.signature.clone(),
            version: value.version.clone(),
            settlement_ref: value.settlement_ref.clone(),
            is_failure: value.is_failure,
            failure_reason: value.failure_reason.clone(),
        }
    }
}

impl Record {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>> {
        let material = RecordMaterial::from(self);
        serde_json::to_vec(&material).map_err(|e| KernelError::RecordIntegrityViolation(e.to_string()))
    }

    pub fn recompute_hash(&self) -> Result<String> {
        let bytes = self.canonical_bytes()?;
        let hash = blake3::hash(&bytes);
        Ok(hash.to_hex().to_string())
    }

    pub fn verify_hash(&self) -> Result<()> {
        let expected = self.recompute_hash()?;
        if expected != self.hash {
            return Err(KernelError::RecordIntegrityViolation(format!(
                "hash mismatch: expected {expected}, found {}",
                self.hash
            )));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordEnvelope {
    pub record: Record,
    pub committed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureRecord {
    pub record_id: RecordId,
    pub event_kind: OperationKind,
    pub reason: String,
    pub space_id: SpaceId,
    pub region_id: RegionId,
    pub entity_id: EntityId,
    pub initiator_pk: String,
    pub timestamp: Timestamp,
    pub logical_clock: LogicalClock,
    pub hash: String,
}

pub fn make_record_id(prefix: &str, logical_clock: LogicalClock) -> RecordId {
    RecordId(format!("{}-{}", prefix, logical_clock))
}
