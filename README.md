#  Bticoin Core Mutuaitons

This repository contains a CLI to perform mutation testing on the Bitcoin Core codebase.
Cirrus CI is used to run the tests in https://github.com/aureleoules/btc-mutations.

Testing Bitcoin Core's source code with mutations was first proposed by [brunoerg](https://github.com/brunoerg) [here](https://github.com/bitcoin/bitcoin/pull/24499) and [here](https://github.com/brunoerg/bitcoin-core-mutation).

### Server

The server host mutations and Cirrus-CI runs the tests.

```bash
version: "3.1"
services:
  bcm_server:
    image: aureleoules/bcm-server
    container_name: bcm-server
    networks:
      - bcm
    restart: always
    command: --redis redis --token aureleoules:token --token user2:token --mutation_repo /tmp/btc-mutations # See https://github.com/aureleoules/btc-mutations
    volumes:
      - ./btc-mutations:/tmp/btc-mutations
      - /home/YOUR_USER/.ssh:/root/.ssh:ro
    
  redis:
    image: redislabs/rejson
    container_name: bcm_redis
    networks:
      - bcm
    restart: always
    command: redis-server --save 60 1 --loglevel warning --loadmodule '/usr/lib/redis/modules/rejson.so'
    volumes:
      - ./db:/data

networks:
  bcm:
```

### Add mutations

```bash
docker run -it --rm aureleoules/bcm-mutator --token yourtoken --server https://YOUR_SERVER.com -f src/wallet/spend.cpp -f src/validation.cpp
```

If you are not using Docker your working directory must be Bitcoin Core. Otherwise, it will not able to find the files.
