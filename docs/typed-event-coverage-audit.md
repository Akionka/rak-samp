# Typed Event Coverage Audit

This audit compares `rak_rs_plugin_api`'s typed helpers with the public
SAMP.Lua event catalog. It measures convenience-schema coverage, not the
host's ability to observe traffic: raw inbound and outbound packet and RPC
subscriptions already cover all four directions.

| Surface | Typed helpers | Reference entries | Remaining |
| --- | ---: | ---: | ---: |
| Incoming RPCs | 139 | 139 | 0 |
| Outgoing RPCs | 33 | 33 | 0 |
| Packet/sync events | 28 | 28 directional entries | 0 |

These counts include the complete outgoing RPC catalog, all R1 inbound RPC
layouts, and authentication/connection/synchronization packet helpers. They
are an R1 compatibility statement, not a compatibility guarantee for another
detected SA-MP build.

## Recommended order

All cataloged R1 layouts are implemented. The final batch adds game
initialization and class/spawn state, streamed players and vehicles, compressed
3D text, object creation and mixed materials, menus/textdraws, animation
branches, attached objects, and score/ping collections. `WorldPlayerAdd` keeps
all eleven R1 weapon-skill levels after the player fields.

## Packet and sync helpers

`events::packet` provides a byte-by-byte wire layer for outgoing
authentication, RCON, stats, weapons, player, vehicle, passenger, aim,
unoccupied, trailer, bullet, and spectator packets. Incoming authentication and
connection lifecycle packets are typed alongside remote aim, bullet,
unoccupied, trailer, passenger, compressed player and vehicle sync, and marker
sync. The three variable layouts use exact bit lengths, normalized quaternions,
compressed velocities, and optional branches. R1 markers use signed `i16`
coordinates. Decoders reject trailing semantic bits and unbounded collections;
the R1 marker packet consumes its terminal sub-byte transport padding.
Uncertain text remains bytes and no payload is logged.

## Evidence

The names and candidate wire layouts were compared with the public
[SAMP.Lua event catalog](https://github.com/THE-FYP/SAMP.Lua/blob/c0f2de815425b20615f93816f36372d3a03110f2/samp/events.lua)
and [synchronization definitions](https://github.com/THE-FYP/SAMP.Lua/blob/c0f2de815425b20615f93816f36372d3a03110f2/samp/synchronization.lua).
The numeric RPC catalog was cross-checked against the public
[SA-MP RPC list](https://github.com/Brunoo16/samp-packet-list/wiki/RPC-List).
RakLua remains useful for its raw dispatcher model, but does not supply this
typed-schema catalog.
