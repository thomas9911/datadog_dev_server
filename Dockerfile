FROM ekidd/rust-musl-builder:stable as builder

RUN USER=root cargo new --bin datadog_dev_server
WORKDIR ./datadog_dev_server
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src
RUN rm ./target/x86_64-unknown-linux-musl/release/deps/datadog_dev_server*
RUN cargo build --release


FROM alpine:latest

ARG APP=/usr/src/app

EXPOSE 8125

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN addgroup -S $APP_USER \
    && adduser -S -g $APP_USER $APP_USER

RUN apk update \
    && apk add --no-cache ca-certificates tzdata \
    && rm -rf /var/cache/apk/*

COPY --from=builder /home/rust/src/datadog_dev_server/target/x86_64-unknown-linux-musl/release/datadog_dev_server ${APP}/datadog_dev_server

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./datadog_dev_server"]