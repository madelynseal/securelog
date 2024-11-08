FROM rust:bullseye as builder
WORKDIR /usr/src/securelog-server/
COPY . .
RUN cargo install --path .

FROM debian:bullseye
COPY --from=builder /usr/local/cargo/bin/securelog-server /usr/local/bin/securelog-server
VOLUME /var/log/securelog-server/
VOLUME /etc/securelog/

CMD ["securelog-server"]
