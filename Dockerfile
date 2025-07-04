FROM rust:1.84 AS base
WORKDIR /app
ADD Cargo.toml Cargo.lock /app/
RUN mkdir -p /app/src &&\
    echo "fn main() {}" > /app/src/main.rs &&\
    cargo build --release

FROM base AS builder
ADD . /app
RUN touch src/main.rs &&\
    cargo build --release

FROM debian:12-slim
# ARG ENV_FILE
WORKDIR /app
RUN apt update && \
    apt install -y ca-certificates 
COPY --from=builder /app/data /app/data
COPY --from=builder /app/target/release/maxfun-evt /app
# COPY --from=builder /app/${ENV_FILE} /app/.env
CMD ["/app/maxfun-evt"]