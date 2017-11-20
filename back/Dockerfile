FROM debian:stretch

RUN apt-get update && \
    apt-get install --no-install-recommends -y libc6 && \
    rm -rf /var/lib/apt/lists/*

VOLUME ["/data/task"]

WORKDIR /data

ENV RUST_BACKTRACE 1
ENV ROCKET_ENV production

COPY ./target/release/abc /usr/bin/

ENTRYPOINT ["/usr/bin/abc", "server"]
