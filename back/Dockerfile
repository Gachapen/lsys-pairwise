FROM debian:stretch

RUN apt-get update && \
    apt-get install --no-install-recommends -y libc6 && \
    rm -rf /var/lib/apt/lists/*

VOLUME ["/data/task"]

WORKDIR /data

ENV RUST_BACKTRACE 1
ENV ROCKET_ENV production

COPY ./target/release/lsys-pairwise /usr/bin/

ENTRYPOINT ["/usr/bin/lsys-pairwise", "server"]
