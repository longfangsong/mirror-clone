FROM rust AS builder
RUN apt update && apt install -y libssl-dev ca-certificates openssl && \
 rustup toolchain install nightly && \
 rustup default nightly && \
 rustup component add rustfmt
# If we are lucky enough, these will build the dependencies and cache the docker layer
RUN mkdir -p /mirror-clone/src && echo "fn main() { panic!(\"should never run this\"); }" > /mirror-clone/src/main.rs
COPY Cargo.toml /mirror-clone/
COPY Cargo.lock /mirror-clone/
WORKDIR /mirror-clone
RUN ls
RUN cargo build --release
# Do the real build
COPY ./src /mirror-clone/
RUN cargo fmt -- --check
RUN cargo test --all-features --workspace
