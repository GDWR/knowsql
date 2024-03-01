{
  description = "A Rust implementation of bitcask.";

  outputs = { self, nixpkgs, ... }: let
    forAllSystems = function:
      nixpkgs.lib.genAttrs [
        "x86_64-linux"
        "aarch64-linux"
      ] (system: function nixpkgs.legacyPackages.${system});
  in rec {
    packages = forAllSystems (pkgs: rec {
      default = hoard;
      hoard = let
        manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
      in pkgs.rustPlatform.buildRustPackage rec {
        pname = manifest.name;
        version = manifest.version;

        src = pkgs.lib.cleanSource ./.;
	cargoLock.lockFile = ./Cargo.lock;
      };
    });
    devShells= forAllSystems (pkgs: {
      default = pkgs.mkShell {
        packages = [ pkgs.cargo pkgs.rustc ];
      };
    });
  };
}
