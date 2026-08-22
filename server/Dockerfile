FROM node:22-alpine AS web-builder
RUN corepack enable && corepack prepare pnpm@latest --activate
WORKDIR /app/web
COPY web/package.json web/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY web/ ./
RUN pnpm run build

FROM rust:alpine AS server-builder
RUN apk add --no-cache musl-dev gcc
WORKDIR /app
COPY . .
COPY --from=web-builder /app/web/dist /app/web/dist
RUN cargo build --release -p tnotes-server

FROM alpine:3.21
RUN apk add --no-cache ca-certificates tzdata && \
    adduser -D -u 1000 notat && \
    mkdir -p /data && \
    chown -R notat:notat /data

COPY --from=server-builder /app/target/release/tnotes-server /usr/local/bin/tnotes-server

USER notat
WORKDIR /data
EXPOSE 8787

ENV NOTAT_DATA_DIR=/data
ENV NOTAT_HOST=0.0.0.0
ENV NOTAT_PORT=8787

CMD ["tnotes-server"]
