# Projection and reconstruction flow

1. The caller submits a projection request.
2. The kernel validates ownership, type compatibility, certification, and zero-vector safety.
3. The locked portion is isolated and persisted as a projection envelope.
4. The contract layer evaluates access control and settlement policy.
5. Reconstruction consumes the projection exactly once unless the settlement rule explicitly enables stages.
6. A settlement record is appended and the live cache is updated.
