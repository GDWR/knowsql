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
      environment.systemPackages = [ pkgs.redis ];
      system.stateVersion = "23.11";
    };
  };
  testScript = ''
    start_all()
    client.wait_for_open_port(9001, 'server', timeout=10)

    client.succeed('redis-cli -h server -p 9001 SET hello world | grep "OK"', timeout=10)
    client.succeed('redis-cli -h server -p 9001 GET hello | grep "world"', timeout=10)
  '';
}
