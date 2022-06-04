# Agent Gas Limit Values

## Context

Gas estimation is hard. `eth_estimateGas` calls have historically caused our agents many issues across different RPC providers and networks. Given that we now have a reasonable dataset on how much gas is needed for different contract methods, we are moving to custom-coding our gas limits using estimates from previous gas usage data.

<br>

## Standard EVM

### Home

Update

- Take average limit for one message (50k) and double base to 100k
- Each additional message dequeued is ~5k gas so double that cost to 10k per message
- `100k + (num_messages * 10k)`

Improper Update:

- Subset of `update` gas (`update` calls `improperUpdate`)
- Using same estimation as `update`
- `100k + (num_messages * 10k)`

DoubleUpdate:

- Signature check is ~`50k`
- Double that for two signature checks in double update `100k`
- Double total for safety to `200k`

<br>

### Replica

Update

- `70k` on average
- Double that to `140k` (constant)

Prove

- `100k` on average
- Double that to `200k` (constant since merkle proofs always same size)

Process

- Minimum `850k` required
- Double minimum to `1.7M`

ProveAndProcess

- `1.7M` for process
- `200k` for prove
- `1.9M` total

DoubleUpdate:

- Signature check is ~`50k`
- Double that for two signature checks in double update `100k`
- Double total for safety to `200k`

<br>

### XAppConnectionManager

UnenrollReplica:

- `60k` average
- Double that to `120k`

OwnerUnenrollReplica:

- Cheaper version of normal `unenrollReplica` without signature check
- Use same estimate of `120k`
