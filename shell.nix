{ pkgs ? import <nixpkgs> { }, ... }:
pkgs.mkShell {
  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
  packages = with pkgs; [ rustc cargo clippy rustfmt rust-analyzer mdbook ];
}
