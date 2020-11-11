FROM rust:1.47

RUN apt-get update && \
    apt-get install -y pkg-config libudev-dev

WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock build.rs .
COPY abi abi/
COPY src src/
COPY config/testnet.json config/testnet.json
COPY config/testnet.key config/testnet.key
COPY config/authority.json config/authority.json

RUN cargo build --release

CMD ["target/release/moebius-bridge", "--config", "config/testnet.json", "--eth-private-key", "config/testnet.key", "--authority-key", "config/authority.json"]
