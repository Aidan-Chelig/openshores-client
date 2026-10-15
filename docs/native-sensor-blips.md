# Native sensor blip receive dependencies

Static target: reference/openshores-full/OpenShores.exe, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
No executable was run or public server contacted.

Complete reviewed factory record: evidence/native-sensor-blip-factory.json,
`ev_38e1aa95c6d308dd2772c54f9bb9a5c7c8cb6ddd8abf7f6ac632b79b2fdb4f25`.
Types 1..43 allocate the concrete objects below; all other values return null.
Factory-written vptrs plus independently read .rdata slots are retained in
`evidence/native-sensor-blip-slot-bindings.json`. The full body virtual at
+158 differs from minimal reception at +190 (400 decimal).

Complete reviewed minimal-list record: evidence/native-sensor-blip-list-min.json,
`ev_f1c119d92972a6d9676f12b4f86987b64639fa152496a6140358dc395559af4d`.
Clear/destroy all old entries before receiving short count. Per iteration read
byte type, Create, receive +190 and append. **A null factory breaks out and
returns after diagnostics**, leaving the unprocessed remainder unread. This is
not the condition/chromatid unknown-type continuation rule. Count overload/loop
width is now independently corroborated below.

| Type | Factory class | Full reader | Minimal reader |
| --- | --- | --- | --- |
| 1 | AuBlipAircraft | 0x141191050 | 0x14118f670 |
| 2 | AuBlipAnimal | 0x141190890 | 0x1411906a0 |
| 3 | AuBlipAvatar | 0x141190a70 | 0x14118f670 |
| 4 | AuBlipBoat | 0x141191290 | 0x14118f670 |
| 5 | AuBlipCitizen | 0x141190750 | 0x14118f670 |
| 6 | AuBlipCrew | 0x141190930 | 0x14118f670 |
| 7 | AuBlipGasGiant | 0x141190130 | 0x14118ca70 |
| 8 | AuBlipGroundVehicle | 0x1411911d0 | 0x14118f670 |
| 9 | AuBlipTitan | 0x1411901d0 | 0x14118ca70 |
| 10 | AuBlipMoon | 0x14118fef0 | 0x14118ca70 |
| 11 | AuBlipOfficer | 0x141190b10 | 0x14118f670 |
| 12 | AuBlipPassenger | 0x1411909d0 | 0x14118f670 |
| 13 | AuBlipPlanet | 0x141190270 | 0x14118ca70 |
| 14 | AuBlipRing | 0x14118fcb0 | 0x14118ca70 |
| 15 | AuBlipRingSection | 0x14118fd40 | 0x14118ca70 |
| 16 | AuBlipSpacecraftHiding | 0x141192020 | 0x14118f670 |
| 17 | AuBlipStation | 0x141191ff0 | 0x14118f670 |
| 18 | AuBlipStationShielded | 0x141191f60 | 0x14118f670 |
| 19 | AuBlipSpaceship | 0x141192000 | 0x14118f670 |
| 20 | AuBlipSpaceshipShielded | 0x141191f90 | 0x14118f670 |
| 21 | AuBlipStarCompanion | 0x141190de0 | 0x14118ca70 |
| 22 | AuBlipStar | 0x141191350 | 0x14118ca70 |
| 23 | AuBlipStarship | 0x141192010 | 0x14118f670 |
| 24 | AuBlipStarshipShielded | 0x141191fc0 | 0x14118f670 |
| 25 | AuBlipTransporterSpacecraft | 0x141190c60 | 0x14118f670 |
| 26 | AuBlipTransporterStation | 0x141190bb0 | 0x14118f670 |
| 27 | AuBlipTroop | 0x1411907f0 | 0x14118f670 |
| 28 | AuBlipBlackHole | 0x141190d10 | 0x14118ca70 |
| 29 | AuBlipMissile | 0x1411914e0 | 0x14118f670 |
| 30 | AuBlipBeacon | 0x14118f5f0 | 0x14118ca70 |
| 31 | AuBlipAnomaly | 0x14118f580 | 0x14118ca70 |
| 32 | AuBlipBdBase | 0x141191d10 | 0x14118ff80 |
| 33 | AuBlipBdBaseHQ | 0x141191d10 | 0x14118ff80 |
| 34 | AuBlipBdCapitolCity | 0x141191d10 | 0x14118ff80 |
| 35 | AuBlipBdCapitolEmpire | 0x141191d10 | 0x14118ff80 |
| 36 | AuBlipBdCapitolSector | 0x141191d10 | 0x14118ff80 |
| 37 | AuBlipBdCity | 0x141191d10 | 0x14118ff80 |
| 38 | AuBlipTransporterBd | 0x141191110 | 0x14118f670 |
| 39 | AuBlipBdCapitolGalaxy | 0x141191d10 | 0x14118ff80 |
| 40 | AuBlipNeutronStar | 0x141190eb0 | 0x14118ca70 |
| 41 | AuBlipNeutronStarCompanion | 0x141190f80 | 0x14118ca70 |
| 42 | AuBlipAsteroid | 0x14118fe60 | 0x14118ca70 |
| 43 | AuBlipPlanetoid | 0x1411900a0 | 0x14118ca70 |

The 41 distinct reader results were obtained in the same warm process and are
now retained in evidence/ as native-sensor-blip-reader-<address>.json. Full and
minimal bodies are implemented as separate APIs. No opaque sensor skipping or
complete Creature receiver is claimed. Sensor/Weapon/Transporter station readers also have local
capsule/environment augmentation after wire reception, separately documented
in native-consoles.md.

## Minimal receive implementation

Reviewed complete records in evidence/:

- native-sensor-blip-reader-14118ca70.json:
  ev_dc7d7ac70de361caadf4f8ef85f02c8e505ad7a6321397a9094bf75f74dcaf05.
- native-sensor-blip-reader-14118f670.json:
  ev_bbd51e6f1a3fe6684b52c09263edf776ed76f7c874a00e966f72793a9ece9fec.
- native-sensor-blip-reader-1411906a0.json:
  ev_5b39abb6b37426279fcc55dbeff001677001e9bb2f94125eaa7a22712c336d31.
- native-sensor-blip-reader-14118ff80.json:
  ev_29a6a09180366db785ea9ad6837f82baf5c708ecf134baf97ca01d77a0cf1847.

Independent native-sensor-min-gnu.txt corroborates all four layouts and imported
overloads. Base: ID08 then three float-overload values widened to doubles at
10/18/20. Empire adds raw byte2c. Animal adds byte2c then packed byte B:
bit1→byte40 and inverse bit0→u32 at28. Bd adds byte2c then byte34. This is
minimal reception only: do not append full species/string/model suffixes.
The factory-written vptr's +190 slot selects the actual layout for all 43
types, including subclasses whose minimal layout differs from their full one.

The same GNU record at141199be9..141199c6c pins the count to a short overload,
zero-extended word, low-word decrement/store and zero test: all 16 count bits
are used. Its successful body dispatch is +190, not +158.

world::native_sensor_blip implements all minimal bodies and the real counted
replacement list. Unknown factory types stop the list and return a must-use
result reporting type, declared count and remaining unread entries. There is
no guessed skip and no condition-list-style continuation. Earlier completed
appends remain on a later Rust error; fresh body decoding itself is atomic.
These are Rust error policies, not Qt failed-stream/destructor parity. Counts
do not directly allocate storage; their 16-bit range bounds the list.

Fixtures cover all 256 factory bytes and all profiles, all Animal packed bytes,
high-bit IDs/raw bytes, signed zero/non-finite coordinates, every truncated
body prefix, all 43 types together, zero/high-bit counts, destructive clearing,
partial-list retention, explicit unknown-stop reporting and exact suffixes.
Local sensor augmentation and Creature/Person dispatch remain unfinished;
successful blip reception does not imply complete avatar reception.

## Full body reception

`NativeSensorBlipFull::read` implements fresh factory-created +158 bodies for
all 43 types. `evidence/native-sensor-full-index.json` indexes the 37 distinct
full factory reader records plus the spacecraft base and AuPoint helper by
Evidence ID. Complete envelopes retain provider limitations and tail-call
warnings. `evidence/native-sensor-full-gnu.txt` independently corroborates scalar
overloads, helper calls and tail jumps using exact GNU COFF symbol selection;
it is not REA evidence and does not describe runtime observation.

All full bodies begin with AuId/u32 at08 and three float-overload coordinates,
narrowed to f32 then widened to f64 at10/18/20. In the table, E means i32 at28
then raw byte2c, Q means nullable QString, F float overload, D double overload,
B boolean overload, byte means raw eight bits, and short preserves all 16 bits.
Offsets name observed native fields, not guessed gameplay meanings.

| Factory types | Ordered suffix after ID/position |
| --- | --- |
| 31 Anomaly | none |
| 30 Beacon | Q28 |
| 14 Ring | B28, F2c |
| 10 Moon, 15 RingSection, 43 Planetoid | B28, byte29 |
| 7 GasGiant, 9 Titan, 13 Planet | B28, byte29, byte2a |
| 42 Asteroid | short28, byte2a |
| 2 Animal | E, raw QByteArray species30, B40 |
| 3 Avatar, 5 Citizen, 6 Crew, 11 Officer, 12 Passenger, 27 Troop | E, raw QByteArray species30, Q40 |
| 21 StarCompanion, 22 Star, 28 BlackHole, 40 NeutronStar, 41 NeutronStarCompanion | bytes A/B, F2c; field28=A>>7, field32=B>>4, short30=((B<<8)\|A)&0x0f0f |
| 1 Aircraft, 4 Boat, 8 GroundVehicle | E, short S, byte31; short2e=S&0x7fff, byte30=0 |
| 29 Missile | E, unmasked short2e, byte30, byte31, byte32 |
| 25 TransporterSpacecraft, 26 TransporterStation | E, byte2d, short2e, Q30 |
| 38 TransporterBd | E, Q30, byte38, byte39, short3a |
| 32..37 and39 Bd subclasses | E, flags, i32 at30, byte34, i32 at38, Q40 only if flags&2, F48, byte4c, i32 at50; byte35=flags&1 |
| 16 Hiding, 17 Station, 19 Spaceship, 23 Starship | E, i32 at30, Q38, B40, B41, B42, i32 at44, Q48, F50, byte54, three D at58/60/68, i32 at70 |
| 18/20/24 shielded spacecraft | spacecraft suffix, then F74 |

Vehicle31, Missile32, Bd4c and Spacecraft54 replace zero with one, preserving
every other byte. Absent Bd Q40 retains the fresh factory shared-null default;
this fresh-value API does not emulate receiving into an existing full blip.
Null and empty QString/QByteArray values remain distinct. Species reuse the
existing raw AuSpecies reader, not DNA/chromosome decoding.

The spacecraft wrappers at141191ff0/2000/2010/2020 jump to141191e10. Its call
at141191f0b resolves to AuPoint141648040, whose three imported `rsERd` reads
establish double overloads. Retained base/point Evidence IDs respectively:
`ev_cebaa77b6b1d13d1216e6754a180608023667e61638bcfb9bc6bace5f48eca7c`
and `ev_323a7bba10ba410f5b92563e005e0f034a58c90472b174508fbfaa52c675e041`.
Under SinglePrecision these D reads consume f32 and widen; TypeWidths and
DoublePrecision retain f64. F destinations always narrow to f32. There is no
finite-value filtering in wire state; rendering safeguards are a separate stage.

The supplied Qt5Core.dll SHA-256 is
`3aad8885b1c7b2c0c5e2016095c8c51c3089c17a70269bc332cfe2c483b65ccc`.
Its exported bool reader (RVA265590, VA3956b5590) reads one byte and uses
compare-zero/setne at3956b55f8..5600: all nonzero bytes mean true. The GNU
record includes this complete successful branch and its failed-stream branch;
Rust errors do not claim Qt failed-stream mutation parity.

Fixtures assert every full type/profile and field, every truncated prefix,
suffix alignment through all 43 types, all boolean/zero-repair/Bd-flag bytes,
all 65536 star-byte pairs, raw non-finite coordinates/signed zero, Unicode,
null/empty species and names, unknown factories and malformed input budgets.
Fresh body errors leave the parent input unchanged; strict UTF-16 and the
1MiB species budget are explicit Rust safety policies, not native limits.
