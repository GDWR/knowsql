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
          environment.systemPackages = [
             packages.x86_64-linux.hoard
             pkgs.netcat
          ];

          users.users.user = {
            isNormalUser = true;
            extraGroups = [ "wheel" ]; 
          };
    
          system.stateVersion = "23.11";
        };
        testScript = ''
          machine.start()
          machine.wait_for_unit('default.target')

          # In the future we will use a NixOSModule to start the service
          machine.execute('hoard > /dev/console 2>&1 &')

          machine.wait_for_open_port(6379, 'localhost', 10)
          machine.succeed('echo "set hello world" | nc localhost 6379 | grep "OK"')
          machine.succeed('echo "get hello" | nc localhost 6379 | grep "world"')
        '';
      };
      basicRemote = pkgs.nixosTest {
        name = "basicRemote";
        nodes = { 
          server = { config, pkgs, ... }: {          
            environment.systemPackages = [
                packages.x86_64-linux.hoard
            ];
            networking.firewall = {
              enable = true;
              allowedTCPPorts = [ 6379 ];
            };
            system.stateVersion = "23.11";
          };
          client = { config, pkgs, ... }: {          
            environment.systemPackages = [
                pkgs.netcat
            ];
            system.stateVersion = "23.11";
          };
        };
        testScript = ''
          start_all()

          # In the future we will use a NixOSModule to start the service
          server.execute('hoard > /dev/console 2>&1 &')

          client.wait_for_open_port(6379, 'server', 10)
          client.succeed('echo "set hello world" | nc server 6379 | grep "OK"')
          client.succeed('echo "get hello" | nc server 6379 | grep "world"')
        '';
      };
    });
  };
}
