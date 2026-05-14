use v_contractx::{
    certification::{AuthInputs, AuthWeights, CertificationContext, CertificationThreshold, MicroScore},
    projection::{DrainPolicy, ProjectionLockKind, ProjectionRequest, SettlementRule, SettlementTrigger},
    reconstruction::{ReconstructionRequest, SettlementOutcome},
    runtime::{Kernel, KernelContext},
    types::{EntityId, OperationKind, ProjectionId, RegionId, SpaceId, Vector, VectorState, VectorTypeTag, Wallet},
};

fn cert_ctx() -> CertificationContext {
    CertificationContext {
        vector_type: "free".to_string(),
        operation_class: "projection".to_string(),
        space_policy_version: "1".to_string(),
        risk_profile: "standard".to_string(),
        protocol_version: "1.1".to_string(),
        threshold: CertificationThreshold::new(600_000).unwrap(),
        weights: AuthWeights::normalized(400_000, 300_000, 200_000, 100_000).unwrap(),
    }
}

#[test]
fn projection_isolate_and_reconstruct() {
    let mut kernel = Kernel::new(KernelContext {
        protocol_version: "1.1".to_string(),
        space_id: SpaceId("space-a".to_string()),
        region_id: RegionId("region-1".to_string()),
        timestamp: 1,
        logical_clock: 1,
    });

    let wallet = Wallet {
        wallet_id: "wallet-a".to_string(),
        pk: "pk-a".to_string(),
        wallet_meta: serde_json::json!({}),
    };

    let vector = VectorState {
        vector: Vector::new(vec![10, 5, 0]),
        owner_pk: "pk-a".to_string(),
        tau: VectorTypeTag::Free,
        meta: serde_json::json!({}),
    };

    let auth_inputs = AuthInputs {
        magnitude: MicroScore::new(900_000).unwrap(),
        composition: MicroScore::new(800_000).unwrap(),
        ownership: MicroScore::new(700_000).unwrap(),
        policy: MicroScore::new(600_000).unwrap(),
    };

    let create_record = kernel
        .create_entity(wallet.clone(), vector.clone(), auth_inputs.clone(), cert_ctx())
        .unwrap();
    assert_eq!(create_record.event_kind, OperationKind::Create);

    let request = ProjectionRequest {
        projection_id: ProjectionId("proj-1".to_string()),
        source_entity_id: EntityId("wallet-a".to_string()),
        source_space_id: SpaceId("space-a".to_string()),
        contract_id: None,
        source_state: vector.clone(),
        locked_amount: Vector::new(vec![5, 0, 0]),
        lock_kind: ProjectionLockKind::Stake,
        rule_environment: "task-a".to_string(),
        settlement_rule: SettlementRule::immediate(),
        settlement_trigger: SettlementTrigger::Manual,
        drain_policy: DrainPolicy::None,
        certification_threshold: CertificationThreshold::new(500_000).unwrap(),
        created_by_pk: "pk-a".to_string(),
        parent_record_ids: vec![create_record.record_id.clone()],
    };

    let project_record = kernel.project(request.clone(), None).unwrap();
    assert_eq!(project_record.event_kind, OperationKind::Project);

    let projection = kernel.derived.projection_states.get("proj-1").unwrap().clone();
    assert_eq!(projection.locked_vector, Vector::new(vec![5, 0, 0]));
    assert_eq!(projection.remaining_vector, Vector::new(vec![5, 5, 0]));

    let recon_request = ReconstructionRequest {
        projection,
        outcome: SettlementOutcome::Gain {
            amount: Vector::new(vec![2, 0, 0]),
        },
        reconstructed_record_id: v_contractx::record::make_record_id("reconstruct", 1),
        settlement_reference: "settlement-1".to_string(),
    };

    let (record, receipt) = kernel.reconstruct(recon_request, None).unwrap();
    assert_eq!(record.event_kind, OperationKind::Reconstruct);
    assert_eq!(receipt.resulting_vector, Vector::new(vec![12, 5, 0]));
}