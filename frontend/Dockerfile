FROM rust:latest

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk

RUN curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64
RUN mv tailwindcss-linux-x64 tailwindcss
RUN chmod +x tailwindcss
RUN ./tailwindcss

ADD index.html tailwind.config.js Trunk.toml Cargo.toml Cargo.lock public/ /
ADD src/ src/

RUN trunk build --release