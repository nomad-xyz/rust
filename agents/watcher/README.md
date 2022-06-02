## Watcher Agent

The watcher observes the Updater's interactions with the Home contract (by watching the Home contract) and reacts to malicious or faulty attestations. It also observes any number of replicas to ensure the Updater does not bypass the Home and go straight to a replica.

It is an off-chain actor that does the following:

- Observe the home
- Observe 1 or more replicas
- Maintain a DB of seen updates
- Submit double-update proofs
- Submit invalid update proofs
- If configured, issue an emergency halt transaction

For more info on how to run a watcher, refer to our [running watcher guide](./RUNNING-WATCHER.md) and our [general guide on running agents](../RUNNING-AGENTS.md).
