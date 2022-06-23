FROM archlinux:latest
USER root

RUN pacman -Syu --noconfirm
RUN pacman -S sudo git gcc make tmux --noconfirm
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly

USER gitpod
USER root
RUN usermod -aG wheel gitpod

USER gitpod
