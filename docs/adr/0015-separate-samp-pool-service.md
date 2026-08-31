# Publish SA-MP pool mappings as copied values

Phase 11 publishes object, pickup, vehicle, and player-ped forward/reverse mappings plus owned gangzone snapshots under exact-version service ID `0x0000_2006`. `SampPoolServiceV1` keeps live pool identity in the Host, crosses the ABI only with scalar handles, checked IDs, and copied records, and avoids extending frozen `SampServiceV1`; opaque native addresses remain unavailable.
