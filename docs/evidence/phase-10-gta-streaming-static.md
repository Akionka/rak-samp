# Phase 10 GTA `CStreaming` static evidence

Date: 2026-08-30
Target: GTA San Andreas 1.0 US, Windows x86
Executable SHA-256: `A559AA772FD136379155EFA71F00C47AAD34BBFEAE6196B0FE1047D0645CBD26`

## Verified native facts

Exact-executable inspection confirms these cdecl entries and data fields:

| Fact | Address/offset | Evidence grade |
| --- | ---: | --- |
| `CStreaming::RequestModel(int, int)` | `0x4087E0` | A |
| `CStreaming::LoadAllRequestedModels(bool)` entry | `0x40EA10` | A for entry only |
| `CStreaming::RemoveModel(int)` | `0x4089A0` | A |
| `CStreaming::SetModelIsDeletable(int)` | `0x409C10` | A |
| `CStreaming::SetModelTxdIsDeletable(int)` | `0x409C70` | A |
| streaming-info array base | `0x8E4CC0` | A |
| streaming-info stride | `0x14` | A |
| flags field | `+0x06` | A |
| load-state field | `+0x10` | A |

`RequestModel` accepts a model ID and shared flag bits. `RemoveModel` reads the
same state record and performs native unlink/unload work. The exact code contains
a `0x4E20` model threshold, but this does not prove a safe public bound because
the pinned reference declares 26316 records and routes IDs above the threshold
differently.

The pinned reference identifies flags `GAME_REQUIRED=0x02`,
`MISSION_REQUIRED=0x04`, `KEEP_IN_MEMORY=0x08`,
`PRIORITY_REQUEST=0x10`, and `LOADING_SCENE=0x20`. Exact instructions confirm
that the deletable functions clear shared bits. The observed load-state values
are `NOT_LOADED=0`, `LOADED=1`, `REQUESTED=2`, `CHANNELED=3`, and
`FINISHING=4`.

## Publication decision

No public lifecycle API is published. The exact `LoadAllRequestedModels` entry
continues through packed jump islands and I/O/channel code; static evidence does
not prove that it is non-blocking, reentrant, or safe in the Host post-process
command pump. The valid model range and behavior above model 19999 also remain
unresolved.

Native request flags are shared bits, not ownership counters. A Host-local
reference count cannot prove that GTA or another plugin does not own the same
bit. `SetModelIsDeletable`, `SetModelTxdIsDeletable`, and `RemoveModel` can
therefore release or unload state owned elsewhere. Calling one as failure
cleanup would violate the ownership boundary.

The narrow future slice is queued request plus read-only loaded observation. It
still requires live phase/non-blocking evidence, a verified public model bound,
and an ownership policy that does not promise release. Full
request/load/release remains unsupported until sole-owner cleanup is proven.
