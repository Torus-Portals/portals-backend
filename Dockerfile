FROM clux/muslrust:1.55.0 as builder
WORKDIR /build
COPY . .

# run unit tests
RUN cargo test
RUN cargo test -- --ignored

# build prod binary
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN strip target/x86_64-unknown-linux-musl/release/portals-backend

# init
FROM scratch
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/portals-backend /
EXPOSE 8088
EXPOSE 443
ENTRYPOINT ["/portals-backend"]
