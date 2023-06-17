FROM docker.io/library/alpine:3.18.2 AS builder
RUN apk --no-cache --update upgrade --ignore alpine-baselayout \
 && apk --no-cache add build-base gstreamer-dev gst-plugins-base-dev libnice-dev openssl-dev cargo
COPY . .
RUN cargo build --release -p gst-meet

FROM docker.io/library/alpine:3.18.2
RUN apk --update --no-cache upgrade --ignore alpine-baselayout \
 && apk --no-cache add openssl gstreamer gst-plugins-good gst-plugins-bad gst-plugins-ugly gst-libav libnice libnice-gstreamer
COPY --from=builder target/release/gst-meet /usr/local/bin
ENTRYPOINT ["/usr/local/bin/gst-meet"]
