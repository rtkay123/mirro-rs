FROM alpine:3.17.0

ENV RUSTFLAGS="-C target-feature=-crt-static"

RUN apk add --no-cache openssl-dev gcc musl-dev rustup

RUN rustup-init -t x86_64-unknown-linux-musl --default-toolchain nightly --profile minimal -y

WORKDIR /usr/src/app

COPY . .

RUN /root/.cargo/bin/cargo build --release --all-features

FROM alpine:3.17.0

RUN apk add --no-cache libgcc

COPY --from=0 /usr/src/app/target/release/mirro-rs /bin/

ENTRYPOINT ["mirro-rs"]
