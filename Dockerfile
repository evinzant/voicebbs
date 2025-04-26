# Build stage
FROM rust:1.81 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

WORKDIR /app
COPY --from=builder /app/target/release/voicebbs /app/voicebbs

EXPOSE 8080

CMD ["/app/voicebbs"]