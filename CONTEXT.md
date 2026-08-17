# SA-MP Native Client

This context defines the profile-backed native client bridge and the owned
values it exposes to independently loaded plugins.

## Language

**ARGB colour**:
The public and cache colour value with alpha in the most significant byte.
_Avoid_: raw colour, native colour

**Native RGBA colour**:
The colour representation stored by the SA-MP native text-label record.
_Avoid_: ARGB, public colour
