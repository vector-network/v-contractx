import { useMemo } from "react";
import type { ProjectionRequest } from "../src/types.js";

export function useProjectionPreview(request: ProjectionRequest) {
  return useMemo(() => {
    const remaining = request.sourceVector.components.map((value, index) => {
      const locked = request.lockedAmount.components[index] ?? 0n;
      return value >= locked ? value - locked : 0n;
    });

    return {
      remaining,
      locked: request.lockedAmount.components,
      totalLocked: request.lockedAmount.components.reduce((acc, value) => acc + value, 0n),
    };
  }, [request]);
}
