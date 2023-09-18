FROM rust AS build
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY build.rs wrapper.h ./
COPY azure-iot-sdk-c ./azure-iot-sdk-c

RUN apt update && \
    apt install -y cmake build-essential curl libcurl4-openssl-dev libssl-dev \
    uuid-dev libclang-dev pkg-config

RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt update && \
  apt install --no-install-recommends -y curl openssl uuid ca-certificates && \
  apt clean

COPY --from=build /usr/local/cargo/bin/pnio_hart /usr/local/bin/pnio_hart

ENV RUST_LOG=info

ENTRYPOINT ["/usr/local/bin/pnio_hart"]
