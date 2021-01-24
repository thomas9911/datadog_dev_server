FROM ekidd/rust-musl-builder:stable as builder

RUN USER=root cargo new --bin upd_tester
WORKDIR ./upd_tester
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src
RUN rm ./target/x86_64-unknown-linux-musl/release/deps/udp_tester*
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

COPY --from=builder /home/rust/src/upd_tester/target/x86_64-unknown-linux-musl/release/udp_tester ${APP}/udp_tester

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./udp_tester"]