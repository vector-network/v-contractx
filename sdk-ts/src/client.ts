import type { ProjectionRequest, ProjectionEnvelope, SettlementOutcome } from "./types.js";

export interface VContractClientOptions {
  baseUrl: string;
  fetchImpl?: typeof fetch;
}

export class VContractClient {
  private readonly baseUrl: string;
  private readonly fetchImpl: typeof fetch;

  constructor(options: VContractClientOptions) {
    this.baseUrl = options.baseUrl.replace(/\/$/, "");
    this.fetchImpl = options.fetchImpl ?? fetch;
  }

  async project(request: ProjectionRequest): Promise<ProjectionEnvelope> {
    const response = await this.fetchImpl(`${this.baseUrl}/project`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(request),
    });
    if (!response.ok) {
      throw new Error(`project failed: ${response.status} ${response.statusText}`);
    }
    return response.json();
  }

  async reconstruct(projectionId: string, outcome: SettlementOutcome): Promise<unknown> {
    const response = await this.fetchImpl(`${this.baseUrl}/reconstruct`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ projectionId, outcome }),
    });
    if (!response.ok) {
      throw new Error(`reconstruct failed: ${response.status} ${response.statusText}`);
    }
    return response.json();
  }
}
