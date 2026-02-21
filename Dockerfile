FROM rust:1.75-alpine AS builder
WORKDIR /app
RUN apk add --no-cache musl-dev

COPY Cargo.toml ./
COPY src ./src

# 强制静态链接确保所有库都包含在二进制文件中
RUN RUSTFLAGS="-C target-feature=+crt-static" cargo build --release

FROM scratch
WORKDIR /app
# 仅复制优化后的二进制文件
COPY --from=builder /app/target/release/mqtt-wol /mqtt-wol
ENTRYPOINT ["/mqtt-wol"]