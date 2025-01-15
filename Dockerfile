# Build Stage
FROM rust:1.84-bookworm AS builder

WORKDIR /app
COPY . .

RUN \
  --mount=type=cache,target=/app/target/ \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  export SQLX_OFFLINE=true && \
  cargo build --release && \
  cp ./target/release/api /

# Run stage
FROM debian:bookworm AS final
RUN apt-get update && \
  apt-get install -y openssl ca-certificates && \
  update-ca-certificates

VOLUME ["/app/data"]
ENV DATABASE_URL=sqlite:///app/data/db.sqlite
ENV DATABASE_CREATE=true

WORKDIR /app
COPY --from=builder /api /app/api
EXPOSE 3333
ENTRYPOINT ["/app/api"]
