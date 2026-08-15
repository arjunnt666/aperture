# Architecture

Aperture is split into focused crates:

1. core clocks decisions and shared config
2. limiter token bucket and sliding window
3. breaker circuit breaker state machine
4. bulkhead concurrency isolation
5. adaptive latency aware concurrency adjustment
6. metrics simple counters
7. server control plane that wires the pieces
8. client thin wrapper for local use

The goal is a small embeddable toolkit rather than a full service mesh.
