with import <nixpkgs> {};
let
  libnice-patched = libnice.overrideAttrs(old: rec {
    buildInputs = [
      gst_all_1.gstreamer
      gst_all_1.gst-plugins-base
      openssl
    ];
    outputs = [ "bin" "out" "dev" ];
    mesonFlags = old.mesonFlags ++ ["-Dgupnp=disabled" "-Dgtk_doc=disabled"];
    meta.platforms = lib.platforms.unix;
  });
in
mkShell {
  name = "gst-meet";
  buildInputs = [
    cargo
    pkg-config
    openssl
    glib
    glib-networking
    gst_all_1.gstreamer
    gst_all_1.gst-plugins-base
    gst_all_1.gst-plugins-good
    gst_all_1.gst-plugins-bad
    libnice-patched
  ] ++ (if stdenv.isDarwin then [
    darwin.apple_sdk.frameworks.AppKit
    darwin.apple_sdk.frameworks.Security
  ] else []);

  GIO_EXTRA_MODULES = ["${pkgs.glib-networking.out}/lib/gio/modules"];
}
