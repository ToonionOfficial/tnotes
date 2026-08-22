# Build stage
FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev gcc
WORKDIR /app
COPY . .
RUN cargo build --release -p notat-server

# Runtime stage
FROM alpine:3.21
RUN apk add --no-cache ca-certificates tzdata && \
    adduser -D -u 1000 notat && \
    mkdir -p /data && \
    chown -R notat:notat /data

COPY --from=builder /app/target/release/notat-server /usr/local/bin/notat-server

USER notat
WORKDIR /data
EXPOSE 8787

ENV NOTAT_DATA_DIR=/data
ENV NOTAT_HOST=0.0.0.0
ENV NOTAT_PORT=8787

CMD ["notat-server"]
