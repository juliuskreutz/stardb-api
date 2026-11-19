#!/bin/sh
# Manage a disposable test database without loading the application's .env.
set -eu

repo_root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"
export DATABASE_URL="postgres://postgres:stardb-test@127.0.0.1:${STARDB_TEST_DB_PORT:-55432}/stardb_test"

compose() {
    docker compose --env-file /dev/null -p stardb-api-test -f compose.test.yml "$@"
}

# SQLx cache formats follow the library version; reject an older globally installed CLI.
sqlx_cli() {
    case "$(cargo sqlx --version)" in
        *" 0.9.0") ;;
        *) printf '%s\n' 'Install SQLx CLI 0.9.0 using the command in README.md.' >&2; return 1 ;;
    esac
    cargo sqlx --no-dotenv "$@"
}

case "${1:-help}" in
    start)
        compose up -d --wait
        ;;
    migrate)
        compose up -d --wait
        sqlx_cli migrate run
        ;;
    test)
        shift
        compose up -d --wait
        SQLX_OFFLINE=true cargo test --locked "$@"
        ;;
    verify)
        compose up -d --wait
        scripts/verify-gacha.sh
        ;;
    prepare)
        compose up -d --wait
        sqlx_cli migrate run
        SQLX_OFFLINE=false sqlx_cli prepare -- --all-targets --locked
        ;;
    check-schema)
        compose up -d --wait
        sqlx_cli migrate run
        SQLX_OFFLINE=false sqlx_cli prepare --check -- --all-targets --locked
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
