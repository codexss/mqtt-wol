FROM --platform=$BUILDPLATFORM rust:1.75-alpine AS builder
WORKDIR /app
RUN apk add --no-cache musl-dev

COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release

COPY src ./src
# 触摸一下文件确保触发重新编译
RUN touch src/main.rs && cargo build --release && \
    cp target/release/mqtt-wol /mqtt-wol

FROM scratch
COPY --from=builder /mqtt-wol /mqtt-wol
ENTRYPOINT ["/mqtt-wol"]