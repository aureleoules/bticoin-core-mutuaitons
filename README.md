#  Bticoin Core Mutuaitons

This repository contains a CLI to perform mutation testing on the Bitcoin Core codebase and orchestrate the execution of the workers.

Testing Bitcoin Core's source code with mutations was first proposed by [brunoerg](https://github.com/brunoerg) [here](https://github.com/bitcoin/bitcoin/pull/24499) and [here](https://github.com/brunoerg/bitcoin-core-mutation).

### Server

The server host mutations and assigns work to different workers.

```bash
version: "3.1"
services:
  bcm_server:
    image: aureleoules/bcm-server
    container_name: bcm-server
    networks:
      - bcm
    restart: always
    command: --redis redis
    
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
docker run -it --rm aureleoules/bcm-mutator --server https://YOUR_SERVER.com -f src/wallet/spend.cpp -f src/validation.cpp
```

### Worker

The worker performs the mutations and reports the results to the server.
It patches the corresponding file and runs the unit tests and the functional tests.
If the CI fails, the mutation is considered as killed. Otherwise, it is considered as survived.

```bash
version: "3.1"
services:
  bcm_worker:
    image: aureleoules/bcm-worker
    container_name: bcm-worker
    restart: always
    command:
      "--server https://YOUR_SERVER.com"
```
