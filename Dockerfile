# --- build stage ---
FROM rust:latest AS builder
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /build

# fetch dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

COPY src ./src
RUN cargo build --target x86_64-unknown-linux-musl --release

# --- bundle stage ---
FROM scratch
WORKDIR /
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/systemd-fail-notifications .
CMD ["/systemd-fail-notifications"]
