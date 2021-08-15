with import <nixpkgs> {};
mkShell {
  name = "gst-meet";
  buildInputs = [
    pkg-config
    glib
    gst_all_1.gstreamer
    libnice
  ];
}
