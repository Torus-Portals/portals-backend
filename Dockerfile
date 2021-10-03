FROM fedora:34 as builder
WORKDIR /build

# compile openssl for static linking
RUN dnf -y install gcc-c++ pkg-config musl-gcc git perl-core binaryen
RUN git clone git://git.openssl.org/openssl.git
RUN cd openssl && git checkout OpenSSL_1_1_1-stable
RUN cd openssl && ./config -fPIC no-weak-ssl-ciphers no-async --prefix=/usr/local/ssl --openssldir=/usr/local/ssl
RUN cd openssl && make && make install
ENV OPENSSL_STATIC true
ENV OPENSSL_DIR /usr/local/ssl

# setup rust tooling
RUN curl --proto '=https' --tlsv1.3 -O https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init
RUN chmod +x rustup-init
RUN ./rustup-init -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup target add x86_64-unknown-linux-musl

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
