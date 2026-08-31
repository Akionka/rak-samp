# Publish a separate SA-MP text-label service

Phase 11 publishes text-label snapshots and mutations as exact-version service ID `0x0000_2002` instead of appending fields to frozen `SampServiceV1` or recreating the legacy monolithic table. `SampTextLabelServiceV1` returns Core receipt IDs; automatic creation carries its allocated `TextLabelId` in `CommandCompletionV1.value0`, which keeps polling, waiting, release, wait rejection, and shutdown behavior under the common Core contract while preserving a typed safe facade result.
