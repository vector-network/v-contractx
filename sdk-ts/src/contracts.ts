import type { ProjectionEnvelope, SettlementOutcome, SettlementRule, Vector } from "./types.js";

export interface ContractPreviewInput {
  envelope: ProjectionEnvelope;
  requestedOutcome: SettlementOutcome;
}

export interface ContractPreview {
  accepted: boolean;
  settlementRule: SettlementRule;
  projectedStake: Vector;
  liveRemaining: Vector;
  projectedEscrow: Vector;
  summary: string;
}

export function previewDefaultStakeContract(input: ContractPreviewInput): ContractPreview {
  const { envelope, requestedOutcome } = input;

  const accepted = envelope.status === "Active" || typeof envelope.status === "object";
  const projectedStake = envelope.lockedVector;

  let summary = "projection accepted";
  if (requestedOutcome.kind === "Gain") {
    summary = "gain settlement preview";
  } else if (requestedOutcome.kind === "Loss") {
    summary = "loss settlement preview";
  } else if (requestedOutcome.kind === "PartialRelease") {
    summary = "partial release preview";
  } else if (requestedOutcome.kind === "StagedRelease") {
    summary = "staged settlement preview";
  }

  return {
    accepted,
    settlementRule: envelope.settlementRule,
    projectedStake,
    liveRemaining: envelope.remainingVector,
    projectedEscrow: envelope.escrowVector,
    summary,
  };
}
