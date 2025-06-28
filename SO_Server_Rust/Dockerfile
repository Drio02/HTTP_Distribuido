FROM rust:latest as builder

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release

FROM debian:latest

COPY --from=builder /usr/src/app/target/release/so_server_rust /usr/local/bin/worker-server

EXPOSE 7878

CMD ["worker-server"]