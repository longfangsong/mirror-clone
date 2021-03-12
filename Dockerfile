FROM rust AS builder
RUN apt update && apt install -y libssl-dev ca-certificates openssl
# If we are lucky enough, these will build the dependencies and cache the git layer
COPY ./Cargo.toml /mirror-clone/
RUN mkdir /mirror-clone/src && echo "fn main() {}" > /mirror-clone/src/main.rs
WORKDIR /mirror-clone
RUN cargo build --release
# Do the real build
COPY ./* /mirror-clone/
RUN cargo +nightly fmt --check
RUN cargo +nightly test --all-features --workspace
