FROM rust:1.81.0-bookworm

# instal solana CLI
RUN sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
ENV PATH="/root/.local/share/solana/install/active_release/bin:$PATH"

# instal tools for solana program development
RUN apt update && apt install -y \
        protobuf-compiler \ 
        libudev-dev
