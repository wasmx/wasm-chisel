FROM rust:1.34.0-stretch

COPY . /chisel/

RUN cd /chisel && cargo build --release

ENTRYPOINT ["/chisel/target/release/chisel"]
