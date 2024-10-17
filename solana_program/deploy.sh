#!/bin/bash
set -e # return immediately if any command fails
cargo build-sbf
solana program deploy ./target/deploy/file_event_tracker.so
solana program show --programs
