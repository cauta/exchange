# Use the latest version of Ubuntu as the base for our environment.
FROM ubuntu:24.04

# Avoids prompts from package installers.
ARG DEBIAN_FRONTEND=noninteractive

# Update packages and install build tools needed for compiling Rust crates.
RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    pkg-config \
    curl \
    git \
    docker.io \
    wget \
    ca-certificates \
    gnupg \
    lsb-release \
    unzip \
    postgresql-client \
    python3 \
    python3-pip \
    zsh \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID
RUN groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME -s /bin/zsh \
    && apt-get update \
    && apt-get install -y sudo \
    && echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME \
    && chmod 0440 /etc/sudoers.d/$USERNAME \
    && rm -rf /var/lib/apt/lists/*

# Set the Rustup and Cargo home directories.
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

# Install rustup and the default stable Rust toolchain.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.90 \
    && chmod -R a+w $RUSTUP_HOME $CARGO_HOME

# Install Rust tools
RUN cargo install just sqlx-cli --no-default-features --features rustls,postgres

# Install Node.js 22
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Install Bun
RUN curl -fsSL https://bun.sh/install | bash \
    && mv /root/.bun /usr/local/bun \
    && ln -s /usr/local/bun/bin/bun /usr/local/bin/bun

# Install ClickHouse CLI
RUN curl https://clickhouse.com/ | sh \
    && mv ./clickhouse /usr/local/bin/clickhouse \
    && chmod +x /usr/local/bin/clickhouse

# Install pgcli (better PostgreSQL CLI)
RUN pip3 install --break-system-packages pgcli

# Create directories for volume mounts
RUN mkdir -p /var/lib/docker /usr/local/cargo/registry /usr/local/rustup

# Set up user permissions
RUN chown -R $USERNAME:$USERNAME /usr/local/cargo /usr/local/rustup

# Switch to non-root user
USER $USERNAME

# Set the working directory inside the container.
WORKDIR /workspace

# Keep the container alive for development.
CMD ["sleep", "infinity"]
