FROM rust:1.62.0 as build-env
ARG toolchain
ARG version
RUN rustup default $toolchain

WORKDIR /app
COPY . .
WORKDIR /app/apps/mproc
RUN sed 's/version = "0.0.0"/version = "'$version'"/g' Cargo.toml > Cargo.toml.tmp
RUN mv Cargo.toml.tmp Cargo.toml
RUN cargo build --release

FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=build-env /app/apps/mproc/target/release/spoderman-mproc .
USER 1000
CMD ["./spoderman-mproc", "work", "-c", "./config/config.yaml"]
