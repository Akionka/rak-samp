# Publish textdraws as a separate exact-version service

Phase 11 publishes owned textdraw snapshots and queued create, delete, position, style, letter, proportional, shadow, outline, box, alignment, text, and model-style actions under service ID `0x0000_2007`. `SampTextdrawServiceV1` preserves bounded pool IDs, fixed copied strings, and the global Host command order without extending frozen `SampServiceV1`; every mutation returns a Core receipt.
