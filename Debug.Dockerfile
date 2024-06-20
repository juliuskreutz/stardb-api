FROM rust:1.79.0 as builder
WORKDIR /usr/src/stardb-api
COPY . .
RUN cargo install --path .
FROM ubuntu:jammy
COPY --from=builder /usr/local/cargo/bin/stardb-api /usr/local/bin/stardb-api
CMD ["stardb-api"]
