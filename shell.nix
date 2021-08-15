with import <nixpkgs> {};
mkShell {
  name = "gst-meet";
  buildInputs = [
    pkg-config
    glib
    gst_all_1.gstreamer
    gst_all_1.gst-plugins-base
    gst_all_1.gst-plugins-good
    gst_all_1.gst-plugins-bad
    libnice
  ] ++ (if stdenv.isDarwin then [
    darwin.apple_sdk.frameworks.AppKit
    darwin.apple_sdk.frameworks.Security
  ] else []);
}
