FROM rustembedded/cross:x86_64-unknown-linux-gnu

RUN apt-get update && \
    apt-get install --assume-yes pkg-config libssl-dev
