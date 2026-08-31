FROM rust:1.80-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p aperture-cli

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/aperture /usr/local/bin/aperture
CMD ["aperture", "version"]
