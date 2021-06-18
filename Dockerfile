FROM alpine:3.13 as builder

RUN apk add --no-cache rustup gcc file unixodbc-dev unixodbc-static libltdl-static musl-dev openssl-dev openssl-libs-static
RUN rustup-init -y --default-host x86_64-unknown-linux-musl --profile minimal
ENV PATH=$PATH:/root/.cargo/bin

WORKDIR /app
ENV OPENSSL_STATIC=yes
ENV OPENSSL_LIB_DIR=/usr/lib
ENV OPENSSL_INCLUDE_DIR=/usr/include
ADD . ./
RUN cargo build --release --target=x86_64-unknown-linux-musl
RUN strip ./target/x86_64-unknown-linux-musl/release/wait_for_db
RUN file ./target/x86_64-unknown-linux-musl/release/wait_for_db
RUN ls -lh ./target/x86_64-unknown-linux-musl/release/wait_for_db

FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/wait_for_db /wait_for_db
ENTRYPOINT ["/wait_for_db"]
