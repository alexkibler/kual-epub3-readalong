# Dockerfile for cross-compiling Immersion Reader
# This provides a complete build environment with all dependencies

FROM messense/rust-musl-cross:aarch64-musl

# Install additional dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /workspace

# Build command will be run by the user mounting the project
CMD ["bash"]
