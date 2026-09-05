#!/bin/sh
# Manage a disposable test database without loading the application's .env.
set -eu

repo_root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"
export DATABASE_URL="postgres://postgres:stardb-test@127.0.0.1:${STARDB_TEST_DB_PORT:-55432}/stardb_test"

compose() {
    docker compose --env-file /dev/null -p stardb-api-test -f compose.test.yml "$@"
}

case "${1:-help}" in
    start)
        compose up -d --wait
        ;;
    migrate)
        compose up -d --wait
        cargo sqlx migrate run
        ;;
    test)
        shift
        compose up -d --wait
        SQLX_OFFLINE=true cargo test "$@"
        ;;
    verify)
        compose up -d --wait
        scripts/verify-gacha.sh
        ;;
    prepare)
        compose up -d --wait
        cargo sqlx migrate run
        SQLX_OFFLINE=false cargo sqlx prepare -- --all-targets
        ;;
    check-schema)
        compose up -d --wait
        cargo sqlx migrate run
        SQLX_OFFLINE=false cargo sqlx prepare --check -- --all-targets
        ;;
    stop)
        compose stop
        ;;
    reset)
        compose down --volumes
        compose up -d --wait
        ;;
    url)
        printf '%s\n' "$DATABASE_URL"
        ;;
    *)
        printf '%s\n' 'Usage: scripts/test-db.sh {start|migrate|test [cargo test arguments]|verify|prepare|check-schema|stop|reset|url}'
        ;;
esac
