import React from "react";
import type { ProjectionRequest } from "../src/types.js";

export interface ProjectionPanelProps {
  request: ProjectionRequest;
  onSubmit: (request: ProjectionRequest) => void;
}

export function ProjectionPanel({ request, onSubmit }: ProjectionPanelProps) {
  const lockedSummary = request.lockedAmount.components.map((c) => c.toString()).join(", ");
  const liveSummary = request.sourceVector.components.map((c) => c.toString()).join(", ");

  return (
    <section>
      <h2>Projection</h2>
      <dl>
        <dt>Projection ID</dt>
        <dd>{request.projectionId}</dd>
        <dt>Source entity</dt>
        <dd>{request.sourceEntityId}</dd>
        <dt>Live vector</dt>
        <dd>{liveSummary}</dd>
        <dt>Locked vector</dt>
        <dd>{lockedSummary}</dd>
        <dt>Rule environment</dt>
        <dd>{request.ruleEnvironment}</dd>
      </dl>
      <button type="button" onClick={() => onSubmit(request)}>
        Commit stake
      </button>
    </section>
  );
}
