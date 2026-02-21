# 移除 --platform=$BUILDPLATFORM，让 Docker 自动匹配目标架构镜像
FROM rust:1.75-alpine AS builder
WORKDIR /app

# 只需要安装最基础的编译依赖
RUN apk add --no-cache musl-dev gcc g++ make

# 复制项目文件
COPY Cargo.toml ./
COPY src ./src

# 直接编译。由于 QEMU 的存在，这行在 ARM64 容器里会自动产出 ARM64 二进制文件
RUN cargo build --release

# 最终镜像
FROM scratch
WORKDIR /app
# 使用通配符或固定路径，Cargo 会把产物放在 target/release 下
COPY --from=builder /app/target/release/mqtt-wol /mqtt-wol
ENTRYPOINT ["/mqtt-wol"]