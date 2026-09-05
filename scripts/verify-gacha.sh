#!/bin/sh

set -eu

: "${DATABASE_URL:?Set DATABASE_URL to a migrated test PostgreSQL database}"

export SQLX_OFFLINE=true

cargo check --all-targets
cargo test
cargo clippy --all-targets
git diff --check
