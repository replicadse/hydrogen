FROM archlinux:latest
USER root

RUN pacman -Syu --noconfirm
RUN pacman -S sudo git gcc make tmux --noconfirm
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly

USER gitpod
USER root
RUN echo "gitpod ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

USER gitpod
