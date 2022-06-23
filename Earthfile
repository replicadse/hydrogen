VERSION 0.6

rust:
  FROM debian:bullseye
  ARG toolchain
  RUN [ ! -z "$toolchain" ] || exit 1
  RUN apt update && apt upgrade
  RUN apt install gcc make build-essential curl -y
  RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain $toolchain
  ENV PATH=/root/.cargo/bin:"$PATH"

spoderman:
  FROM +rust
  WORKDIR /app
  COPY ./spoderman .

retype:
  FROM node:18-buster
  RUN npm i -g retypeapp

image:
  ARG toolchain
  ARG version
  ARG tag
  FROM DOCKERFILE \
    -f ./spoderman/docker/Dockerfile \
    --build-arg toolchain=$toolchain \
    --build-arg version=$version \
    ./spoderman
  SAVE IMAGE $tag

fmt:
  FROM +spoderman
  RUN cargo fmt --all -- --check

test:
  FROM +spoderman
  ARG features
  RUN cargo test $features

docs:
  FROM +spoderman
  RUN cargo doc --document-private-items --all-features
  SAVE ARTIFACT ./target/doc AS LOCAL ./.artifacts/docs

wiki:
  FROM +retype
  WORKDIR /app
  COPY ./docs/* .
  RUN ls -alghR
  RUN retype build
  SAVE ARTIFACT .retype AS LOCAL ./.artifacts/wiki
