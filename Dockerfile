FROM rust:1.83-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release

FROM alpine:3.21
RUN adduser -D -h /burrow burrow
WORKDIR /burrow
COPY --from=builder /app/target/release/burrowd /usr/local/bin/burrowd
COPY --from=builder /app/target/release/burrow /usr/local/bin/burrow
USER burrow
EXPOSE 7070
CMD ["burrowd"]
