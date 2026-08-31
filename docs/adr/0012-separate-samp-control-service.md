# Separate SA-MP control from state and network traffic

Phase 11 publishes queued game-state writes, replication send rates, connect, and disconnect under exact-version service ID `0x0000_2003`. These operations change client lifecycle or scheduling state, so they do not extend frozen read-oriented `SampServiceV1` or Packet/RPC-oriented `SampNetServiceV1`; all return common Core receipts and retain the existing global Host command order.
