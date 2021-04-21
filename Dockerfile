# --- build stage ---
FROM docker.io/rust:alpine AS builder
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /systemd-fail-notifications

# fetch dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch
RUN apk add --no-cache musl-dev
RUN cargo install cargo-about

# final build
COPY src ./src
COPY *.rs *.toml *.hbs ./
RUN cargo build --target x86_64-unknown-linux-musl --release

# --- bundle stage ---
FROM scratch
WORKDIR /
COPY --from=builder /systemd-fail-notifications/target/x86_64-unknown-linux-musl/release/systemd-fail-notifications .
ENTRYPOINT ["/systemd-fail-notifications"]
