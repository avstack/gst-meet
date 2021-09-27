with import <nixpkgs> {};
let
  gstreamer = gst_all_1.gstreamer.overrideAttrs(old: rec {
    version = "1.19.2";
    src = fetchurl {
      url = "https://gstreamer.freedesktop.org/src/${old.pname}/${old.pname}-${version}.tar.xz";
      sha256 = "6e5b7ba5931e5389c21d10986615f72859b2cc8830a5ba8b5253dad1ba7e6e0d";
    };
    patches = [];
    mesonFlags = old.mesonFlags ++ ["-Dorc=disabled"];
  });
  gst-plugins-base = (gst_all_1.gst-plugins-base.override {
    gstreamer = gstreamer;
  }).overrideAttrs(old: rec {
    version = "1.19.2";
    src = fetchurl {
      url = "https://gstreamer.freedesktop.org/src/${old.pname}/${old.pname}-${version}.tar.xz";
      sha256 = "cde304fd3c006b61a97894b5c4e6f4687edd52cab6767d536b09bdb78d31a513";
    };
    patches = [];
    mesonFlags = old.mesonFlags ++ ["-Dorc=disabled"];
  });
  gst-plugins-good = (gst_all_1.gst-plugins-good.override {
    gst-plugins-base = gst-plugins-base;
  }).overrideAttrs(old: rec {
    version = "1.19.2";
    src = fetchurl {
      url = "https://gstreamer.freedesktop.org/src/${old.pname}/${old.pname}-${version}.tar.xz";
      sha256 = "4be92e021144bc6dca5082d028275d4b6e69183c01b90791e0837173d58d4e2e";
    };
    patches = [];
    mesonFlags = old.mesonFlags ++ ["-Dorc=disabled"];
  });
  gst-plugins-bad = (gst_all_1.gst-plugins-bad.override {
    gst-plugins-base = gst-plugins-base;
  }).overrideAttrs(old: rec {
    version = "1.19.2";
    src = fetchurl {
      url = "https://gstreamer.freedesktop.org/src/${old.pname}/${old.pname}-${version}.tar.xz";
      sha256 = "5382f98a9af2c92e5c0ca4fcb3911025cafd9f89b3142b206eb7b92b812e0979";
    };
    patches = [];
    mesonFlags = old.mesonFlags ++ ["-Dorc=disabled" "-Dgs=disabled" "-Disac=disabled" "-Dldac=disabled" "-Donnx=disabled" "-Dopenaptx=disabled" "-Dqroverlay=disabled" "-Dtests=disabled"];
  });
  libnice-patched = libnice.override {
    gst_all_1 = {
      gstreamer = gstreamer;
      gst-plugins-base = gst-plugins-base;
    };
  };
in
mkShell {
  name = "gst-meet";
  buildInputs = [
    cargo
    pkg-config
    glib
    glib-networking
    gstreamer
    gst-plugins-base
    gst-plugins-good
    gst-plugins-bad
    libnice-patched
  ] ++ (if stdenv.isDarwin then [
    darwin.apple_sdk.frameworks.AppKit
    darwin.apple_sdk.frameworks.Security
  ] else []);

  GIO_EXTRA_MODULES = ["${pkgs.glib-networking.out}/lib/gio/modules"];
}
