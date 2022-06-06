## Updater Agent

The updater is responsible for signing attestations of new roots.

It is an off-chain actor that does the following:

- Observe the home chain contract
- Sign attestations to new roots
- Publish the signed attestation to the home chain
