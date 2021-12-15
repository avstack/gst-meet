FROM docker.io/library/alpine:edge AS builder
RUN apk --no-cache --update upgrade --ignore alpine-baselayout \
 && apk --no-cache add curl \
 && curl -fL https://apk.avstack.net/avstack.rsa.pub -o "/etc/apk/keys/$(basename $(curl -LI --silent -w '%{url_effective}' https://apk.avstack.net/avstack.rsa.pub | tail -n 1))" \
 && apk --no-cache -X https://apk.avstack.net/main add gstreamer-dev gst-plugins-base-dev \
 && apk --no-cache add build-base libnice-dev openssl-dev cargo
COPY . .
RUN cargo build --release -p gst-meet

FROM docker.io/library/alpine:edge
RUN apk --update --no-cache upgrade --ignore alpine-baselayout \
 && apk --no-cache add curl \
 && curl -fL https://apk.avstack.net/avstack.rsa.pub -o "/etc/apk/keys/$(basename $(curl -LI --silent -w '%{url_effective}' https://apk.avstack.net/avstack.rsa.pub | tail -n 1))" \
 && apk --no-cache -X https://apk.avstack.net/main -X https://apk.avstack.net/community add gstreamer gst-plugins-good gst-plugins-bad gst-plugins-ugly gst-libav \
 && apk --no-cache add libnice openssl
COPY --from=builder target/release/gst-meet /usr/local/bin
ENTRYPOINT ["/usr/local/bin/gst-meet"]