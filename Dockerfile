FROM alpine:3.11 as builder

RUN apk add --no-cache -X http://dl-cdn.alpinelinux.org/alpine/edge/community rustup gcc file unixodbc-dev unixodbc-static libltdl-static musl-dev
# FIXME: Need beta for https://github.com/rust-lang/rust/pull/69519 but can revert back once 1.44 is released
RUN rustup-init -y --default-host x86_64-unknown-linux-musl --default-toolchain beta --profile minimal
ENV PATH=$PATH:/root/.cargo/bin

WORKDIR /app
ADD . ./
RUN cargo build --release --target=x86_64-unknown-linux-musl
RUN strip ./target/x86_64-unknown-linux-musl/release/wait_for_db
RUN file ./target/x86_64-unknown-linux-musl/release/wait_for_db
RUN ls -lh ./target/x86_64-unknown-linux-musl/release/wait_for_db

FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/wait_for_db /wait_for_db
ENTRYPOINT ["/wait_for_db"]
