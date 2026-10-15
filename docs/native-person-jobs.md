# Native Person jobs

Target: reference/openshores-full/OpenShores.exe, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
No supplied executable was run and no public server was contacted.

## Factory and list evidence

Complete retained AuJobList REA record: `evidence/native-job-list.json`, Evidence
ID `ev_0424bab989018ab6c16b407cc55aea994f54515f6bdcb34025a27f75a269abfd`.
Its limitations are preserved. Independent full factory assembly and raw vptr
windows are in native-job-factory-gnu.txt; decoded27-case index is
native-job-factory-bindings.json (both under evidence/). The signed-relative
table at1421b6c28 maps types0..26; values outside that range return null.
Factory blocks directly assign each primary vptr. Raw+70 selects its reader,
raw+50 its IsObsolete predicate. The static verifier tests/native_jobs.py checks
every entry against current immutable bytes and exact GNU constructor blocks.

Native type0 is Assassin,20 is InterdictSystem,24 is Story. The source-server's
Story type31 is **not** a valid native Story job and is never source-decoded.
Types1/2/3 are BountyHunter variants whose +50 returns true; every other +50
returns false. Obsolete jobs must still have their real bodies received before
being discarded, not skipped based on type. Those three bodies are currently
Unsupported, so no unimplemented filtering is presented as successful reception.

NativeJobList clears **before** reading the16-bit count; its decrement-to-zero
loop uses all16 bits, including high-bit counts. A null factory consumes that
type then warns/returns immediately, leaving remaining job bytes untouched.
NativeJobListReceive surfaces the stop and remaining declared count. The Person
suffix continues its own following native fields, exactly as the enclosing
native function does; it does not guess where unreceived job bodies would end.
The nested result remains must-use through Person/body results. Known but
unimplemented valid bodies instead fail Unsupported, preserving prior appends.

## Implemented bodies

Exact full GNU records are retained in native-job-readers-gnu.txt. NativeJob
reads fresh values atomically on Rust failure, not claiming Qt malformed-stream
or partial-native-storage parity. The initial three classes below are
non-obsolete; additional implemented families are listed in the next section.

- Assassin1409069a0: timestamp08, QString10, AuId18, QString20, signed32 at28.
- InterdictSystem140906900: timestamp08, QString10, AuId1c, QString20, signed32
  at18 (different destination offsets, despite the same wire scalar sequence).
- Story140906af0: timestamp08, QString10, QString18, one ignored byte, signed32
  at20, QString28. The ignored byte accepts every value and is exposed only as
  a wire observation; it is not an invented native stored field or tag branch.

Other exact-symbol reader results were batched and retained separately in
native-job-remaining-readers-gnu.txt. The additional families below have now
been reviewed/implemented; the rest remain unsupported. No Ghidra import was
needed for these small static reconstructions.

## Expanded delivery/passenger/travel families

Current supported factory types are0,6..24,26 (21 of27), all with verified
non-obsolete predicates. Types1/2/3 (BountyHunter),4/5 (CommandShip) and25
(TransferPrisoners) remain Unsupported. Timestamp limitations below still apply.

- Types6/7/8/10/11/12/13 bind DeliverCargo1409079f0: timestamp08, short→unsigned
  zero-extended32 field10, ID14, QString18, signed32 field20, byte24 (zero→1),
  ignored byte, signed32 field28, QString30, ID38, QString40.
- Type9 binds DeliverCargoManufacture140907af0: calls the entire Cargo reader,
  then reads another short and zero-extends its16 bits into field48. It does
  not read another timestamp or reinitialize the common fields.
- Types14/15 bind DeliverFood1409078f0: timestamp08, ID10, QString18, byte→u32
  field20, signed32 field24, byte28 (zero→1), ignored byte, signed32 field2c,
  QString30, ID38, QString40. Cargo's short is not substituted for Food's byte.
- Types16/17/18 bind DeliverPassengers140907330: timestamp08, ID10, QString18,
  signed32 fields20/24, QString28, ID30, QString38.
- Types21/23 bind PickUpPassengers140907260: timestamp08, ID10, QString18,
  signed32 field20, QString28, ID30, QString38, QString40. Unlike delivery,
  this has no field24 reader and does have the extra string40.
- Type22 binds PickUpPassengersOverpopulated140907590: the pickup wire sequence
  followed by ID48 via the tail-jumped AuId reader.
- Type19 binds DestroyEnemyStation140906a40: timestamp08, QString10, ID18,
  QString20, ID28, QString30 (last QString is tail-jumped, not absent).
- Type26 binds VisitSystem140906ba0: timestamp08, ID10, signed32 field14,
  QString18, ID20, QString28 (also a tail-jumped final QString).

Separate typed structs preserve distinct receive destinations and raw IDs.
Ignored bytes are wire observations, not invented stored native fields.
All raw byte values, zero→one normalization, both short zero extensions across
every16-bit value, exact string/ID tails, every truncated body prefix, mixed
lists and Person suffix continuation have offline fixtures. The remaining
six bodies are not treated as empty/obsolete without real decoding.

## Timestamp profile

The supplied Qt5Core.dll is independently identified by SHA-256
`3aad8885b1c7b2c0c5e2016095c8c51c3089c17a70269bc332cfe2c483b65ccc`.
Its exported QDateTime stream reader has RVA f81c0 (base395450000). The bounded
entry/Qt7 branch is retained in native-job-datetime-qt-gnu.txt; it is not a full
DLL analysis or runtime oracle. Version7 reads32-bit Julian day,32-bit time
milliseconds, then a private signed time-spec byte. Encoded2 becomes Qt::UTC.
The native job then calls setTimeSpec(UTC) before storing toMSecsSinceEpoch.

For valid UTC input, timestamp08 is `(day - 2440588) * 86400000 + milliseconds`.
The implementation explicitly rejects non-UTC specs, zero/invalid Julian day,
and time milliseconds outside a day. No local-time/DST or invalid-QDateTime
return behavior is guessed. This is a Qt7-only API, not arbitrary stream version
support. Local/invalid datetime semantics remain required follow-up work.

## Verification and remaining goal

Fixtures cover all16 count bits, every null factory byte, all known unsupported
body types, every Story ignored byte, the actual three bodies and truncation
prefixes, UTC epoch vectors, explicit unsupported datetime profiles, clear-before-
count, retained prior appends, and real Person suffix composition/late fields.
The independent static verifier checks all27 factory/reader/discard bindings.

Remaining job bodies/local-time semantics, item inventory/weight, concrete
Creature runtime context, inherited native construction/dispatch and actual
avatar presentation still belong to the unchanged end-to-end goal. These job
fixtures do not establish complete native Person packet reception.

`just check` passes410 headless and431 client library tests,9 CLI tests per
profile,1 graphical-entry CLI test, formatting and strict client-feature
all-target Clippy. `just test-native-jobs` passes the independent static binding
and datetime-anchor verification without compiling or executing the native target.

After the family extension, `just check` passes414 headless and435 client
library tests, CLI tests, formatting and strict client-feature all-target Clippy.
