# In-game validation plugin

This ASI records packet/RPC IDs and runs local rewrite-and-block checks without
logging payloads. Optional marker files enable server-bound sends and
coordinated shutdown. It also exposes the shutdown contract used by the
[validation unload manager](../validation_unloader).

Deploy and run it using [VALIDATION.md](../../VALIDATION.md).
