FROM rust:1.38.0-alpine3.10 as builder

WORKDIR /app
RUN apk add --no-cache unixodbc-dev
ADD . ./
RUN cargo build --release --target=x86_64-unknown-linux-musl