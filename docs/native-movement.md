# Native movement-speed modifier

`native_movement_speed_modifier` reconstructs the actual float return of
DaCreature::MovementSpeedMod at1418f4f50. The saved REA record
`evidence/native-creature-movement-speed.json` incorrectly recovers a void return;
do not derive numeric behavior from that pseudocode. Complete independent GNU
[native-movement-speed-gnu.txt](evidence/native-movement-speed-gnu.txt) shows
the result copied from xmm6 to xmm0 before return and the full arithmetic paths.
`just test-native-movement` corroborates those static anchors and PE constants.
No original executable was run and no analysis database was reimported.

The implementation samples actual selected-body HP (Dp priority) and growth494.
Nonpositive HP returns1 without reading capacities or burden. Positive HP with
nonzero growth uses signed32 arithmetic and truncating division by GrowthTime,
then signed16 truncation; a zero short result becomes1. Zero GrowthTime is a
Rust InvalidData safety error instead of the native integer divide trap.

Dp capacity is25*(MaxHP/10+1+(byte29>>4)), using signed truncating division.
AuAnimalia::BurdenMax is a direct load of PE float43480000: exactly200, not a
guessed species-dependent observation. The first actual burden sample at or
above capacity returns0. Otherwise a distinct second sample is used in
(capacity-second)/capacity. A nonpositive ordered result returns positive zero;
unordered NaNs continue. The helper then samples actual MaxHP and, if it exceeds
signed16(HP*2), multiplies by float(signed16(HP*2))/float(MaxHP).
The short wrap can produce a negative modifier; it is not silently clamped.
Each native float stage remains f32, with no fusion or f64 promotion.

The required context supplies actual GrowthTime, MaxHP, Dp byte29 and virtual970
burden observations only when the native branch needs them. Person's vtable
binds970 to DaPerson::Burden at1419466b0; its weight/catalog/model dependencies
remain unresolved, not replaced with a zero weight. Production context must
return Unsupported for missing required values. Missing both models must also
fail, not select a fabricated body.

Tests cover growth bytes, zero division safety, signed-short boundaries,
zero-to-one adjustment, two differing burden observations, capacity gates,
NaN/infinite/negative burden values and explicit context errors. Full native
Person packet fixtures now compose this real helper through head tilt using the
actual fixture HP at the scalar stage (before the packet's HP update), removing
the earlier constant-speed stub. Fixture animation/environment observations are
still synthetic; model generation/weight dependencies and client presentation
remain unfinished.

Focused remaining model-getter disassemblies are retained in
`evidence/native-movement-dependencies-gnu.txt`: Dp MaxHP1412c60a0 depends on
model-part volume/cache and body-byte flags; Animalia MaxHP1411eb820 depends on
chromatid volume and fly-capable wings; its GrowthTime1411ebb40 calls MaxHP.
The HP/growth arithmetic and ordered provider operations are now implemented;
see native-health.md. Actual model/chromatid volume and wing generation remain
required, not fabricated. Reuse these complete records rather than reanalyzing
the executable for each helper.
