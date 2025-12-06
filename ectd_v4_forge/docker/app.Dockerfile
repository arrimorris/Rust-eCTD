# Start with the official Rust image (using latest stable)
FROM rust:latest

# 1. Install System Dependencies (The "Missing Links")
# Tauri requires these specific libraries to compile on Linux
# Updated for modern Debian (Bookworm/Trixie): use 4.1 instead of 4.0
RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    pkg-config \
    javascriptcoregtk-4.1 \
    libsoup-3.0-dev

# 2. Install Node.js (Required for the React Frontend build)
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y nodejs

# 3. Install SQLx CLI (For database management inside the container)
# Removed due to dependency conflict in bleeding edge env.
# We rely on ectd_cli for migrations now.
# RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres

# 4. Setup Working Directory
WORKDIR /app

# 5. Environment Variables for Tauri
# We need to tell Tauri we are in a container/Linux environment
ENV WEBKIT_DISABLE_COMPOSITING_MODE=1

# 6. Default Command (Keep the container alive for interaction)
CMD ["tail", "-f", "/dev/null"]
