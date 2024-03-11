{ knowsql, pkgs, ... }:
pkgs.nixosTest {
  name = "remote";
  nodes = {
    server = { config, pkgs, ... }: {
      imports = [ knowsql.nixosModules.default { } ];
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
}