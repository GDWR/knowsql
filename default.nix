{ pkgs, ... }: 
let
  manifest = (pkgs.lib.importTOML ./Cargo.toml).workspace.package;
in 
  pkgs.rustPlatform.buildRustPackage {
    pname = "knowsql";
    version = manifest.version;

    src = ./.;
    cargoLock.lockFile = ./Cargo.lock;
  }