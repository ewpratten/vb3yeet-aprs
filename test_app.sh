#! /bin/bash
set -ex

# Set up the environment
export CONFIG_FILE=configs/config.toml
export ROCKET_CLI_COLORS=false
export ROCKET_PORT=8080

# Run the app
cargo run
