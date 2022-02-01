FROM rust:1.58 as build

RUN USER=root cargo new --bin cfproxy
WORKDIR /cfproxy

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/cfproxy*
RUN cargo build --release

FROM rust:1.58

COPY --from=build ./cfproxy/target/release/cfproxy .

CMD ["./cfproxy"]
