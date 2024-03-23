# knowsql 

knowsql is a Key Value store implemented as a [bitcask](https://riak.com/assets/bitcask-intro.pdf) accessible over tcp.

This package packages the [Microsoft Garnet](https://github.com/microsoft/garnet/) Benchmark tool which can be ran using nix via;
```console
nix shell github:gdwr/knowsql#garnet-benchmark --command Resp.benchmark
```
