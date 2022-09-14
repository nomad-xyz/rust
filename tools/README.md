## Tools

### Killswitch

- Kills bridge channels manually in effectively the same way as Watcher
- Takes a set of environment variables and an agent config file
- Can either kill all configured networks or all inbound to a single network

#### Interface

```$ killswitch --help```
```bash
killswitch
Command line args

USAGE:
    killswitch --app <APP> <--all|--all-inbound <NETWORK>>

OPTIONS:
        --all                      Kill all available networks
        --all-inbound <NETWORK>    Kill all replicas on network
        --app <APP>                Which app to kill [possible values: token-bridge]
    -h, --help                     Print help information
```

#### Environment variables

A config file can be specified with `CONFIG_PATH` (local) or `CONFIG_URL` (remote).

Secrets must be in the explicit form `<NETWORK>_TXSIGNER_{KEY,ID}` and `<NETWORK>_ATTESTATION_SIGNER_{KEY,ID}`.

#### Example environment variables file
```text
CONFIG_PATH=./config.json
DEFAULT_RPCSTYLE=ethereum
DEFAULT_SUBMITTER_TYPE=local

GOERLI_CONNECTION_URL=https://rpc.endpoint
POLYGONMUMBAI_CONNECTION_URL=https://rpc.endpoint
RINKEBY_CONNECTION_URL=https://rpc.endpoint

GOERLI_TXSIGNER_KEY=0x0
POLYGONMUMBAI_TXSIGNER_KEY=0x0
RINKEBY_TXSIGNER_KEY=0x0

GOERLI_ATTESTATION_SIGNER_KEY=0x0
POLYGONMUMBAI_ATTESTATION_SIGNER_KEY=0x0
RINKEBY_ATTESTATION_SIGNER_KEY=0x0
```

#### Return value

- Streams newline-delimited JSON containing error / success reports
