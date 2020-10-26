# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.134.1/containers/rust/.devcontainer/base.Dockerfile
FROM mcr.microsoft.com/vscode/devcontainers/rust:0-1

# Install packages
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends gnupg2 linux-perf linux-base valgrind python-pip gnuplot-nox

RUN rm -rf /home/vscode/.gnupg
RUN rm -rf /root/.gnupg

# Set proper locale
RUN echo "en_US.UTF-8 UTF-8" > /etc/locale.gen && /usr/sbin/locale-gen

RUN mkdir /cargo-target
RUN chown -R vscode /cargo-target

# Set a proper shell as default
RUN chsh vscode -s /usr/bin/zsh

USER vscode

# Install oh-my-zsh
ENV TERM xterm
ENV SHELL /usr/bin/zsh
RUN wget https://github.com/robbyrussell/oh-my-zsh/raw/master/tools/install.sh -O - | zsh || true

# Cargo performance hack
ENV CARGO_TARGET_DIR /cargo-target

# Misc tools
RUN cargo install cargo-tarpaulin
RUN cargo install flamegraph
