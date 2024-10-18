#!/bin/bash
set -e # return immediately if any command fails
cargo build-sbf
# If a program id is not provided, the program will be 
# deployed to the default address at `<PROGRAM_NAME>-keypair.json`,
# This default keypair is generated during the first program compilation
# in the same directory as the program's shared object (.so).
# When using different program keys: https://docs.solanalabs.com/cli/examples/deploy-a-program
solana program deploy ./target/deploy/file_event_tracker.so
solana program show --programs
