FROM rust:1.61.0 as build-env
ARG toolchain
ARG version
RUN rustup default $toolchain

WORKDIR /app
COPY . .
WORKDIR /app/apps/gateway
RUN ls -algh /app
RUN sed 's/version = "0.0.0"/version = "'$version'"/g' Cargo.toml > Cargo.toml.tmp
RUN mv Cargo.toml.tmp Cargo.toml
RUN cargo build --release

FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=build-env /app/apps/gateway/target/release/hydrogen-gateway .
USER 1000
CMD ["./hydrogen-gateway", "serve", "-c", "./config/config.yaml"]
