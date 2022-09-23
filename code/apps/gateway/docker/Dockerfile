FROM rust:1.64.0 as build-env
ARG toolchain
ARG version
RUN rustup default $toolchain

WORKDIR /code
COPY ./.rust ./.rust
COPY ./libs ./libs
COPY ./apps/gateway ./apps/gateway
RUN ls -alghR /code
WORKDIR /code/apps/gateway
RUN sed 's/version = "0.0.0"/version = "'$version'"/g' Cargo.toml > Cargo.toml.tmp
RUN mv Cargo.toml.tmp Cargo.toml
RUN cargo build --release

FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=build-env /code/apps/gateway/target/release/hydrogen-gateway .
USER 1000
CMD ["./hydrogen-gateway", "serve", "-c", "./config/config.yaml"]
