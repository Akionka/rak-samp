# Keep process-module probes out of core SA-MP services

Phase 11 does not migrate legacy SA-MP/SAMPFUNCS module-presence probes or the SAMPFUNCS console export into the core `samp` crate. New plugins use Host status and exact service discovery; optional external-module behavior remains Phase 12 `sampfuncs-compat` scope, preventing core APIs from promising compatibility-only process state.
