FROM rust:1.84.1
WORKDIR /app
COPY . .
RUN cargo build --release
CMD ["./target/release/metadata"]