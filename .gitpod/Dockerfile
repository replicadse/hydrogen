FROM archlinux:latest
USER root

RUN pacman -Syu --noconfirm
RUN pacman -S git gcc make tmux --noconfirm
RUN RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly

USER gitpod
