FROM archlinux:latest
USER root

RUN pacman -Syu --no-confirm
RUN pacman -S git gcc make tmux --no-confirm
RUN RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly

USER gitpod
