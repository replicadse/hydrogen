FROM rust:1.74.0 as build-env
ARG toolchain
ARG version
RUN rustup default $toolchain

WORKDIR /code
COPY ./.rust ./.rust
COPY ./libs ./libs
COPY ./apps/mproc ./apps/mproc
RUN ls -alghR /code
WORKDIR /code/apps/mproc
RUN sed 's/version = "0.0.0"/version = "'$version'"/g' Cargo.toml > Cargo.toml.tmp
RUN mv Cargo.toml.tmp Cargo.toml
RUN cargo build --release

FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=build-env /code/apps/mproc/target/release/hydrogen-mproc .
USER 1000
CMD ["./hydrogen-mproc", "work", "-c", "./config/config.yaml"]
