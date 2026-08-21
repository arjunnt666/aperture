# aperture

traffic shaping you can unit test in a few milliseconds.

token bucket. sliding window. bulkhead budget. circuit breaker knobs. not a service mesh, not envoy, not a sidecar you forget to restart.

## works today

- token bucket allows then denies when burst is spent
- sliding window respects burst
- bulkhead enter/exit
- `aperture version`

## does not work yet

- distributed limit coordination
- adaptive control from real latency traces

## try it

```bash
cargo test --workspace
cargo build -p aperture-cli
./target/debug/aperture version
```

## license

mit. deny is a feature.
