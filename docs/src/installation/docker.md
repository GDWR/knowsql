# Docker

knowsql provides docker containers built in CI for each tagged version. \
These can be [found here](https://github.com/GDWR/knowsql/pkgs/container/knowsql) along with other development images. 

Release tags can be [found here](https://github.com/GDWR/knowsql/releases).

## Docker Compose

```yaml filename="compose.yml"
services:
    knowsql:
        image: ghcr.io/gdwr/knowsql:latest
        ports:
            - "2288:2288"
        # volumes:
        #     - ./config.toml:/etc/knowsql/config.toml
```

# Docker

```console
docker run -p 2288:2288 ghcr.io/gdwr/knowsql:latest
```
