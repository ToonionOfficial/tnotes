# Build stage
FROM rust:1.80-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY . .
RUN cargo build --release -p notat-server

# Runtime stage
FROM alpine:3.20
RUN adduser -D notat
COPY --from=builder /app/target/release/notat-server /usr/local/bin/
USER notat
EXPOSE 8787
CMD ["notat-server"]
