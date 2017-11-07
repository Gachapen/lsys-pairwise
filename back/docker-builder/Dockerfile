FROM debian:stretch

RUN apt-get update && \
    apt-get install --no-install-recommends --no-install-suggests -y curl ca-certificates gcc libc6-dev libssl-dev pkg-config && \
    rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain nightly -y

ENV PATH=/root/.cargo/bin:$PATH

VOLUME /src /root/.cargo/registry /root/.cargo/git

WORKDIR /src
