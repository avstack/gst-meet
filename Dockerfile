
FROM ekidd/rust-musl-builder:stable as builder
RUN USER=root cargo new --bin rust-webserver

COPY ./rust-webserver  ./rust-webserver

WORKDIR ./rust-webserver
RUN cargo build --release
RUN rm -r ./target/x86_64-unknown-linux-musl/release/deps
RUN cargo build --release

FROM docker.io/library/alpine:3.18.2 AS builder1

COPY ./streaming-service-bridge  ./streaming-service-bridge
WORKDIR ./streaming-service-bridge
RUN apk --no-cache add gstreamer-dev gst-plugins-base-dev 
RUN apk --no-cache add build-base openssl-dev cargo libnice-dev
RUN cargo build --release -p gst-meet

FROM docker.io/library/alpine:3.18.2
RUN apk update
RUN apk --no-cache add curl
RUN apk --no-cache add sed
RUN apk add --no-cache --upgrade bash
RUN apk --no-cache add jq
RUN apk --no-cache add unzip
RUN apk --no-cache add gstreamer gst-plugins-good gst-plugins-bad gst-plugins-ugly gst-libav libnice-gstreamer
RUN apk --no-cache add libnice openssl

ARG APP=/usr/src/app

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN addgroup -S $APP_USER \
    && adduser -S -g $APP_USER $APP_USER

RUN apk update \
    && apk add --no-cache ca-certificates tzdata \
    && rm -rf /var/cache/apk/*
COPY --from=builder1 /streaming-service-bridge/target/release/gst-meet  /usr/src/app/
COPY --from=builder /home/rust/src/rust-webserver/target/x86_64-unknown-linux-musl/release/rust-webserver ${APP}/rust-webserver
RUN chown -R $APP_USER:$APP_USER ${APP}
USER $APP_USER
WORKDIR ${APP}

EXPOSE 8080
CMD ["./rust-webserver"]
