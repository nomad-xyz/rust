## Processor Agent

The processor proves the validity of pending messages and sends them to end recipients.

It is an off-chain actor that does the following:

- Observe the home
- Maintain local merkle tree with all leaves
- Observe 1 or more replicas
- Maintain list of messages corresponding to each leaf
- Generate and submit merkle proofs for pending (unproven) messages
- Dispatch proven messages to end recipients
