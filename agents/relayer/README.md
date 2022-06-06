## Relayer Agent

The relayer forwards updates from the home to one or more replicas.

It is an off-chain actor that does the following:

- Observe the home
- Observe 1 or more replicas
- Polls home for new signed updates (since replica's current root) and submits them to replica
- Polls replica for confirmable updates (that have passed their optimistic time window) and confirms if available (updating replica's current root)
