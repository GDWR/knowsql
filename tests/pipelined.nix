{ knowsql, pkgs, ... }:
pkgs.nixosTest {
  name = "pipelined";
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

    # Pipeline 3 PINGS, expect 3 PONGS
    client.succeed(r'[ $(printf "PING\r\nPING\r\nPING\r\n" | redis-cli -p 2288 | grep "PONG" | wc -l) -eq 3 ];', timeout=10)
  '';
}
