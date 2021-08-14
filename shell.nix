{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "gst-meet";
  buildInputs = with pkgs; [
    glib
    pkg-config
  ];
}
