{
  outputs = { self, nixpkgs, ... }:
    let
      forAllSystems = function:
        nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ]
        (system: function nixpkgs.legacyPackages.${system});
    in rec {
      formatter = forAllSystems (pkgs: pkgs.nixfmt);
      packages = forAllSystems (pkgs: rec {
        default = knowsql;
        knowsql = pkgs.callPackage ./default.nix { };
        dockerImage = pkgs.dockerTools.buildImage {
          name = "knowsql";
          tag = self.rev or self.dirtyRev;
          runAsRoot = ''
            #!${pkgs.stdenv.shell}
            mkdir -p /etc/knowsql
            printf 'port = 2288\ndata_dir = "/etc/knowsql/data"' > /etc/knowsql/config.toml
          '';
          config = {
            Entrypoint = [ "${knowsql}/bin/knowsql" ];
          };
        };
        docs = pkgs.stdenv.mkDerivation {
          name = "knowsql-docs";
          src = ./.;

          buildInputs = [ pkgs.mdbook ];
          buildPhase = ''
            cd ./docs
            mdbook build --dest-dir $out
          '';
        };
	garnet-benchmark = pkgs.callPackage ./externals/garnet-benchmark { };
      });
      devShells = forAllSystems (pkgs: {
        default = pkgs.callPackage ./shell.nix { };
      });
      checks = forAllSystems (pkgs: {
        basic = pkgs.callPackage ./tests/basic.nix { inherit pkgs; knowsql = self;};
        remote = pkgs.callPackage ./tests/remote.nix { inherit pkgs; knowsql = self;};
        over9000 = pkgs.callPackage ./tests/over9000.nix { inherit pkgs; knowsql = self;};
      });

      nixosModules = rec {
        default = knowsql;
        knowsql = { config, lib, pkgs, ... }:
          with lib;
          let settingsFormat = pkgs.formats.toml { };
          in {
            options = {
              services.knowsql = {
                enable = mkEnableOption "knowsql";
                data_dir = lib.mkOption {
                  type = lib.types.path;
                  default = "/etc/knowsql/data";
                  description =
                    "The directory where knowsql will store its data.";
                };
                port = lib.mkOption {
                  type = lib.types.int;
                  default = 2288;
                  description = "The port on which knowsql will listen.";
                };
              };
            };

            config = mkIf config.services.knowsql.enable {
              environment.etc."knowsql/config.toml".source =
                settingsFormat.generate "config.toml" {
                  port = config.services.knowsql.port;
                  data_dir = config.services.knowsql.data_dir;
                };

              systemd.services.knowsql = {
                description = "Knowsql";
                after = [ "network.target" ];
                wantedBy = [ "multi-user.target" ];
                serviceConfig = {
                  ExecStart = "${packages.${pkgs.system}.knowsql}/bin/knowsql";
                  Restart = "always";
                };
              };
            };
          };
      };
    };
}
