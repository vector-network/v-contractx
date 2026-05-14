"""Projection / reconstruction simulation for v-contractx.

This file is intentionally dependency-light and uses integer accounting so it
mirrors the kernel rules in a deterministic way.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import List


class OutcomeKind(str, Enum):
    NO_CHANGE = "NoChange"
    GAIN = "Gain"
    LOSS = "Loss"
    PARTIAL_RELEASE = "PartialRelease"
    STAGED_RELEASE = "StagedRelease"


@dataclass(frozen=True)
class Vector:
    components: List[int]

    def check_dim(self, other: "Vector") -> None:
        if len(self.components) != len(other.components):
            raise ValueError(f"dimension mismatch {len(self.components)} != {len(other.components)}")

    def is_zero(self) -> bool:
        return all(c == 0 for c in self.components)

    def sub(self, other: "Vector") -> "Vector":
        self.check_dim(other)
        out = []
        for a, b in zip(self.components, other.components):
            if b > a:
                raise ValueError("cannot subtract a larger vector component from a smaller one")
            out.append(a - b)
        return Vector(out)

    def add(self, other: "Vector") -> "Vector":
        self.check_dim(other)
        return Vector([a + b for a, b in zip(self.components, other.components)])


@dataclass(frozen=True)
class SettlementRule:
    allow_gain: bool = True
    allow_loss: bool = True
    allow_partial_release: bool = True
    allow_staged_release: bool = False
    max_phases: int = 1


@dataclass(frozen=True)
class ProjectionEnvelope:
    projection_id: str
    source_entity_id: str
    original_vector: Vector
    remaining_vector: Vector
    locked_vector: Vector
    escrow_vector: Vector
    rule_environment: str
    settlement_rule: SettlementRule
    status: str = "Active"


@dataclass(frozen=True)
class SettlementResult:
    outcome: OutcomeKind
    resulting_vector: Vector
    returned_to_live: Vector
    locked_remaining: Vector
    status: str
    note: str


def project(source: Vector, locked: Vector, projection_id: str = "proj-1", source_entity_id: str = "wallet-a") -> ProjectionEnvelope:
    source.check_dim(locked)
    if locked.is_zero():
        raise ValueError("locked vector must not be zero")
    remaining = source.sub(locked)
    return ProjectionEnvelope(
        projection_id=projection_id,
        source_entity_id=source_entity_id,
        original_vector=source,
        remaining_vector=remaining,
        locked_vector=locked,
        escrow_vector=locked,
        rule_environment="task-a",
        settlement_rule=SettlementRule(),
    )


def reconstruct(envelope: ProjectionEnvelope, outcome: OutcomeKind, amount: Vector | None = None) -> SettlementResult:
    if envelope.status not in {"Active", "Staged"}:
        raise ValueError("projection is not settleable")

    locked = envelope.locked_vector
    zero = Vector([0] * len(locked.components))

    if outcome == OutcomeKind.NO_CHANGE:
        live = envelope.remaining_vector.add(locked)
        return SettlementResult(outcome, live, zero, zero, "Settled", "no change")

    if amount is None:
        amount = zero

    if outcome == OutcomeKind.GAIN:
        if not envelope.settlement_rule.allow_gain:
            raise ValueError("gain disallowed")
        live = envelope.remaining_vector.add(locked).add(amount)
        return SettlementResult(outcome, live, amount, zero, "Settled", "gain settled")

    if outcome == OutcomeKind.LOSS:
        if not envelope.settlement_rule.allow_loss:
            raise ValueError("loss disallowed")
        live = envelope.remaining_vector.add(locked).sub(amount)
        return SettlementResult(outcome, live, zero, zero, "Settled", "loss settled")

    if outcome == OutcomeKind.PARTIAL_RELEASE:
        if not envelope.settlement_rule.allow_partial_release:
            raise ValueError("partial release disallowed")
        remaining_locked = locked.sub(amount)
        live = envelope.remaining_vector.add(amount)
        return SettlementResult(outcome, live, amount, remaining_locked, "Settled", "partial release settled")

    if outcome == OutcomeKind.STAGED_RELEASE:
        if not envelope.settlement_rule.allow_staged_release:
            raise ValueError("staged release disallowed")
        remaining_locked = locked.sub(amount)
        live = envelope.remaining_vector.add(amount)
        return SettlementResult(outcome, live, amount, remaining_locked, "Staged", "staged release settled")

    raise ValueError(f"unknown outcome: {outcome}")


if __name__ == "__main__":
    source = Vector([10, 5, 0])
    locked = Vector([5, 0, 0])
    envelope = project(source, locked)
    result = reconstruct(envelope, OutcomeKind.GAIN, Vector([2, 0, 0]))
    print("projection:", envelope)
    print("reconstruction:", result)