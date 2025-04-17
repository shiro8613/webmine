#https://note.com/minato_kame/n/n8b919b05bc25
#rustをbuildするコンテナ
FROM rust:latest as builder
WORKDIR /build
COPY . .
RUN cargo build --release

#buildしたバイナリをpushするコンテナ
FROM rust:latest
WORKDIR /app
COPY --from=builder /build/target/release/webmine .
CMD ["./webmine"]