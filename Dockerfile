FROM rust:latest AS builder

# Set the working directory inside the container
WORKDIR /usr/src/app

ENV CARGO_HOME=/usr/local/cargo-persistent

# Copy the Cargo.toml and Cargo.lock files
COPY ./ ./

COPY ./cargo-config.toml /usr/local/cargo-persistent/config.toml
COPY ./cargo-config.toml ./.cargo/config.toml

# build
RUN --mount=type=cache,target=/usr/local/cargo-persistent \
    cargo build --release

FROM debian:stable-slim

# Set the working directory inside the container
WORKDIR /usr/src/app

COPY --from=builder /usr/src/app/target/release/blossom_backend_rs ./
COPY --from=builder /usr/src/app/Rocket.toml ./
COPY --from=builder /usr/src/app/templates ./templates
#COPY --from=builder /usr/src/app/.env ./

CMD ["./blossom_backend_rs"]