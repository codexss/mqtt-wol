FROM --platform=$BUILDPLATFORM rust:1.75-alpine AS builder
WORKDIR /app

# 安装 musl 编译所需的工具
RUN apk add --no-cache musl-dev

# 直接复制项目文件
COPY Cargo.toml ./
COPY src ./src

# 编译生成二进制文件
# Alpine 默认会尝试静态链接，配合 Profile 优化实现最小体积
RUN cargo build --release && \
    cp target/release/mqtt-wol /mqtt-wol

# 最终运行镜像
FROM scratch
WORKDIR /app
COPY --from=builder /mqtt-wol /mqtt-wol
ENTRYPOINT ["/mqtt-wol"]