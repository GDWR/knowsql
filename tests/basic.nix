{ knowsql, pkgs, ... }:
pkgs.nixosTest {
  name = "basic";
  nodes.machine = { config, pkgs, ... }: {
    imports = [ knowsql.nixosModules.default { } ];
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
}