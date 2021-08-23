with import <nixpkgs> {};
let
  gstreamer = gst_all_1.gstreamer.overrideAttrs(old: rec {
    version = "1.19.9999";
    src = fetchGit {
      url = "https://gitlab.freedesktop.org/gstreamer/gstreamer.git";
      rev = "637b0d8dc25b660d3b05370e60a95249a5228a39";
    };
    patches = [];
  });
  gst-plugins-base = (gst_all_1.gst-plugins-base.override {
    gstreamer = gstreamer;
  }).overrideAttrs(old: rec {
    version = "1.19.9999";
    src = fetchGit {
      url = "https://gitlab.freedesktop.org/gstreamer/gst-plugins-base.git";
      rev = "f5a79ce05f62ad98134435955ed3d10d22f17cb9";
    };
    patches = [];
  });
  gst-plugins-good = (gst_all_1.gst-plugins-good.override {
    gst-plugins-base = gst-plugins-base;
  }).overrideAttrs(old: rec {
    version = "1.19.9999";
    src = fetchGit {
      url = "https://gitlab.freedesktop.org/hgr/gst-plugins-good.git";
      ref = "hgr/twcc-fixes";
      rev = "3cff164ef4fab1a74ecfe5fd247edb723c9a41a1";
    };
    patches = [];
  });
  gst-plugins-bad = (gst_all_1.gst-plugins-bad.override {
    gst-plugins-base = gst-plugins-base;
  }).overrideAttrs(old: rec {
    version = "1.19.9999";
    src = fetchGit {
      url = "https://gitlab.freedesktop.org/gstreamer/gst-plugins-bad.git";
      rev = "4eb22b769559ef2696a78a03b30de215bd677d47";
    };
    patches = [];
    mesonFlags = old.mesonFlags ++ ["-Dgs=disabled" "-Disac=disabled" "-Dldac=disabled" "-Donnx=disabled" "-Dopenaptx=disabled" "-Dqroverlay=disabled"];
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
