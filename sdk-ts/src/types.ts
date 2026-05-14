export type ComponentAmount = bigint;

export interface Vector {
  components: ComponentAmount[];
}

export enum VectorTypeTag {
  Position = "Position",
  Free = "Free",
  Bound = "Bound",
  Unit = "Unit",
  Zero = "Zero",
  Spatial = "Spatial",
}

export enum ProjectionLockKind {
  Stake = "Stake",
  DirectedAllocation = "DirectedAllocation",
  RiskEscrow = "RiskEscrow",
  TaskLock = "TaskLock",
}

export type SettlementTrigger =
  | { kind: "Manual" }
  | { kind: "HeightReached"; height: bigint }
  | { kind: "TimeReached"; timestamp: bigint }
  | { kind: "ContractSignal" }
  | { kind: "OracleSignal" };

export type DrainPolicy =
  | { kind: "None" }
  | { kind: "FixedBps"; bps: number };

export interface SettlementRule {
  allowGain: boolean;
  allowLoss: boolean;
  allowPartialRelease: boolean;
  allowStagedRelease: boolean;
  maxPhases: number;
}

export interface ProjectionRequest {
  projectionId: string;
  sourceEntityId: string;
  sourceSpaceId: string;
  contractId?: string | null;
  sourceVector: Vector;
  lockedAmount: Vector;
  lockKind: ProjectionLockKind;
  ruleEnvironment: string;
  settlementRule: SettlementRule;
  settlementTrigger: SettlementTrigger;
  drainPolicy: DrainPolicy;
  certificationThreshold: number;
  createdByPk: string;
  parentRecordIds: string[];
}

export interface ProjectionEnvelope {
  projectionId: string;
  sourceEntityId: string;
  sourceSpaceId: string;
  contractId?: string | null;
  originalVector: Vector;
  remainingVector: Vector;
  lockedVector: Vector;
  escrowVector: Vector;
  lockKind: ProjectionLockKind;
  ruleEnvironment: string;
  settlementRule: SettlementRule;
  settlementTrigger: SettlementTrigger;
  drainPolicy: DrainPolicy;
  certificationThreshold: number;
  status: "Active" | "Settling" | "Settled" | { staged: number } | "Revoked";
  createdRecordId: string;
  settlementRecordId?: string | null;
  createdByPk: string;
}

export type SettlementOutcome =
  | { kind: "NoChange" }
  | { kind: "Gain"; amount: Vector }
  | { kind: "Loss"; amount: Vector }
  | { kind: "PartialRelease"; amount: Vector }
  | { kind: "StagedRelease"; released: Vector; remainingLocked: Vector; remainingPhases: number };
