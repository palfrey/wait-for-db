FROM alpine:edge as builder

RUN apk add --no-cache -X http://dl-cdn.alpinelinux.org/alpine/edge/testing rustup && rustup-init -y --default-host x86_64-unknown-linux-musl
RUN apk add --no-cache gcc file unixodbc-dev unixodbc-static libltdl-static
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
