VERSION 0.6

rust:
  FROM alpine:3.16
  ARG toolchain
  RUN [ ! -z "$toolchain" ] || exit 1
  RUN apk update && apk upgrade
  RUN apk add bash coreutils make git curl ca-certificates build-base libc-dev musl-dev alpine-sdk gcc rustup
  ENV PATH=/root/.cargo/bin:"$PATH"
  RUN rustup-init -y
  RUN rustup default $toolchain

retype:
  FROM node:18-buster
  RUN npm i -g retypeapp

code:
  FROM +rust
  WORKDIR /app
  COPY ./spoderman .

release:
  FROM +code
  RUN cargo build -Zunstable-options --locked --out-dir=./out
  SAVE ARTIFACT ./out/spoderman AS LOCAL ./.artifacts/spoderman

fmt:
  FROM +code
  RUN cargo fmt --all -- --check

test:
  FROM +code
  ARG features
  RUN cargo test $features

docs:
  FROM +code
  RUN cargo doc --no-deps --document-private-items --all-features
  SAVE ARTIFACT ./target/doc AS LOCAL ./.artifacts/docs

wiki:
  FROM +retype
  WORKDIR /app
  COPY ./docs/* .
  RUN ls -alghR
  RUN retype build
  SAVE ARTIFACT .retype AS LOCAL ./.artifacts/wiki
