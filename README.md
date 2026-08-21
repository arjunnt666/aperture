# aperture

traffic shaping you can unit test in a few milliseconds.

token bucket. sliding window. bulkhead budget. circuit breaker. adaptive concurrency stacked in one `check()`. not a service mesh, not envoy, not a sidecar you forget to restart.

## works today

- stacked admission: breaker, adaptive, bucket, window, bulkhead
- token bucket allows then denies when burst is spent
- open breaker denies even if tokens remain
- bulkhead sheds when concurrent slots are gone
- adaptive sheds when inflight hits the limit
- `aperture demo --requests 12` uses a tight burst and refuses to print a fake all-green run

## does not work yet

- distributed limit coordination across processes
- adaptive control from real production traces (the math is local)

## try it

```bash
cargo test --workspace
cargo build -p aperture-cli
./target/debug/aperture demo --requests 12
```

## license

mit. deny is a feature.
