# aperture

adaptive concurrency and traffic shaping.

rate limits. circuit breakers. bulkheads. load shedding.
not a service mesh. not istio. just the pieces you reach for when the happy path stops being happy.

## why

you start with a simple request path.
then production happens.
someone floods an endpoint.
a dependency starts timing out.
threads pile up.
everything slows down together.

aperture is a small toolkit for those moments.
token buckets. sliding windows. failure thresholds. concurrency caps.
the algorithms are familiar. the packaging is meant to stay out of the way.

## what is here

- rate limiting with token bucket and sliding window variants
- circuit breakers with open half open closed states
- bulkheads that isolate thread or semaphore budgets
- adaptive concurrency that reacts to latency and error rate
- basic metrics hooks so you can see what is happening
- a thin server and client layer for embedding

## status

early skeleton.
the types and control flow exist.
the production grade tuning and distributed coordination are still future work.

do not put your busiest production path on this yet.
do poke at the structure if you care how these controls fit together.

## crates

- aperture-core geometry for limits and clocks
- aperture-limiter token bucket and window limiters
- aperture-breaker circuit breaker state machine
- aperture-bulkhead isolation and semaphore budgets
- aperture-adaptive latency aware concurrency
- aperture-metrics simple counters and histograms
- aperture-server embeddable control plane stub
- aperture-client thin client wrappers
- aperture-cli local experiments

js and python packages live under packages.

## license

mit. take the ideas. improve the code. do not blame the library when the limit was set too tight.
