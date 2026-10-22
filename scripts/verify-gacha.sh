#!/bin/sh

set -eu

: "${DATABASE_URL:?Set DATABASE_URL to a migrated test PostgreSQL database}"

export SQLX_OFFLINE=true

cargo check --locked --all-targets
cargo test --locked
cargo clippy --locked --all-targets
git diff --check
