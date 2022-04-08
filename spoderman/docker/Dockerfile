FROM rust:1.60.0 as build-env
RUN rustup default nightly

WORKDIR /app
COPY Cargo.toml .
COPY Cargo.lock .
RUN mkdir -p src
RUN printf "fn main() {}" > src/main.rs
RUN cargo +nightly build --release
RUN rm -rf ./src
COPY . .
RUN cargo +nightly build --release

FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=build-env /app/target/release/spoderman .
USER 1000
CMD ["./spoderman", "serve", "-c", "./config/config.yaml"]
