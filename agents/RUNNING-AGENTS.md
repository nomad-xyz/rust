# Running Agents

## Overview

Agents read settings from a mix of our public [JSON configuration files](/configuration/configs/) and private environment variables.

Our hosted JSON configs provide public network info such as contract addresses and chain finality settings. Environment variables specify what networks you want to run the agent against as well as secrets such as signer keys or private RPC endpoints.

## Configuring an Agent

To configure an agent, you must populate the proper environment variables. The key fields one must specify are:

- **Run Environment**
  - `RUN_ENV`: Development, staging, or production
- **Agent Home**
  - `AGENT_HOME_NAME`: What home the agent is running against
- **Agent Replicas**

  - Specify networks:
    - `AGENT_REPLICA_0_NAME`, `AGENT_REPLICA_1_NAME`, `AGENT_REPLICA_2_NAME`, etc...
    - What replica(s) the agent will run against
  - Default to all connected networks:
    - `AGENT_REPLICAS_ALL`
    - Expects all connected replicas if `true`
    - Expects specified networks if `false` or not set

- **RPC Info**
  - Network-specific:
    - `{network}_RPCSTYLE`: What RPC style `network` is; "ethereum" for all EVM chains
    - `{network}_CONNECTION_URL`: RPC endpoint url
  - Default:
    - `DEFAULT_RPCSTYLE`: Default rpc style for any network not explicitly configured
- **Transaction Submission Info**
  - Network-specific:
    - Transaction Submission Type:
      - `{network}_SUBMITTER_TYPE`
      - `local` for local signing/submitting
      - `gelato` if you are integrated with Gelato Relay
    - Local Submission:
      - Transaction signer key:
        - Hex key:
          - `{network}_TXSIGNER_KEY`
          - Raw 0x-prefixed hex key
        - AWS Key:
          - `{network}_TXSIGNER_ID`
          - AWS key id
    - Gelato Submission (ignore if you do not plan on using Gelato Relay):
      - Sponsor signer:
        - Hex key:
          - `{network}_GELATO_SPONSOR_KEY`
          - Raw 0x-prefixed hex key
        - AWS Key:
          - `{network}_GELATO_SPONSOR_ID`
          - AWS key id
      - Fee token
        - `{network}_GELATO_SPONSOR_FEETOKEN`
        - 0x-prefixed token contract address
  - Default:
    - Default for any network not explicitly configured
    - Same as network-specific (above) but replacing specific `{network}` with `DEFAULT`
    - Example:
      - `DEFAULT_SUBMITTER_TYPE=local`
      - `DEFAULT_TXSIGNER_ID=some_aws_id`
      - All networks use `local` transaction submission with the default txsigner key
- **Attestation Signer (optional)**
  - Required _only_ for updater and watcher
  - Hex key:
    - `ATTESTATION_SIGNER_KEY`
    - Raw 0x-prefixed hex key
  - AWS Key:
    - `ATTESTATION_SIGNER_ID`
    - AWS key id

<br>

Defaults:
Note that default values are only used if a network-specific value is not provided. In other words, network-specific values override the default if both are provided.

AWS Keys:
Note that the AWS `key_id` field can be a key id, key name, alias name, or alias ARN, as documented in the [Rusoto KMS docs](https://docs.rs/rusoto_kms/latest/rusoto_kms/struct.GetPublicKeyRequest.html#structfield.key_id). For more information on configuring AWS credentials, please refer to the [Rusoto AWS credentials usage documentation](https://github.com/rusoto/rusoto/blob/master/AWS-CREDENTIALS.md#credentials).

For more info on our different run environments and key configuration/provisioning, please refer to our [agents operations page](./AGENT-OPERATIONS.md).

You can see an example .env file below:

```
# Only runs agent for Ethereum <> Moonbeam channel (production)
RUN_ENV=production
AGENT_HOME_NAME=ethereum
AGENT_REPLICA_0_NAME=moonbeam

# can provide default rpc style for all networks, or specify network specific
# network-specific values always override the default
DEFAULT_RPCSTYLE=ethereum
ETHEREUM_RPCSTYLE=ethereum
MOONBEAM_RPCSTYLE=ethereum

# provide network-specific RPC endpoints
ETHEREUM_CONNECTION_URL=https://main-light.eth.linkpool.io/
MOONBEAM_CONNECTION_URL=https://rpc.api.moonbeam.network

# we will default to local transaction signing/submission
DEFAULT_SUBMITTER_TYPE=local

# can provide tx signer as hex key (for ethereum) or aws key (for moonbeam)
# again, default tx signer is overriden by network-specifics
DEFAULT_TXSIGNER_KEY=0x1111111111111111111111111111111111111111111111111111111111111111
ETHEREUM_TXSIGNER_KEY=0x1111111111111111111111111111111111111111111111111111111111111111
MOONBEAM_TXSIGNER_ID=dummy_id

# can provide attestation signer as aws or hex key
ATTESTATION_SIGNER_ID=dummy_id
```

If you would like to configure an agent to run against all connected networks (against all replicas the home is connected to), see [this example](https://github.com/nomad-xyz/rust/blob/main/fixtures/env.test). For more examples of .env files, see our [test fixtures folder](https://github.com/nomad-xyz/rust/tree/main/fixtures).

## Running Agent

Once you have populated a .env file, running an agent is as simple as running the following command:

`env $(cat .env | xargs) cargo run --bin <AGENT>`

This will build the codebase and run the specified `<AGENT>` binary (updater, relayer, processor, or watcher) using the provided environment variables.

## Agents Release Process

Our release process follows a monthly cadence. We follow [Semantic Versioning](https://semver.org/), where breaking changes constitute changes that break agent configuration compatibility.

We manage releases through GitHub. You can find new per-agent releases [here](https://github.com/nomad-xyz/rust/releases).

## Production Builds

When making changes to the Rust codebase, it is important to ensure the Docker build used in production environments still works. You can check this automatically in CI as it is built on every PR ([see docker workflow here](https://github.com/nomad-xyz/rust/blob/main/.github/workflows/docker.yml)), however you can check it much faster usually by attempting to build it locally.

You can build the docker image by running the following script in the `rust` directory:

`./build.sh latest`

If that goes smoothly, you can rest assured it will most likely also work in CI.
