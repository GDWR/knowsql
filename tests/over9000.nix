{ knowsql, pkgs, ... }:
pkgs.nixosTest {
  name = "over9000";
  nodes = {
    server = { config, pkgs, ... }: {
      imports = [ knowsql.nixosModules.default { } ];
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
}