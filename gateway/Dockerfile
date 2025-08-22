FROM rust:1.79 as builder
WORKDIR /app
COPY ../Cargo.toml ../rust-toolchain.toml ./
COPY ../engine ../engine
COPY ../risk ../risk
COPY ../md ../md
COPY . ./gateway
RUN cargo build --release -p gateway

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/gateway /usr/local/bin/gateway
ENV DATA_DIR=/data
EXPOSE 8081
CMD ["gateway"]
