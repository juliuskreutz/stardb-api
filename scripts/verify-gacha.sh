#!/bin/sh

set -eu

export SQLX_OFFLINE=true

cargo check --all-targets
cargo test banner_helpers -- --nocapture
cargo test pool_ids -- --nocapture
cargo clippy --all-targets
git diff --check
