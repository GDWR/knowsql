{ knowsql, pkgs, ... }:
pkgs.nixosTest {
  name = "basic";
  nodes.client = { config, pkgs, ... }: {
    imports = [ knowsql.nixosModules.default { } ];
    environment.systemPackages = [ pkgs.redis ];

    services.knowsql.enable = true;

    users.users.user = {
      isNormalUser = true;
      extraGroups = [ "wheel" ];
    };

    system.stateVersion = "23.11";
  };
  testScript = ''
    start_all()
    client.wait_for_open_port(2288, 'localhost', timeout=10)

    client.succeed('redis-cli -p 2288 SET hello world | grep "OK"', timeout=10)
    client.succeed('redis-cli -p 2288 GET hello | grep "world"', timeout=10)
  '';
}
