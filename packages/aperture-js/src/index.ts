export type Decision = "allow" | "deny" | "shed";

export interface Outcome {
  decision: Decision;
  remaining?: number;
  retryAfterMs?: number;
}

export class ApertureClient {
  private tokens = 20;
  check(_name: string = "default"): Outcome {
    if (this.tokens > 0) {
      this.tokens -= 1;
      return { decision: "allow", remaining: this.tokens };
    }
    return { decision: "deny", remaining: 0, retryAfterMs: 1000 };
  }
  release() {
    this.tokens = Math.min(20, this.tokens + 1);
  }
}

export default ApertureClient;
