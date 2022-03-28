# 0. BUILD STAGE
FROM ekidd/rust-musl-builder:stable AS build
# only build deps in the first stage for faster builds
COPY Cargo.toml Cargo.lock ./
USER root
RUN cargo install cargo-build-deps
RUN cargo build-deps --release
RUN rm -f target/x86_64-unknown-linux-musl/release/deps/rust-web-server*
# build
COPY --chown=root:root src src
RUN cargo build --release --bin main
RUN strip target/x86_64-unknown-linux-musl/release/main
RUN useradd rust-web-server

# 1. APP STAGE
FROM scratch
WORKDIR /app
COPY --from=build /home/rust/src/target/x86_64-unknown-linux-musl/release/main ./rust-web-server
COPY --from=build /etc/passwd /etc/passwd
# permissions
#RUN addgroup -g 1000 rust-web-server
#RUN adduser -D -s /bin/sh -u 1000 -G rust-web-server rust-web-server
#RUN chown rust-web-server:rust-web-server rust-web-server
USER rust-web-server
STOPSIGNAL SIGKILL

# run it 
ENTRYPOINT ["./rust-web-server"]
