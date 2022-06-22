FROM rust:1.58.1 as build-env
RUN rustup default nightly

WORKDIR /app
COPY . .
RUN cargo +nightly build --release

FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=build-env /app/target/release/spoderman .
USER 1000
CMD ["./spoderman", "serve", "-c", "./config/config.yaml"]
