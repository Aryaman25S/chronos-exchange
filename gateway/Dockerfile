FROM rust:1.79 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY engine ./engine
COPY risk ./risk
COPY md ./md
COPY gateway ./gateway
RUN cargo build --release -p gateway

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/gateway /usr/local/bin/gateway
COPY markets_seed.json /etc/chronos/markets_seed.json
ENV DATA_DIR=/data
ENV MARKETS_SEED=/etc/chronos/markets_seed.json
EXPOSE 8081
CMD ["gateway"]
