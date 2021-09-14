# BUILD
FROM rust:1.52 as builder

WORKDIR /usr/src/torus-backend

COPY . .

ENV SQLX_OFFLINE true

RUN cargo build --release --verbose


# RUN
FROM debian:buster-slim

RUN apt-get update && apt-get -y install openssl postgresql-client-11 ca-certificates

WORKDIR /usr/local/bin/torus-backend

COPY --from=builder /usr/src/torus-backend/target/release/torus-backend ./

EXPOSE 8088

EXPOSE 5432

CMD ["./torus-backend"]