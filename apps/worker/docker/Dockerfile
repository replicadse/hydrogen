FROM rust:1.61.0 as build-env
ARG toolchain
ARG version
RUN rustup default $toolchain

WORKDIR /app
COPY . .
RUN sed 's/version = "0.0.0"/version = "'$version'"/g' Cargo.toml > Cargo.toml.tmp
RUN mv Cargo.toml.tmp Cargo.toml
RUN cargo build --release

FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=build-env /app/target/release/spoderman-worker .
USER 1000
CMD ["./spoderman-worker", "work", "-c", "./config/config.yaml"]
