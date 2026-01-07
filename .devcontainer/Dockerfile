FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
ENV PATH=/usr/local/cargo/bin:$PATH

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    git \
    build-essential \
    pkg-config \
    clang \
    lld \
    cmake \
    mold \
    gdb \
    lldb \
    strace \
    lsof \
    linux-tools-common \
    linux-tools-generic \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev \
    libxkbcommon-dev \
    libssl-dev \
    zlib1g-dev \
    libzstd-dev \
    liblz4-dev \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /usr/local/rustup /usr/local/cargo \
    && curl -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal --default-toolchain 1.93.0 \
    && rustup component add clippy rustfmt

ARG USERNAME=dev
ARG USER_UID=1000
ARG USER_GID=$USER_UID

RUN set -e; \
    if getent group "$USER_GID" >/dev/null; then \
        GROUP_NAME="$(getent group "$USER_GID" | cut -d: -f1)"; \
        if [ "$GROUP_NAME" != "$USERNAME" ]; then \
            groupmod -n "$USERNAME" "$GROUP_NAME"; \
            GROUP_NAME="$USERNAME"; \
        fi; \
    else \
        groupadd --gid "$USER_GID" "$USERNAME"; \
        GROUP_NAME="$USERNAME"; \
    fi; \
    if getent passwd "$USER_UID" >/dev/null; then \
        EXISTING_USER="$(getent passwd "$USER_UID" | cut -d: -f1)"; \
        if [ "$EXISTING_USER" != "$USERNAME" ]; then \
            usermod -l "$USERNAME" "$EXISTING_USER"; \
            usermod -d "/home/$USERNAME" -m "$USERNAME"; \
        fi; \
    else \
        useradd --uid "$USER_UID" --gid "$USER_GID" -m "$USERNAME"; \
    fi; \
    usermod -g "$USER_GID" "$USERNAME"; \
    chown -R "$USER_UID":"$GROUP_NAME" /usr/local/cargo /usr/local/rustup

USER $USERNAME
WORKDIR /workspaces/muster-hub
