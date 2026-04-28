# Stage 1: Generate dependency recipe
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app/backend

FROM chef AS planner
COPY backend/ /app/backend/
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Build dependencies (cached layer — only rebuilds when Cargo.toml changes)
FROM chef AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=planner /app/backend/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY backend/ /app/backend/
RUN cargo build --release

# Stage 3: Minimal runtime image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/backend/target/release/server ./server
ENV DATABASE_PATH=/data/tickets.sqlite3
ENV PORT=3000
EXPOSE 3000
CMD ["./server"]
