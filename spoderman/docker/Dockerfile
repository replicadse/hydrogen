# FROM rust:1.58.1 as build-env
# RUN rustup default nightly

# WORKDIR /app
# COPY Cargo.toml .
# COPY Cargo.lock .
# RUN mkdir -p src
# RUN printf "fn main() {}" > src/main.rs
# RUN cargo +nightly build --release
# RUN rm -rf ./src
# COPY . .
# RUN cargo +nightly build --release

# FROM gcr.io/distroless/cc
# WORKDIR /app
# COPY --from=build-env /app/target/release/spoderman .
# USER 1000
# CMD ["./spoderman", "serve", "-c", "./config/config.yaml"]

FROM alpine:3.15 as build-env
ENV LANG en_US.UTF-8
RUN apk update && apk upgrade
RUN apk add \
  bash \
  coreutils \
  make \
  git \
  curl \
  ca-certificates \
  build-base \
  libc-dev \
  musl-dev \
  alpine-sdk \
  gcc \
  rustup
ENV PATH=/root/.cargo/bin:"$PATH"
RUN rustup-init -y
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

FROM alpine:3.15 as runtime-env
ENV LANG en_US.UTF-8
RUN apk update && apk upgrade
RUN apk add bash
WORKDIR /app
COPY --from=build-env /app/target/release/spoderman .
RUN addgroup -g 1000 app && adduser -D -s /bin/sh -u 1000 -G app app
USER app
CMD ["./spoderman", "serve", "-c", "./config/config.yaml"]
# CMD ["/bin/bash"]
