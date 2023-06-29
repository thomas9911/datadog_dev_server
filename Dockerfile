FROM rust:1.70.0-alpine3.18 as builder

WORKDIR /home/rust
RUN apk add musl-dev
RUN USER=root cargo new --bin datadog_dev_server
WORKDIR /home/rust/datadog_dev_server
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --all-features --release
RUN rm src/*.rs

COPY ./src ./src
RUN rm ./target/release/deps/datadog_dev_server*
RUN cargo build --all-features --release

FROM alpine:3.18

ARG APP=/usr/src/app

EXPOSE 8125

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN addgroup -S $APP_USER \
    && adduser -S -g $APP_USER $APP_USER

RUN apk update \
    && apk add --no-cache ca-certificates tzdata \
    && rm -rf /var/cache/apk/*

COPY --from=builder /home/rust/datadog_dev_server/target/release/datadog_dev_server ${APP}/datadog_dev_server

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./datadog_dev_server"]