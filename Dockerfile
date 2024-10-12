FROM rust:1.81.0-slim-bookworm

RUN apt update && apt install -y clang-15 make