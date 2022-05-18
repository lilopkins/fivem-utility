FROM rust:slim AS builder
RUN apt update && apt install -y \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo install --path .

FROM debian:stable-slim AS runner
COPY --from=builder /usr/local/cargo/bin/fivem-utility /usr/local/bin/fivem-utility
ENTRYPOINT [ "/usr/local/bin/fivem-utility" ]
