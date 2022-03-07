# gst-meet: Integrate Jitsi Meet conferences with GStreamer pipelines

Note: gst-meet is in an **alpha** state and is under active development. The command-line options and the lib-gst-meet API are subject to change.

gst-meet provides a library and tool for integrating Jitsi Meet conferences with GStreamer pipelines. You can pipe audio and video into a conference as a participant, and pipe out other participants' audio and video streams.

Thanks to GStreamer's flexibility and wide range of plugins, this enables many new possibilities.

## Dependencies

* `gstreamer` 1.20
* `gst-plugins-good`, `gst-plugins-bad` (same version as `gstreamer`) and any other plugins that you want to use in your pipelines
* `glib`
* `libnice`

### For building:

* `pkg-config`
* A Rust toolchain ([rustup](https://rustup.rs/) is the easiest way to install one)

## Installation

`cargo install --force gst-meet` (`--force` will upgrade `gst-meet` if you have already installed it.)

To integrate gst-meet into your own application, add a Cargo dependency on `lib-gst-meet`.

## Docker

A `Dockerfile` is provided that uses AVStack-built Alpine APKs for gstreamer 1.20.

## Nix

For nix users, a `shell.nix` is provided. Within the repository, run `nix-shell --pure` to get a shell with access to all needed dependencies (and nothing else).

## Development

Install the dependencies described above, along with their `-dev` packages if your distribution uses them. `cargo build` should then successfully build the libraries and `gst-meet` binary.

## Pipeline Structure

You can pass two different pipeline fragments to gst-meet.

`--send-pipeline` is for sending audio and video. If it contains an element named `audio`, this audio will be streamed to the conference. The audio codec must be 48kHz Opus. If it contains an element named `video`, this video will be streamed to the conference. The video codec must match the codec passed to `--video-codec`, which is VP8 by default.

`--recv-pipeline-participant-template` is for receiving audio and video from other participants. This pipeline will be created once for each other participant in the conference. If it contains an element named `audio`, the participant's audio (48kHz Opus) will be sent to that element. If it contains an element named `video`, the participant's video will be sent to that element. The strings `{jid}`, `{jid_user}`, `{participant_id}` and `{nick}` are replaced in the template with the participant's full JID, user part, MUC JID resource part (a.k.a. participant/occupant ID) and nickname respectively.

## Examples

A few examples of `gst-meet` usage are below. The GStreamer reference provides full details on available pipeline elements.

`gst-meet --help` lists full usage information.

Stream an Opus audio file to the conference. This is very efficient; the Opus data in the file is streamed directly without transcoding:

```
gst-meet --web-socket-url=wss://your.jitsi.domain/xmpp-websocket \
         --room-name=roomname \
         --send-pipeline="filesrc location=sample.opus ! queue ! oggdemux name=audio"
```

Stream a FLAC audio file to the conference, transcoding it to Opus:

```
gst-meet --web-socket-url=wss://your.jitsi.domain/xmpp-websocket \
         --room-name=roomname \
         --send-pipeline="filesrc location=shake-it-off.flac ! queue ! flacdec ! audioconvert ! audioresample ! opusenc name=audio"
```

Stream a .webm file containing VP8 video and Vorbis audio to the conference. This pipeline passes the VP8 stream through efficiently without transcoding, and transcodes the audio from Vorbis to Opus:

```
gst-meet --web-socket-url=wss://your.jitsi.domain/xmpp-websocket \
         --room-name=roomname \
         --send-pipeline="filesrc location=big-buck-bunny_trailer.webm ! queue ! matroskademux name=demuxer
                          demuxer.video_0 ! queue name=video
                          demuxer.audio_0 ! queue ! vorbisdec ! audioconvert ! audioresample ! opusenc name=audio"
```

Stream the default video & audio inputs to the conference, encoding as VP8 and Opus, display incoming video streams at 360p, and play back incoming audio (a very basic, but completely native, Jitsi Meet conference!):

```
gst-meet --web-socket-url=wss://your.jitsi.domain/xmpp-websocket \
         --room-name=roomname \
         --send-pipeline="autovideosrc ! queue ! videoconvert ! vp8enc buffer-size=1000 deadline=1 name=video
                          autoaudiosrc ! queue ! audioconvert ! audioresample ! opusenc name=audio" \
         --recv-pipeline-participant-template="opusdec name=audio ! autoaudiosink
                                               vp8dec name=video ! videoconvert ! videoscale ! video/x-raw,width=640,height=360 ! autovideosink"
```

Record a .webm file for each other participant, containing VP8 video and Opus audio, without needing to do any transcoding:

```
gst-meet --web-socket-url=wss://your.jitsi.domain/xmpp-websocket \
         --room-name=roomname \
         --recv-pipeline-participant-template="webmmux name=muxer ! filesink location={participant_id}.webm
                                               opusparse name=audio ! muxer.audio_0
                                               identity name=video ! muxer.video_0"
```

The above .webm file unfortunately may not play in most players, because the resolution will change whenever JVB changes which simulcast layer it is forwarding. You could post-process the recording to scale the lower resolution frames up, or you could do the decoding, scaling, and re-encoding at recording time as follows:

```
gst-meet --web-socket-url=wss://your.jitsi.domain/xmpp-websocket \
         --room-name=roomname \
         --recv-pipeline-participant-template="webmmux name=muxer ! filesink location={participant_id}.webm
                                               opusparse name=audio ! muxer.audio_0
                                               vp8dec name=video ! videoconvert ! videoscale ! video/x-raw,width=1280,height=720 ! vp8enc ! muxer.video_0"
```

Play a YouTube video in the conference. By requesting Opus audio and VP9 video from YouTube, and setting the Jitsi Meet video codec to VP9, no transcoding is necessary. Note that not every YouTube video has VP9 and Opus available, so the pipeline may need adjusting for other videos.

```
YOUTUBE_URL="https://www.youtube.com/watch?v=vjV_2Ri2rfE"
gst-meet --web-socket-url=wss://your.jitsi.domain/xmpp-websocket \
         --room-name=roomname \
         --video-codec=vp9 \
         --send-pipeline="curlhttpsrc location=\"$(youtube-dl -g $YOUTUBE_URL -f 'bestaudio[acodec=opus]')\" ! queue ! matroskademux name=audiodemux
                          curlhttpsrc location=\"$(youtube-dl -g $YOUTUBE_URL -f 'bestvideo[vcodec=vp9]')\" ! queue ! matroskademux name=videodemux
                          audiodemux.audio_0 ! queue ! clocksync name=audio
                          videodemux.video_0 ! queue ! clocksync name=video"
```

## Feature flags

By default, the `rustls` TLS library is used with the system's native root certificates. This can be turned off by passing `--no-default-features` to Cargo, and one of the following features can be enabled:

```
tls-rustls-native-roots  use rustls for TLS with the system's native root certificates (the default)
tls-rustls-webpki-roots  use rustls for TLS and bundle webpki's root certificates
tls-native               link to the system native TLS library
tls-native-vendored      automatically build a copy of OpenSSL and statically link to it
```

Building with the `tls-insecure` feature adds a `--tls-insecure` command line flag which disables certificate verification. Use this with extreme caution.

The `tls-*` flags only affect the TLS library used for the WebSocket connections (to the XMPP server and to the JVB). Gstreamer uses its own choice of TLS library for its elements. DTLS-SRTP (the media streams) is handled via GStreamer and uses automatically-generated ephemeral certificates which are authenticated over the XMPP signalling channel.

Building with the `log-rtp` feture adds a `--log-rtp` command line flag which logs information about every RTP and RTCP packet at the `DEBUG` level.

## Debugging

It can sometimes be tricky to get GStreamer pipeline syntax and structure correct. To help with this, you can try setting the `GST_DEBUG` environment variable (for example, `3` is modestly verbose, while `6` produces copious per-packet output). You can also set `GST_DEBUG_DUMP_DOT_DIR` to the relative path to a directory (which must already exist). `.dot` files containing the pipeline graph will be saved to this directory, and can be converted to `.png` with the `dot` tool from GraphViz; for example `dot filename.dot -Tpng > filename.png`.

## License

`gst-meet`, `lib-gst-meet`, `nice` and `nice-sys` are licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

The dependency `xmpp-parsers` is licensed under the Mozilla Public License, Version 2.0, https://www.mozilla.org/en-US/MPL/2.0/

The dependency `gstreamer` is licensed under the GNU Lesser General Public License, Version 2.1, https://www.gnu.org/licenses/old-licenses/lgpl-2.1.en.html.

## Contribution

Any kinds of contributions are welcome as a pull request.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in these crates by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Acknowledgements

`gst-meet` development is sponsored by [AVStack](https://www.avstack.io/). We provide globally-distributed, scalable, managed Jitsi Meet backends.
