FROM docker.io/rust:1.57-alpine3.14 AS builder
RUN apk --update --no-cache add build-base pkgconf glib-dev gstreamer-dev libnice-dev openssl-dev
COPY . .
RUN cargo build --release -p gst-meet

FROM docker.io/alpine:3.14
RUN apk --update --no-cache add glib gstreamer libnice libc6-compat
COPY --from=builder target/release/gst-meet /usr/local/bin
ENTRYPOINT ["/usr/local/bin/gst-meet"]