# Native console receive dependencies

Target: reference/openshores-full/OpenShores.exe, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
Static only; the native target was never run. All 11 distinct candidate virtual
readers have complete reviewed Ghidra MCP records in evidence/. Keep their
provider limitations and imported tail-jump warnings; signatures sometimes
misrecover the this/stream argument positions.

## Verified virtual slot map

Independent symbol-relative .rdata reads are retained in
`evidence/native-console-slot-bindings.json`. That original symbol-relative map
is now corroborated by the 22 complete constructor records indexed in
`evidence/native-console-constructors-index.json`. All 20 selected constructors
assign the mapped primary vptr at object offset zero. Secondary DpConsole
subobject vptrs are distinct and must not be substituted. Base/empty constructors
are retained too. Binding is not a runtime observation or a complete factory.
Slot +c0 yields shared DpStation readers, including the inherited no-op for
ScCommand. The same reader is shared by ordinary and spacecraft consoles.

| Class | Constructor-confirmed vptr | Receive address |
| --- | --- | --- |
| DpConsoleGame | 0x1423f9500 | 0x1410cfbe0 |
| DpConsolePower | 0x142407e10 | 0x141100210 |
| DpConsoleSurgery | 0x142425700 | 0x14111f660 |
| DpScTransporter | 0x142418970 | 0x141133c70 |
| DpScNav | 0x142481770 | 0x1410f4520 |
| DpConsoleEngineer | 0x142434850 | 0x1410cb0c0 |
| DpScSurgery | 0x1423d7fe0 | 0x14111f660 |
| DpScHelm | 0x14248f390 | 0x1410d9ba0 |
| DpConsoleHelm | 0x1423f9650 | 0x1410d9ba0 |
| DpScShield | 0x1423cd810 | 0x141117c30 |
| DpScWeapon | 0x1423cd940 | 0x1411412e0 |
| DpConsoleShield | 0x1424186d0 | 0x141117c30 |
| DpScSensor | 0x1423cd6e0 | 0x141108bc0 |
| DpConsoleWeapon | 0x142418820 | 0x1411412e0 |
| DpScCommand | 0x1423d7eb0 | 0x1410bdc60 |
| DpScEngineer | 0x1423e84a0 | 0x1410cb0c0 |
| DpScPower | 0x142499010 | 0x141100210 |
| DpConsoleNav | 0x1423e8290 | 0x1410f4520 |
| DpConsoleTransporter | 0x142452f50 | 0x141133c70 |
| DpConsoleSensor | 0x142418580 | 0x141108bc0 |

## Retained reader records

Each file below is prefixed native-console-reader- and ends .json in evidence/.

| Address | Evidence ID |
| --- | --- |
| 0x1410cfbe0 | ev_50573258db5edcd4734369e81c8457632937c3c3ea34baaa28c877ca19bcee03 |
| 0x141100210 | ev_880a82a01a26de5cf28c54dc30fdd599f667049315c9176359aefd7cf200adb7 |
| 0x14111f660 | ev_439c1a1d8088114cb9ee21b111dafda0cde0772f66f1e0c1eea5daea2d46a351 |
| 0x141133c70 | ev_446a47064f505456f9df66b7946c668524b7b5cc11a1fdf1e8700b9d8c6a5de2 |
| 0x1410f4520 | ev_6e9db818765bcf44d5c3c78cd59d8ebb30d06315ea3ec239331ca1c6388e93cd |
| 0x1410cb0c0 | ev_b910cf12b5b7e63a2febba1cccf4b7b7a223bbe13085f4126eb06f7dd8b60e1a |
| 0x1410d9ba0 | ev_9d946d76f1e47a62aff7e83ed31d09f88a4db53c1d62ef9ff7f356b802d9e9d7 |
| 0x141117c30 | ev_213324cdabc7862310d8141223e1778605345b8e686e8c48932f014b972ce238 |
| 0x1411412e0 | ev_cdf09507aad41b85cf7eadbf9dec91dc69b1830dd9665918412ca6a4eef2a8fa |
| 0x141108bc0 | ev_675d5abc4d042083cc599aa13c619c38f33708b7b65f87bbb3d31b94c3d3b432 |
| 0x1410bdc60 | ev_b6aa0b5c4aa98e2c5ae663922afedec35c28cebc41a8a37049066e2fab313d60 |

## Reviewed boundaries

- Base station function 1410bdc60 is a no-op. Ghidra selected the aliased
  Refresh symbol; GNU direct call labels identify Rx at the same address.
- Game: byte flags; bit1 reads byte14, otherwise writes 0x22; bit2 reads
  short16, otherwise writes zero. No other flag adds wire fields.
- Shield: six float overloads at14..28, then double30. Destination floats
  still narrow under a DoublePrecision wire profile.
- Engineer and Helm use packed status bytes plus scalar fields; their packed
  expressions and imported overloads are now GNU-corroborated below.
- Power's sparse eight-entry loops and ignored byte branch are implemented;
  see native-station-collections.md for exact ordering and retained fields.
- Surgery receives a real ConditionList, short38, Qt QList<QPoint>, short14,
  byte16, short18, QString20, short28. QPoint/container framing is now resolved
  and this whole station body is implemented in native-station-collections.md.
- Nav receives packed flags, bytes, three float vectors, IDs and a nested
  DeWaypointRoute. Both are now implemented with reviewed route evidence and
  assembly corroboration in native-station-collections.md.
- Sensor/Weapon/Transporter create a typed AuSensorBlip and invoke +158,
  receive scalar/vector fields and a real AuSensorBlipList::RxMin. Their
  records also include capsule/environment-dependent local sensor augmentation
  after wire reads. Those effects must not be confused with extra packet fields
  or silently replaced by a no-op in a claimed complete native integration.

Rust implements Engineer, Helm, Game, Shield, Surgery, Nav and Power bodies, plus
the Sensor/Weapon/Transporter wire stages in native-station-sensors.md and
Creature's pose wire block. Local capsule/environment sensor augmentation is
still outstanding. These are not a complete console dispatcher or
Creature receiver. Game tests cover all flags; Shield tests cover all explicit
float profiles, narrowing/non-finite values and exact suffix boundaries. Both
reject truncated prefixes atomically as a Rust safety policy.

## Matching-console control flow

Bounded independent GNU evidence is retained in
`evidence/native-creature-pose-console-gnu.txt`. At 14099b694 the IsConsolePose
result is saved in edi. On matching console type, context +468 is fetched.
Non-null invokes +c0; null clears edi at 14099bcc7. Both paths still read the
console's +e0 station field, compare +f0 type and may restore client settings.
At 14099ba54, edi controls fallback: nonzero continues after console handling;
zero re-enters the temporary-console context check. This is not the undefined
return value of a void station Rx reader. If the second context check is also
null, no console body is consumed. Exact context lookup and base-station
initialization/application still need reconstruction before composing this
with concrete Creature reception.

## Engineer and Helm wire fields

Independent GNU corroboration: evidence/native-station-controls-gnu.txt.
Both start with a flags byte. Low-bit direction is 1 when mask01 is set,
otherwise ff when mask02 is set, otherwise zero. Mode is 1 when mask04 is
set, otherwise 2 when mask08 is set, otherwise zero. These branches have
precedence and are not independent booleans.

Engineer then reads packed byte B: field20 gets B&3 and field24 gets B>>2,
both widened to u32. Next wire bytes assign offsets 17,1e,1f,14,1d,15,28 in
that order. Mask20 receives short2a; absent writes zero. Flags also write
field1c from bit4 and field29 from bit6. Direction is stored at16, mode at18.

Helm stores direction15/mode18, bit4→1c and bit5→24. Flags' top two bits map
00→0, 01→1, 10→7, 11→39 at field20. Next wire bytes assign 1d,14,16,25,
then short26. These are raw offsets/values, not inferred controls semantics.

Rust readers return fresh values atomically on errors, explicitly not native
failed-stream partial mutation. Tests cover all 65,536 Engineer flag/B pairs,
all Helm flags, unsigned widening, raw ff state, precedence, high-bit shorts,
field order, every truncated layout prefix and exact following-field alignment.
