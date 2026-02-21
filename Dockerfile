# 使用 BUILDPLATFORM 来让编译器跑在宿主机架构（通常是 x86）
FROM --platform=$BUILDPLATFORM rust:1.75-alpine AS builder
WORKDIR /app
RUN apk add --no-cache musl-dev

# 接收来自 Buildx 的目标架构参数
ARG TARGETARCH

COPY Cargo.toml ./
COPY src ./src

# 根据目标架构设置对应的编译目标
RUN if [ "$TARGETARCH" = "arm64" ]; then \
        rustup target add aarch64-unknown-linux-musl && \
        cargo build --release --target aarch64-unknown-linux-musl && \
        cp target/aarch64-unknown-linux-musl/release/mqtt-wol /mqtt-wol; \
    else \
        rustup target add x86_64-unknown-linux-musl && \
        cargo build --release --target x86_64-unknown-linux-musl && \
        cp target/x86_64-unknown-linux-musl/release/mqtt-wol /mqtt-wol; \
    fi

# 最终镜像
FROM scratch
COPY --from=builder /mqtt-wol /mqtt-wol
ENTRYPOINT ["/mqtt-wol"]