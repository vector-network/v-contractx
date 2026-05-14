use serde::{Deserialize, Serialize};

use crate::certification::{certify, AuthInputs, AuthRatio, CertificationContext, CertificationState};
use crate::contract::{ContractContext, ContractDecision, ContractRegistry};
use crate::errors::{KernelError, Result};
use crate::projection::{build_projection_envelope, ProjectionEnvelope, ProjectionRequest, ProjectionStatus};
use crate::record::{make_record_id, Record};
use crate::reconstruction::{reconstruct_projection, ReconstructionReceipt, ReconstructionRequest};
use crate::types::{EntityId, LogicalClock, OperationKind, RecordId, RegionId, SpaceId, Timestamp, Vector, VectorState, Wallet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelContext {
    pub protocol_version: String,
    pub space_id: SpaceId,
    pub region_id: RegionId,
    pub timestamp: Timestamp,
    pub logical_clock: LogicalClock,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveEntityState {
    pub entity_id: EntityId,
    pub wallet: Wallet,
    pub vector_state: VectorState,
    pub certification: crate::certification::CertificationReport,
    pub active_projection: Option<ProjectionEnvelope>,
}

#[derive(Debug, Default)]
pub struct EventStore {
    records: Vec<Record>,
}

impl EventStore {
    pub fn append(&mut self, record: Record) {
        self.records.push(record);
    }

    pub fn all(&self) -> &[Record] {
        &self.records
    }
}

#[derive(Debug, Default)]
pub struct DerivedStateCache {
    pub entity_states: std::collections::BTreeMap<String, LiveEntityState>,
    pub projection_states: std::collections::BTreeMap<String, ProjectionEnvelope>,
}

pub struct Kernel {
    pub context: KernelContext,
    pub event_store: EventStore,
    pub derived: DerivedStateCache,
    pub contracts: ContractRegistry,
}

impl Kernel {
    pub fn new(context: KernelContext) -> Self {
        Self {
            context,
            event_store: EventStore::default(),
            derived: DerivedStateCache::default(),
            contracts: ContractRegistry::default(),
        }
    }

    fn next_record_id(&self, prefix: &str) -> RecordId {
        make_record_id(prefix, self.context.logical_clock)
    }

    fn append_record(&mut self, mut record: Record) -> Result<Record> {
        record.hash = record.recompute_hash()?;
        record.verify_hash()?;
        self.event_store.append(record.clone());
        Ok(record)
    }

    pub fn create_entity(
        &mut self,
        wallet: Wallet,
        vector_state: VectorState,
        auth_inputs: AuthInputs,
        cert_ctx: CertificationContext,
    ) -> Result<Record> {
        let certification = certify(auth_inputs, &cert_ctx)?;
        let v = vector_state.vector.clone();

        if v.is_zero() && vector_state.tau != crate::types::VectorTypeTag::Zero {
            return Err(KernelError::ZeroVectorViolation(
                "non-zero type cannot host a zero vector on create".to_string(),
            ));
        }

        let certification_state = certification.state.clone();
        let auth_ratio = certification.auth_ratio.clone();

        let record = Record {
            record_id: self.next_record_id("create"),
            parent_record_ids: vec![],
            event_kind: OperationKind::Create,
            space_id: self.context.space_id.clone(),
            region_id: self.context.region_id.clone(),
            entity_id: EntityId(wallet.wallet_id.clone()),
            projection_id: None,
            v_before: Vector::zero(v.len()),
            v_after: v.clone(),
            parameters: serde_json::json!({
                "wallet": wallet.clone(),
                "vector_state": vector_state.clone(),
                "certification": certification.clone(),
            }),
            certification_state,
            auth_ratio,
            timestamp: self.context.timestamp,
            logical_clock: self.context.logical_clock,
            proof: None,
            initiator_pk: vector_state.owner_pk.clone(),
            signature: None,
            hash: String::new(),
            version: self.context.protocol_version.clone(),
            settlement_ref: None,
            is_failure: false,
            failure_reason: None,
        };

        let record = self.append_record(record)?;
        self.derived.entity_states.insert(
            wallet.wallet_id.clone(),
            LiveEntityState {
                entity_id: EntityId(wallet.wallet_id.clone()),
                wallet,
                vector_state,
                certification,
                active_projection: None,
            },
        );
        Ok(record)
    }

    pub fn certify_entity(
        &mut self,
        entity_id: &EntityId,
        auth_inputs: AuthInputs,
        cert_ctx: CertificationContext,
    ) -> Result<Record> {
        let report = certify(auth_inputs, &cert_ctx)?;

        let (v, initiator_pk, certification_state, auth_ratio) = {
            let state = self
                .derived
                .entity_states
                .get_mut(&entity_id.0)
                .ok_or_else(|| KernelError::NotFound(entity_id.0.clone()))?;

            state.certification = report.clone();

            (
                state.vector_state.vector.clone(),
                state.vector_state.owner_pk.clone(),
                state.certification.state.clone(),
                state.certification.auth_ratio.clone(),
            )
        };

        let record = Record {
            record_id: self.next_record_id("certify"),
            parent_record_ids: vec![],
            event_kind: OperationKind::Certify,
            space_id: self.context.space_id.clone(),
            region_id: self.context.region_id.clone(),
            entity_id: entity_id.clone(),
            projection_id: None,
            v_before: v.clone(),
            v_after: v,
            parameters: serde_json::json!({ "certification": report }),
            certification_state,
            auth_ratio,
            timestamp: self.context.timestamp,
            logical_clock: self.context.logical_clock,
            proof: None,
            initiator_pk,
            signature: None,
            hash: String::new(),
            version: self.context.protocol_version.clone(),
            settlement_ref: None,
            is_failure: false,
            failure_reason: None,
        };

        self.append_record(record)
    }

    pub fn project(
        &mut self,
        request: ProjectionRequest,
        contract_ctx: Option<ContractContext>,
    ) -> Result<Record> {
        let live = self
            .derived
            .entity_states
            .get(&request.source_entity_id.0)
            .ok_or_else(|| KernelError::NotFound(request.source_entity_id.0.clone()))?
            .clone();

        if live.certification.state != CertificationState::Certified {
            return Err(KernelError::CertificationFailed(
                "source entity must be certified before projection".to_string(),
            ));
        }

        let projection = build_projection_envelope(&request, self.next_record_id("project"))?;

        if let Some(contract_ctx) = contract_ctx.as_ref() {
            if let Some(contract) = self.contracts.get(&contract_ctx.contract_id) {
                let decision = contract.evaluate(contract_ctx)?;
                if !decision.allow {
                    return Err(KernelError::ContractError(
                        decision.reason.unwrap_or_else(|| "contract denied projection".to_string()),
                    ));
                }
            }
        }

        let record = Record {
            record_id: projection.created_record_id.clone(),
            parent_record_ids: request.parent_record_ids.clone(),
            event_kind: OperationKind::Project,
            space_id: self.context.space_id.clone(),
            region_id: self.context.region_id.clone(),
            entity_id: request.source_entity_id.clone(),
            projection_id: Some(request.projection_id.clone()),
            v_before: request.source_state.vector.clone(),
            v_after: projection.remaining_vector.clone(),
            parameters: serde_json::json!({
                "projection_id": request.projection_id.clone(),
                "locked_vector": projection.locked_vector.clone(),
                "remaining_vector": projection.remaining_vector.clone(),
                "lock_kind": projection.lock_kind,
                "rule_environment": projection.rule_environment.clone(),
                "settlement_rule": projection.settlement_rule.clone(),
                "settlement_trigger": projection.settlement_trigger,
                "drain_policy": projection.drain_policy,
                "certification_threshold": projection.certification_threshold.clone(),
            }),
            certification_state: live.certification.state.clone(),
            auth_ratio: live.certification.auth_ratio.clone(),
            timestamp: self.context.timestamp,
            logical_clock: self.context.logical_clock,
            proof: Some("projection-lock".to_string()),
            initiator_pk: request.created_by_pk.clone(),
            signature: None,
            hash: String::new(),
            version: self.context.protocol_version.clone(),
            settlement_ref: None,
            is_failure: false,
            failure_reason: None,
        };

        let record = self.append_record(record)?;
        self.derived.projection_states.insert(request.projection_id.0.clone(), projection.clone());
        if let Some(state) = self.derived.entity_states.get_mut(&request.source_entity_id.0) {
            state.vector_state.vector = projection.remaining_vector.clone();
            state.active_projection = Some(projection);
        }
        Ok(record)
    }

    pub fn reconstruct(
        &mut self,
        request: ReconstructionRequest,
        contract_decision: Option<ContractDecision>,
    ) -> Result<(Record, ReconstructionReceipt)> {
        let settlement_rule = request.projection.settlement_rule.clone();
        let effective_outcome = contract_decision
            .and_then(|d| d.settlement_outcome)
            .unwrap_or(request.outcome.clone());
        let recon_request = ReconstructionRequest {
            projection: request.projection,
            outcome: effective_outcome,
            reconstructed_record_id: self.next_record_id("reconstruct"),
            settlement_reference: request.settlement_reference,
        };
        let (updated_projection, receipt) = reconstruct_projection(recon_request, &settlement_rule)?;

        let record = Record {
            record_id: receipt.settlement_record_id.clone(),
            parent_record_ids: vec![receipt.prior_record_id.clone()],
            event_kind: OperationKind::Reconstruct,
            space_id: self.context.space_id.clone(),
            region_id: self.context.region_id.clone(),
            entity_id: EntityId(receipt.projection_id.0.clone()),
            projection_id: Some(receipt.projection_id.clone()),
            v_before: updated_projection.original_vector.clone(),
            v_after: receipt.resulting_vector.clone(),
            parameters: serde_json::json!({
                "receipt": receipt.clone(),
                "projection_status": updated_projection.status.clone(),
            }),
            certification_state: CertificationState::Certified,
            auth_ratio: AuthRatio::new(1_000_000).unwrap(),
            timestamp: self.context.timestamp,
            logical_clock: self.context.logical_clock,
            proof: Some("reconstruction-settlement".to_string()),
            initiator_pk: updated_projection.created_by_pk.clone(),
            signature: None,
            hash: String::new(),
            version: self.context.protocol_version.clone(),
            settlement_ref: Some("projection-settlement".to_string()),
            is_failure: false,
            failure_reason: None,
        };

        let record = self.append_record(record)?;
        if let Some(state) = self.derived.entity_states.get_mut(&updated_projection.source_entity_id.0) {
            state.vector_state.vector = updated_projection.remaining_vector.clone();
            let projection_status = updated_projection.status.clone();
            state.active_projection = match projection_status {
                ProjectionStatus::Settled => None,
                _ => Some(updated_projection.clone()),
            };
        }
        self.derived.projection_states.insert(updated_projection.projection_id.0.clone(), updated_projection);
        Ok((record, receipt))
    }

    pub fn query_entity(&self, entity_id: &EntityId) -> Result<LiveEntityState> {
        self.derived
            .entity_states
            .get(&entity_id.0)
            .cloned()
            .ok_or_else(|| KernelError::NotFound(entity_id.0.clone()))
    }
}