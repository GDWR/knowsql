{
  description = "A Rust implementation of bitcask.";

  outputs = { self, nixpkgs, ... }:
    let
      forAllSystems = function:
        nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ]
        (system: function nixpkgs.legacyPackages.${system});
    in rec {
      formatter = forAllSystems (pkgs: pkgs.nixfmt);
      packages = forAllSystems (pkgs: rec {
        default = hoard;
        hoard = let manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
        in pkgs.rustPlatform.buildRustPackage rec {
          pname = manifest.name;
          version = manifest.version;

          src = pkgs.lib.cleanSource ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
      });
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          packages = with pkgs; [
            rustc
            cargo

            clippy
            rustfmt
            rust-analyzer
          ];
        };
      });
      checks = forAllSystems (pkgs: {
        # https://nixos.org/manual/nixos/stable/index.html#sec-nixos-tests
        basic = pkgs.nixosTest {
          name = "basic";
          nodes.machine = { config, pkgs, ... }: {
            imports = [ nixosModules.hoard { } ];  
            environment.systemPackages = [ pkgs.netcat ];

            services.hoard.enable = true;

            users.users.user = {
              isNormalUser = true;
              extraGroups = [ "wheel" ];
            };

            system.stateVersion = "23.11";
          };
          testScript = ''
            machine.start()
            machine.wait_for_unit('default.target')

            machine.wait_for_open_port(6379, 'localhost', 10)
            machine.succeed('echo "set hello world" | nc localhost 6379 | grep "OK"')
            machine.succeed('echo "get hello" | nc localhost 6379 | grep "world"')
          '';
        };
        basicRemote = pkgs.nixosTest {
          name = "basicRemote";
          nodes = {
            server = { config, pkgs, ... }: {
              imports = [ nixosModules.hoard { } ];  
              services.hoard.enable = true;
              networking.firewall = {
                enable = true;
                allowedTCPPorts = [ 6379 ];
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

            client.wait_for_open_port(6379, 'server', 10)
            client.succeed('echo "set hello world" | nc server 6379 | grep "OK"')
            client.succeed('echo "get hello" | nc server 6379 | grep "world"')
          '';
        };
        basicRemoteOver9000 = pkgs.nixosTest {
          name = "basicRemoteOver9000";
          nodes = {
            server = { config, pkgs, ... }: {
              imports = [ nixosModules.hoard { } ];  
              services.hoard = {
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

            client.wait_for_open_port(9001, 'server', 10)
            client.succeed('echo "set hello world" | nc server 9001 | grep "OK"')
            client.succeed('echo "get hello" | nc server 9001 | grep "world"')
          '';
        };
      });

      nixosModules = {
        hoard = { config, lib, pkgs, ... }: {
          options = {
            services.hoard = {
              enable = lib.mkEnableOption "hoard";
              port = lib.mkOption {
                type = lib.types.int;
                default = 6379;
                description = "The port on which hoard will listen.";
              };
            };
          };

          config = lib.mkIf config.services.hoard.enable {
            systemd.services.hoard = {
              description = "Hoard";
              after = [ "network.target" ];
              wantedBy = [ "multi-user.target" ];
              environment.HOARD_PORT = "${toString config.services.hoard.port}";
              serviceConfig = {
                ExecStart = "${packages.x86_64-linux.hoard}/bin/hoard";
                Restart = "always";
              };
            };
          };
        };
      };
    };
}
