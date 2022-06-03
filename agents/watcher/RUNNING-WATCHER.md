# Running a Watcher

## Overview

The watcher is a crucial component of the Nomad security model. Watchers secure applications built on Nomad by observing the updater's attestations on the home contract. In the case of any malicious or faulty attestations, the watcher will disconnect its given application from the underlying messaging channel, eliminating the impact of fraud on that app.

## Steps to Running a Watcher

### High Level

1. Provision a watcher attestation key
2. Enroll the attestation key address on the desired networks [Nomad governance]
3. Provision transaction signer key(s)
4. Fund the transaction signer address(es) on the desired networks
5. Choose RPC endpoint(s) for desired networks.
6. Setup agent monitoring
7. Place the information from steps 1-5 into the watcher's environment and run the agent

### Details

**Step 1: Provision Watcher Key**

The watcher attestation key is used to sign attestations that fraud occurred. Every cross-chain app will enroll a set of watcher attestation addresses. If the app receives an attestation of fraud from an enrolled watcher, the app will disconnect from the messaging channel.

The operator must provision a key for the application to enroll. Agents accept either raw hex keys or AWS KMS keys.

**Step 2: Enroll Watcher Key**

The agent operator should forward the newly provisioned watcher address to the Nomad team. Nomad governance will then enroll the address on the desired application for the appropriate networks.

**Step 3: Provision Transaction Signer Key(s)**

In order for the watcher to submit an attestation of fraud, it must submit a transaction. The agent operator must provision one or more transaction signer keys. These can be the same across all networks or unique per-network.

**Step 4: Fund Transaction Signers**

The agent operator must fund the transaction signer address(es) on all networks. We recommend funding _each address on each chain_ with the a minimum of the values documented [here](#watcher-transaction-signer-funding), according to the network. The agent should have at least the minimum amount at all times.

**Step 5: Choose RPC Endpoints**

The watcher must connect to all chains involved in the channels it watches over. We recommend using private RPC endpoints for the best reliability. This would include connecting through an internally run local node or through top-quality node providers.

**Step 6**

All Nomad agents produce logs and metrics. It is up to the agent operator how they setup the reception of this data. Agents expose Prometheus metrics at port `9090` by default. Agents output logs to stdout in JSON format, following standard [12-factor-app](https://12factor.net/logs) methodology.

**Step 7**

In order to run a watcher, you must configure the watcher's environment to receive the information from steps 1-5. See our [guide on running agents](../RUNNING-AGENTS.md) for more info on configuration and running the agent.

## Watcher Transaction Signer Funding

| Chain        | Funding Amount |
| ------------ | -------------- |
| Ethereum     | 3 ETH          |
| Moonbeam     | 5 GLMR         |
| Milkomeda C1 | 5 milkADA      |
| Evmos        | 5 EVMOS        |
| xDai         | 5 xDAI         |
| Avalanche    | 4 AVAX         |
| Polygon      | 5 MATIC        |
| Arbitrum     | TBD            |
| Optimism     | TBD            |

<br>

**Reasoning for Funding Amounts**

The highest daily average gas price on Ethereum to-date is ~710 gwei. A watcher `unenrollReplica` transaction is ~120k gas while a `doubleUpdate` transaction is ~200k gas. If we 10x the highest daily average gas price, we get 7100 gwei. This means that calling `unenrollReplica` will cost 0.852 ETH and calling `doubleUpdate` will cost 1.42 ETH.

unenrollReplica: (710 x 10 x 120,000) / 1e9 = 0.852 ETH

doubleUpdate: (710 x 10 x 200,000) / 1e9 = 1.42 ETH

A minimum of 3 ETH worth of funds per watcher transaction signer is recommended. For networks outside of Ethereum, the funding amount is inflated due to the fact that the dollar cost of funds is much cheaper on other chains.
