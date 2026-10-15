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
width still needs independent corroboration before implementation.

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

The 41 distinct reader results are saved under target/avatar-analysis-session-2
as sensor-blip-reader-<address>.json, all successful in the same warm process.
Their bodies still require full review and nested-helper reconstruction before
Rust implementation. No opaque sensor payload skipping or complete Creature
receiver is claimed. Sensor/Weapon/Transporter station readers also have local
capsule/environment augmentation after wire reception, separately documented
in native-consoles.md.
