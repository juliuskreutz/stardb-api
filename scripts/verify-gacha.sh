#!/bin/sh

set -eu

export SQLX_OFFLINE=true

cargo check --all-targets
cargo test banner_catalog -- --nocapture
cargo test six_pool_roundtrip -- --nocapture
cargo clippy --all-targets
git diff --check
