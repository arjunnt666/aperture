# Tuning

adaptive concurrency is sensitive to sample window and smoothing.

skeleton defaults:
- min limit: 1
- max limit: 200
- initial: 20
- target latency: 50ms
- smoothing alpha: 0.2

if the limit oscillates, increase the sample window before changing alpha.
if it never climbs, your error signal may be too noisy.
