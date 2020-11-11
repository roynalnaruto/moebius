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

CMD ["target/release/moebius-bridge", "--config", "/run/secrets/config", "--eth-private-key", "/run/secrets/privkey", "--authority-key", "/run/secrets/authority"]
