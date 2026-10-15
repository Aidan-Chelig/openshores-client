# Native Surgery, Navigation and Power reception

Static target/profile match native-consoles.md. No native executable or public
server was used. Existing station reader records are indexed there. New complete
REA records in evidence/:

- native-qpoint-list-reader.json:
  ev_4d0d071b8755630583b13ebe7ff276ecc304f5d3c6e53368843dd9a033f71ea0.
- native-waypoint-route-reader.json:
  ev_90e624dcb28e1a8a6a073281142e5eba92973222b2f20d19bd9afd061a50c902.

Independent bounded GNU corroboration is retained in
evidence/native-station-collections-gnu.txt. It includes the supplied Qt5Core.dll
(SHA-256 3aad8885b1c7b2c0c5e2016095c8c51c3089c17a70269bc332cfe2c483b65ccc).
QPoint export ordinal6308/RVA5bc70 tests stream version against 1. Version 7
receives signed i32 at00 then signed i32 at04; only version 1 uses shorts.
The called int reader is independently bound by export ordinal448/RVA265360.

## Surgery

ConditionList48 → short38 → QList<QPoint>40 → short14 → byte16 → short18 →
QString20 → short28. The real condition decoder, including Pregnant/DNA, is
used. The Qt list receives a 32-bit count followed by x/y signed i32 pairs;
it is not a count of bytes or another 16-bit condition count. Native list
clearing/status handling is in the full record. Fresh Rust decoding is atomic
on error and rejects counts above 20,000 before allocation; these are explicit
Rust safety policies, not native status/reserve behavior.

## Navigation and route

Flags assign 14 as four packed byte states [bit6,bit0,bit1,bit2], bit3 to18,
bits4..5 to5c. Then bytes widen to u32 at1c,64,20; three float-overload values
widen to vector28, three to vector40; IDs68 then58; byte6c, byte→u32 at60;
three float-overload values widen to vector70; finally DeWaypointRoute90.

Route receives a short count **before clearing existing entries**. Each entry
receives two 32-bit IDs then three double-overload values. GNU at141043575
explicitly copies the first two doubles into the waypoint's vector before
the third assignment; Ghidra's recovered locals obscure that copy. Do not
replace the first two values with the temporary initializer's zeros. The
low-word decrement uses every 16 count bits, including high-bit counts.

NativeWaypointRoute preserves completed entries on later Rust errors; a
truncated count retains the old route. No preallocation uses count directly.
NativeStationNav returns a fresh atomic wire value. Profiles remain explicit:
station vectors narrow through float even on DoublePrecision wire; route
positions retain doubles except when SinglePrecision wire widens f32.

## Power

Receive signed i32 at68, then flags. In wire order:

- mask10: eight bytes c0..c7;
- mask08: one byte received but not stored;
- mask04: byte6c;
- mask02: eight entries, each byte c8+i, byte→u32 at14+4i, i32 at94+4i;
- mask01: eight entries, each byte b8+i, byte→u32 at3c+4i, byte34+i, i32 at74+4i.

Other flag bits have no wire fields. Omitted groups retain existing state.
NativeStationPower uses Option for unobserved fields rather than inventing
native constructor defaults. Each successful scalar assignment persists after
a later Rust error; this does not emulate Qt failed-stream continuation.

Tests exercise real nested conditions and nullable strings, signed/high-bit
points, empty/max/oversize point lists, every station flag/profile combination,
raw non-finite float data, ID ordering, route high-bit counts, omitted-group
retention, partial assignments and truncated prefixes/suffix boundaries. These
are wire/state components, not complete Creature dispatch or avatar rendering.
Sensor/Weapon/Transporter still require the blip readers in native-sensor-blips.md.
