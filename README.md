Aperture is one `check()` that stacks a few admission rules.

if the circuit breaker is open, the request dies even if the token bucket still has tokens.
if the burst is spent, the bucket denies.
if concurrent slots are gone, the bulkhead sheds.
if inflight hits the adaptive limit, that sheds too.
order in the stack: breaker, adaptive, bucket, window, bulkhead.

I wrote this so I could unit test traffic shaping in a few milliseconds. there is no sidecar to forget to restart.

`aperture demo --requests 12` uses a tight burst on purpose. it will refuse to print a fake all green run.

What I have not built: coordinating limits across processes, and feeding adaptive from real production traces. the math is local.

```bash
cargo test --workspace
cargo build -p aperture-cli
./target/debug/aperture demo --requests 12
```

MIT. deny is a feature.
