FROM rust:1.80-slim as builder
WORKDIR /usr/src/app
COPY cargo.toml Cargo.lock ./
RUN mkdir SRC && echo "fn main() {}" > SRC/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/fde_rust_portfolio*
COPY SRC ./SRC
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /usr/src/app/target/release/fde-rust-portfolio ./server
COPY PUBLIC ./PUBLIC
ENV PORT=8080
EXPOSE 8080
CMD ["./server"]
