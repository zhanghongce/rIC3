FROM ubuntu:24.04
COPY ./target/release/rIC3 /usr/bin
WORKDIR /root
ENTRYPOINT ["rIC3", "model"]
