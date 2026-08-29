# syntax=docker/dockerfile:1
FROM node:22-alpine AS web-builder
RUN corepack enable && corepack prepare pnpm@latest --activate
WORKDIR /app/web
COPY web/package.json web/pnpm-lock.yaml ./
RUN --mount=type=cache,id=pnpm,target=/root/.local/share/pnpm/store \
    pnpm install --frozen-lockfile
COPY web/ ./
RUN pnpm run build

FROM rust:alpine AS server-builder
RUN apk add --no-cache musl-dev gcc
WORKDIR /app
COPY . .
COPY --from=web-builder /app/web/dist /app/web/dist
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release -p tnotes-server && \
    cp /app/target/release/tnotes-server /app/tnotes-server

FROM alpine:3.21
RUN apk add --no-cache ca-certificates tzdata && \
    adduser -D -u 1000 notat && \
    mkdir -p /data && \
    chown -R notat:notat /data

COPY --from=server-builder /app/tnotes-server /usr/local/bin/tnotes-server

USER notat
WORKDIR /data
EXPOSE 8787

ENV NOTAT_DATA_DIR=/data
ENV NOTAT_HOST=0.0.0.0
ENV NOTAT_PORT=8787

CMD ["tnotes-server"]
