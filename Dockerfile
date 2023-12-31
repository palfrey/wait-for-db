FROM alpine:3.19 as builder
ARG HOST=x86_64-unknown-linux-musl

RUN apk add --no-cache rustup gcc file unixodbc-dev unixodbc-static libltdl-static musl-dev
RUN rustup-init -y --default-host $HOST --profile minimal
ENV PATH=$PATH:/root/.cargo/bin

WORKDIR /app
ADD . ./
RUN cargo build --release --target=$HOST
RUN file ./target/$HOST/release/wait_for_db
RUN ls -lh ./target/$HOST/release/wait_for_db

FROM scratch
ARG HOST=x86_64-unknown-linux-musl
COPY --from=builder /app/target/$HOST/release/wait_for_db /wait_for_db
ENTRYPOINT ["/wait_for_db"]
