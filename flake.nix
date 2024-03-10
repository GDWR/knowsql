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
          tag = knowsql.version;
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
      });
      devShells = forAllSystems (pkgs: {
        default = pkgs.callPackage ./shell.nix { };
      });
      checks = forAllSystems (pkgs: {
        # https://nixos.org/manual/nixos/stable/index.html#sec-nixos-tests
        basic = pkgs.nixosTest {
          name = "basic";
          nodes.machine = { config, pkgs, ... }: {
            imports = [ nixosModules.knowsql { } ];
            environment.systemPackages = [ pkgs.netcat ];

            services.knowsql.enable = true;

            users.users.user = {
              isNormalUser = true;
              extraGroups = [ "wheel" ];
            };

            system.stateVersion = "23.11";
          };
          testScript = ''
            machine.start()
            machine.wait_for_unit('default.target')

            machine.wait_for_open_port(2288, 'localhost', timeout=10)
            machine.succeed('printf "set hello world\nexit" | nc -N localhost 2288 | grep "OK"', timeout=10)
            machine.succeed('printf "get hello\nexit" | nc -N localhost 2288 | grep "world"', timeout=10)
          '';
        };
        basicRemote = pkgs.nixosTest {
          name = "basicRemote";
          nodes = {
            server = { config, pkgs, ... }: {
              imports = [ nixosModules.knowsql { } ];
              services.knowsql.enable = true;
              networking.firewall = {
                enable = true;
                allowedTCPPorts = [ 2288 ];
              };
              system.stateVersion = "23.11";
            };
            client = { config, pkgs, ... }: {
              environment.systemPackages = [ pkgs.netcat ];
              system.stateVersion = "23.11";
            };
          };
          testScript = ''
            start_all()

            client.wait_for_open_port(2288, 'server', timeout=10)
            client.succeed('printf "set hello world\nexit" | nc -N server 2288 | grep "OK"', timeout=10)
            client.succeed('printf "get hello\nexit" | nc -N server 2288 | grep "world"', timeout=10)
          '';
        };
        basicRemoteOver9000 = pkgs.nixosTest {
          name = "basicRemoteOver9000";
          nodes = {
            server = { config, pkgs, ... }: {
              imports = [ nixosModules.knowsql { } ];
              services.knowsql = {
                enable = true;
                port = 9001;
              };
              networking.firewall = {
                enable = true;
                allowedTCPPorts = [ 9001 ];
              };
              system.stateVersion = "23.11";
            };
            client = { config, pkgs, ... }: {
              environment.systemPackages = [ pkgs.netcat ];
              system.stateVersion = "23.11";
            };
          };
          testScript = ''
            start_all()

            client.wait_for_open_port(9001, 'server', timeout=10)
            client.succeed('printf "set hello world\nexit" | nc -N server 9001 | grep "OK"', timeout=10)
            client.succeed('printf "get hello\nexit" | nc -N server 9001 | grep "world"', timeout=10)
          '';
        };
      });

      nixosModules = {
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
