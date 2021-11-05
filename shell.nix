with import <nixpkgs> {};
let
  gstreamer = gst_all_1.gstreamer.overrideAttrs(old: rec {
    version = "1.19.3";
    src = fetchurl {
      url = "https://gstreamer.freedesktop.org/src/${old.pname}/${old.pname}-${version}.tar.xz";
      sha256 = "906d7d4bf92f941586c0cbce717d9cad6aac36994e16fa6f2f153e07e3221bca";
    };
    patches = [];
    mesonFlags = old.mesonFlags ++ ["-Dorc=disabled"];
  });
  gst-plugins-base = (gst_all_1.gst-plugins-base.override {
    gstreamer = gstreamer;
  }).overrideAttrs(old: rec {
    version = "1.19.3";
    src = fetchurl {
      url = "https://gstreamer.freedesktop.org/src/${old.pname}/${old.pname}-${version}.tar.xz";
      sha256 = "e277f198623a26c1b0a1e19734656392e9368bebf3677cd94262a1316a960827";
    };
    patches = [];
    mesonFlags = old.mesonFlags ++ ["-Dorc=disabled"];
  });
  gst-plugins-good = (gst_all_1.gst-plugins-good.override {
    gst-plugins-base = gst-plugins-base;
  }).overrideAttrs(old: rec {
    version = "1.19.3";
    src = fetchurl {
      url = "https://gstreamer.freedesktop.org/src/${old.pname}/${old.pname}-${version}.tar.xz";
      sha256 = "79ea32a77fa47e6596530e38113bf97c113fd95658087d9a91ffb8af47d11d07";
    };
    patches = [];
    mesonFlags = old.mesonFlags ++ ["-Dorc=disabled"];
  });
  gst-plugins-bad = (gst_all_1.gst-plugins-bad.override {
    gst-plugins-base = gst-plugins-base;
  }).overrideAttrs(old: rec {
    version = "1.19.3";
    src = fetchurl {
      url = "https://gstreamer.freedesktop.org/src/${old.pname}/${old.pname}-${version}.tar.xz";
      sha256 = "50193a23b13713ccb32ee5d1852faeeaed29b91f8398285acdfd522fa3e16835";
    };
    patches = [];
    mesonFlags = old.mesonFlags ++ ["-Dorc=disabled" "-Dgs=disabled" "-Disac=disabled" "-Dldac=disabled" "-Donnx=disabled" "-Dopenaptx=disabled" "-Dqroverlay=disabled" "-Dtests=disabled"];
  });
  libnice-patched = (libnice.override {
    gst_all_1 = {
      gstreamer = gstreamer;
      gst-plugins-base = gst-plugins-base;
    };
  }).overrideAttrs(old: rec {
    buildInputs = [
      gst_all_1.gstreamer
      gst_all_1.gst-plugins-base
      openssl
    ];
    mesonFlags = old.mesonFlags ++ ["-Dgupnp=disabled"];
  });
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
