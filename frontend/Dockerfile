FROM rust:latest

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk

ADD index.html Trunk.toml Cargo.toml Cargo.lock public/ /
ADD src/ src/

RUN trunk build --release