# The API for [StarDB](https://stardb.gg)

[![wakatime](https://wakatime.com/badge/user/594e5948-2f7c-499b-955f-f57d579011fe/project/191bdf63-e366-4df1-8e42-10a802d4e404.svg?style=for-the-badge)](https://wakatime.com/badge/user/594e5948-2f7c-499b-955f-f57d579011fe/project/191bdf63-e366-4df1-8e42-10a802d4e404)

You can read API specifications at https://stardb.gg/api/swagger-ui/

![Database](./graph.svg)

## Developing

You need:

- [direnv](https://direnv.net)
- [devenv](https://devenv.sh)

```sh
cd stardb-api
devenv up
```

And you're ready to go :D

## Testing with Docker

Install Docker, Rust 1.94 or newer with Clippy, and SQLx CLI matching the library:

```sh
cargo install sqlx-cli --version 0.9.0 --no-default-features --features postgres,rustls
scripts/test-db.sh start
scripts/test-db.sh migrate
scripts/test-db.sh test
```

`compose.test.yml` provides PostgreSQL 17 on localhost port 55432. The script uses
its own database URL and disables Compose's automatic `.env` loading; it does not
use the application's database. Override the port with `STARDB_TEST_DB_PORT` if
55432 is occupied. Test data lives in the `stardb-api-test` project's Docker volume.
DB-backed tests apply migrations before their fixtures and use distinct fixture IDs.

```sh
scripts/test-db.sh test gacha::          # Focused tests; forwards cargo test arguments
scripts/test-db.sh verify               # All-target check, full tests, Clippy, diff check
scripts/test-db.sh prepare              # Migrate and refresh .sqlx after SQL/schema changes
scripts/test-db.sh check-schema         # Verify .sqlx matches the migrated schema
scripts/test-db.sh stop                 # Stop PostgreSQL, retain test data
scripts/test-db.sh reset                # Delete ONLY this Compose project's test volume and restart
scripts/test-db.sh url                  # Print the isolated test database URL
```

After `reset`, run `migrate` before SQL tooling; DB-backed tests migrate themselves.
Commit `.sqlx` changes together with their queries. Clippy CI type-checks and lints
all targets offline in a single pass; database tests and SQLx schema verification
run locally with these Docker commands. To run checks against a different test database, set
`DATABASE_URL` explicitly and run `scripts/verify-gacha.sh`; tests write fixture data.

## Faster development builds

Use `SQLX_OFFLINE=true cargo check --locked --all-targets` for compiler feedback
without linking an executable. Run focused tests while iterating, then
`scripts/test-db.sh verify` before committing behavior changes. CI runs Clippy once
(it also type-checks) and caches dependencies; it never starts a database or runs tests.

The development and test profiles use line-table debug information, preserving
file/line backtraces with less code-generation and linking work. For debugger locals
and full type information, use the opt-in profile:

```sh
SQLX_OFFLINE=true cargo build --locked --profile debugging
```

Only the PNG decoder used by the asset updater is enabled in `image`; WebP encoding
uses the separate `webp` crate. Enabling extra image formats also adds their codec
dependencies to builds, so add features only when a caller needs them.

Keep `target/` between runs and avoid routine `cargo clean`. Toolchain, dependency,
feature, profile, and `RUSTFLAGS` changes can cause recompilation. Run Cargo commands
sequentially when worktrees share a target directory; concurrent builds contend for
Cargo's lock. New worktrees use their own target directory by default.

To identify the actual bottleneck on your machine, capture Cargo's timing report:

```sh
SQLX_OFFLINE=true cargo build --locked --timings
```

Open `target/cargo-timings/cargo-timing.html`. Compare the same command and cache
state when measuring improvements; a warm check and a cold build are different
workloads. These choices follow the [Cargo build-performance guide](https://doc.rust-lang.org/cargo/guide/build-performance.html).

## HSR image storage

The StarRailRes updater keeps `static/StarRailResWebp` and the successful-revision
marker `static/.StarRailResWebp-revision`. Every ten minutes it checks upstream HEAD. When the
revision changes, it uses the existing shallow Git clone and PNG-to-WebP conversion
flow, then deletes the source checkout and its Git objects. Original
`/static/StarRailRes` PNG URLs are retired; clients use `/static/StarRailResWebp`.

This favors simple maintenance over minimizing downloads: every upstream revision
requires a full clone and conversion pass. The source checkout is temporary, so
allow room for it during a refresh; it is removed afterward. WebP paths, lossless
encoding, and 128×128 character-icon resizing are preserved. The revision marker
is invalidated before publishing changed outputs, so interrupted conversions retry
even if upstream reverts. Unchanged revisions skip conversion; remove the revision
marker to force a rebuild when individual outputs have been manually removed.

## Dependency upgrades

`Cargo.toml` declares the minimum Rust version; use `Cargo.lock` and `--locked` for
reproducible builds. SQLx CLI must match the library (currently 0.9.0); regenerate
and check `.sqlx` after upgrading SQLx, even when SQL text is unchanged.

Reqwest 0.13 uses the operating system's certificate trust store. Deployment hosts
must provide current CA certificates for outbound HTTPS (imports, assets, and
telemetry), including when running the statically linked musl binary.

## Code documentation

Use Rustdoc's `//!` comments to explain a module's responsibility and `///` comments
for production types, functions, and methods. Begin with a concise contract, then
explain relevant input assumptions, return values, side effects, and error behavior.
Use `# Errors` or `# Panics` when callers need to handle a non-obvious failure mode.
Keep ordinary `//` comments near logic whose ordering or invariant matters, such as
transaction boundaries, import cutoffs, provenance rules, or game-specific pity resets.

Generate searchable documentation, including internal helpers, without a database:

```sh
SQLX_OFFLINE=true cargo doc --no-deps --document-private-items --open
```

Keep contracts accurate when implementation changes. Document why an invariant or
special case exists rather than narrating each line; test names and assertions
usually explain routine test helpers without additional Rustdoc.
