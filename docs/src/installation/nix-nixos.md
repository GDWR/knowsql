# Nix + NixOS

Knowsql offers a package that you can use, or a nixosModule that can be used to configure a systemd service for knowsql.

## Flake
Add knowsql to your flake inputs 
```nix
{
    inputs.knowsql = {
        url = "github:gdwr/knowsql";
        follows = "nixpkgs"; 
    };
    # rest of your flake...
}
```

### NixOS Module
Then import the module within your nixosConfiguration.
```nix 
{
    nixosConfigurations.example = {
        imports = [
            knowsql.nixosModules.knowsql
        ];

        services.knowsql.enable = true;

        #services.knowsql = {
        #    enable = true;
        #    port = 9001;            # Configure the port! make sure to enable the firewall ;)
        #    data_dir = "/tmp/data"; # Change where the datastore lives.
        #};
       
        # rest of your configuration...
    }
}
```

### Nix Package
```shell
nix run github:gdwr/knowsql
```
