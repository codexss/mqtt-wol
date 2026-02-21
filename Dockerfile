FROM --platform=$BUILDPLATFORM rust:1.75-alpine AS builder
WORKDIR /app
RUN apk add --no-cache musl-dev

COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

COPY src ./src
RUN cargo build --release && \
    cp target/release/mqtt-wol /mqtt-wol

FROM scratch
COPY --from=builder /mqtt-wol /mqtt-wol
ENTRYPOINT ["/mqtt-wol"]