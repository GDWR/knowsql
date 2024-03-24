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
      environment.systemPackages = [ pkgs.redis ];
      system.stateVersion = "23.11";
    };
  };
  testScript = ''
    start_all()
    client.wait_for_open_port(2288, 'server', timeout=10)

    client.succeed('redis-cli -h server -p 2288 SET hello world | grep "OK"', timeout=10)
    client.succeed('redis-cli -h server -p 2288 GET hello | grep "world"', timeout=10)
  '';
}
