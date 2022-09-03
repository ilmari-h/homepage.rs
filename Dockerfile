FROM rustlang/rust:nightly

WORKDIR /usr/src/myapp

COPY . .

RUN cargo build --release

CMD ["./target/release/homepage_rs"]
