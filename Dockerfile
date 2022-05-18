FROM rust:alpine AS builder
COPY . .
RUN cargo install --path .

FROM alpine AS runner
COPY --from=builder /usr/local/cargo/bin/fivem-utility /usr/local/bin/fivem-utility
ENTRYPOINT [ "/usr/local/bin/fivem-utility" ]
