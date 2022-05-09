FROM rustlang/rust:nightly as builder
ENV RUSTFLAGS="-C target-cpu=native"

WORKDIR /build

RUN apt-get update && apt-get install -y cmake && apt-get clean

# dummy build for dep caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo "// dummy file" > src/lib.rs && \
    cargo build --release && \
    rm -r src



COPY . .

RUN cargo build --release

FROM debian:buster-slim

RUN apt-get update && apt-get upgrade -y && apt-get install -y ffmpeg ca-certificates libopus-dev  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/urusai /usr/local/bin/urusai

COPY Cargo.lock .

CMD ["/usr/local/bin/urusai"]