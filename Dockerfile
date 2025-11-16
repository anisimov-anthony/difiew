FROM rust:1.91-slim AS builder
WORKDIR /difiew

COPY . .
RUN cargo build --release --bin node --bin manager

FROM ubuntu:24.04

COPY --from=builder /difiew/target/release/node /usr/local/bin/node
COPY --from=builder /difiew/target/release/manager /usr/local/bin/manager

RUN chmod +x /usr/local/bin/node /usr/local/bin/manager

CMD ["/bin/bash", "-c", "exec /usr/local/bin/${BIN:-node} ${ARGS:-}"]
