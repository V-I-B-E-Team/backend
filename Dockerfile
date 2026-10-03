FROM rust:1 AS build

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release


FROM debian:bookworm-slim

COPY --from=build /app/target/release/backend /usr/local/bin/backend

EXPOSE 8000

CMD ["backend"]


FROM rust:1 AS debug

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build

EXPOSE 8000

CMD ["/app/target/debug/backend"]
