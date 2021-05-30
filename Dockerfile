# BUILD
FROM rust:1.52 as builder

WORKDIR /usr/src/torus-backend

COPY . .

RUN cargo build --release


# RUN
FROM debian:buster-slim

RUN apt-get update && apt-get -y install openssl postgresql-client-11 ca-certificates

WORKDIR /usr/local/bin/torus-backend

COPY --from=builder /usr/src/torus-backend/auth0.der ./
COPY --from=builder /usr/src/torus-backend/auth0_pkey.pem ./
COPY --from=builder /usr/src/torus-backend/cert.crt ./
COPY --from=builder /usr/src/torus-backend/target/release/torus-backend ./

EXPOSE 8088

EXPOSE 5432

CMD ["./torus-backend"]




# Currently working dockerfile (Don't touch):

# FROM rust:1.41

# RUN echo database $PORTALS_DATABASE

# Run echo host $PORTALS_MAIN_HOST

# WORKDIR /usr/src/torus-backend

# COPY . .

# RUN cargo build --release

# RUN cp ./target/release/torus-backend ./

# EXPOSE 8088

# EXPOSE 5432

# CMD ["./torus-backend"]