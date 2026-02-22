FROM rust:1.75-alpine AS builder
WORKDIR /app

RUN apk add --no-cache musl-dev gcc g++ make

COPY Cargo.toml ./
COPY src ./src

ENV RUSTFLAGS="-C target-feature=+crt-static"
RUN cargo build --release

FROM scratch
WORKDIR /app
COPY --from=builder /app/target/release/mqtt-wol /mqtt-wol
ENTRYPOINT ["/mqtt-wol"]