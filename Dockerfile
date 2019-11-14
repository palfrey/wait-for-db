FROM rust:1.38.0-alpine3.10 as builder

RUN apk add --no-cache git abuild musl-dev make file
ADD abuild-fetch.c /abuild-fetch.c
RUN gcc abuild-fetch.c -o /usr/bin/abuild-fetch
RUN abuild-fetch https://ftp.gnu.org/gnu/libtool/libtool-2.4.6.tar.gz
RUN cd / && git clone -b static-builds https://github.com/palfrey/aports.git
RUN abuild-keygen -ain
RUN cd /aports/main/unixodbc/ && abuild -F fetch unpack build rootpkg
RUN apk add --no-cache bash m4 help2man
RUN cd /aports/main/libtool/ && abuild -F fetch unpack build rootpkg
RUN apk index -o ~/packages/main/x86_64/APKINDEX.tar.gz ~/packages/main/x86_64/*.apk
RUN apk add --allow-untrusted --no-cache --repository ~/packages/main/ unixodbc-static unixodbc-dev libltdl-static

# FIXME: Revert back to this once the following PRs are merged
# * https://github.com/alpinelinux/aports/pull/12004 (creates unixodbc-static)
# * https://github.com/alpinelinux/aports/pull/12083 (creates libltdl-static)
# RUN apk add --no-cache unixodbc-dev unixodbc-static libltdl-static

WORKDIR /app
ADD . ./
RUN cargo build --release --target=x86_64-unknown-linux-musl
RUN strip ./target/x86_64-unknown-linux-musl/release/wait_for_db
RUN file ./target/x86_64-unknown-linux-musl/release/wait_for_db
RUN ls -lh ./target/x86_64-unknown-linux-musl/release/wait_for_db

FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/wait_for_db /wait_for_db
ENTRYPOINT ["/wait_for_db"]