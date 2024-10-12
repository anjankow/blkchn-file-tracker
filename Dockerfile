FROM rust:1.81.0-slim-bookworm

# needed only to generate C bindings
# - already provided by inotify wrapper
# RUN apt update && apt install -y clang-15 make

