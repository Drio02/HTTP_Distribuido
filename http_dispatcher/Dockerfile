FROM rust:latest as builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

FROM debian:latest

RUN apt-get update && apt-get install -y --no-install-recommends openssl ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/http_dispatcher /usr/local/bin/dispatcher-server

EXPOSE 8080

CMD ["dispatcher-server"]