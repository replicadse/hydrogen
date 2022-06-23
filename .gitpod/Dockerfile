FROM archlinux:latest
USER root

RUN pacman -Syu --noconfirm
RUN pacman -S sudo git gcc make tmux zsh --noconfirm

USER gitpod
USER root
RUN echo "gitpod ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers
RUN chsh -s /usr/bin/zsh gitpod

USER gitpod
WORKDIR /home/gitpod
RUN git clone https://github.com/replicadse/dotfiles
