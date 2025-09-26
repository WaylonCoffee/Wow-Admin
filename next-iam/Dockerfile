# syntax=docker/dockerfile:1.6

############################
# 1) 依赖配方阶段（cargo-chef prepare）
# - 只复制工作区的 Cargo 清单，生成“依赖烘焙配方”
# - 目的是把“依赖编译”与“源码编译”分层，最大化缓存命中
############################
FROM rust:1.88.0-slim-bookworm AS chef
WORKDIR /workspace

# 安装 cargo-chef（锁定版本便于复现）
RUN cargo install cargo-chef --locked

# 只复制 workspace 的 Cargo 清单（包含 root + 子 crate 清单）
COPY Cargo.toml Cargo.lock ./
COPY app/Cargo.toml app/Cargo.toml
COPY domain/Cargo.toml domain/Cargo.toml
COPY infra/Cargo.toml infra/Cargo.toml

# ✅ 为每个 member 放置最小占位源码，供 cargo metadata 推断目标
RUN mkdir -p app/src domain/src infra/src && \
    printf "fn main() {}\n" > app/src/main.rs && \
    printf "// lib stub\n" > domain/src/lib.rs && \
    printf "// lib stub\n" > infra/src/lib.rs

# 生成依赖配方
RUN cargo chef prepare --recipe-path recipe.json

############################
# 2) 依赖烘焙阶段（cargo-chef cook）
# - 预编译依赖，后续源码变化不会触发依赖重编
# - 挂载 BuildKit 缓存，加快 crates.io / git 索引与 target 缓存
############################
FROM rust:1.88.0-slim-bookworm AS deps
WORKDIR /workspace

RUN cargo install cargo-chef --locked

# 可选：如你使用 sqlx 宏且 CI/构建时不希望连库，启用离线检查
ENV SQLX_OFFLINE=true

COPY --from=chef /workspace/recipe.json recipe.json

# 使用 BuildKit 缓存挂载（需要 DOCKER_BUILDKIT=1）
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo chef cook --release --recipe-path recipe.json


############################
# 3) 构建阶段（编译应用）
# - 复制全部源代码并编译目标二进制
# - 默认编译 workspace 的 app 二进制（可通过 APP_BIN 覆盖）
############################
FROM rust:1.88.0-slim-bookworm AS builder
WORKDIR /workspace

# （可选）更快的链接器（mold）；如不需要可删除
RUN apt-get update && apt-get install -y --no-install-recommends clang mold pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 复制依赖层产物（target/registry/git 缓存已在上一阶段）
COPY --from=deps /usr/local/cargo /usr/local/cargo
COPY --from=deps /workspace/target /workspace/target

COPY .sqlx /workspace/.sqlx

# 复制全部源码（这一步改变才会触发后续编译层）
COPY . .

# 生产构建参数（可在外部覆盖）
ARG APP_BIN=app

# 若你使用 sqlx 宏且不希望构建期连库，保持离线
ENV SQLX_OFFLINE=true
RUN cargo build --release -p ${APP_BIN}

# 使用 mold 链接器（可选）
ENV RUSTFLAGS="-Clink-arg=-fuse-ld=mold"

# BuildKit 缓存挂载：显著加速二次构建
RUN --mount=type=cache,target=/workspace/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release -p ${APP_BIN}


############################
# 4) 运行阶段（生产镜像）
# - 选择 Debian slim 作为 runtime：带 CA 证书、时区数据、glibc 兼容好
# - 非 root 运行，提高安全性
############################
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# 基础运行依赖：证书/时区/SSL 库（如需）
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN useradd -m -u 10001 appuser
USER appuser

# 从 builder 拷贝目标二进制
ARG APP_BIN=app
COPY --from=builder /workspace/target/release/${APP_BIN} /app/${APP_BIN}

# 复制 workspace 根的 config/ 到容器（你的项目就是放在父目录）
COPY config /app/config

# 日志级别可由外部覆盖
ENV RUST_LOG=info

# 健康检查（可选；针对 /healthz）
# HEALTHCHECK --interval=30s --timeout=3s --start-period=10s \
#     CMD wget -qO- http://127.0.0.1:8080/healthz || exit 1

EXPOSE 8080

# 使用绝对路径更稳
ENTRYPOINT ["/app/app"]
