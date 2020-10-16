FROM rust:1.47 as builder
RUN apt-get update && apt-get install -y openssl libssl-dev pkg-config
WORKDIR /usr/src/semantic-rs
COPY . .
RUN cargo install --path .

FROM rust:1.47-slim
ENV RUST_LOG=info
RUN apt-get update && apt-get install -y \
    ca-certificates git openssl libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/semantic-rs /usr/local/bin/semantic-rs
WORKDIR /home
ENTRYPOINT ["/usr/local/bin/semantic-rs"]
