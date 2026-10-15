# Native Person reception

Selected patched target: `reference/openshores-full/OpenShores.exe`, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
Complete REA `_ZN8DaPerson8RxUpdateER11QDataStreamb` decompilation is retained in
`evidence/native-person-update.json`, evidence
`ev_7c21d6bd5041896517ec6cd7baf6271ed190525ee97835989f62d203fb6f772a`.
The full pseudocode has been reviewed. This is static Ghidra 12.1.2 evidence,
not execution; its type-propagation warning requires scalar-width corroboration.

Independent GNU assembly of the entry at `0x140a42db0` establishes that Person
first calls **DaSentient::RxUpdate** at `0x140b22720`, forwarding its is-new
argument. It then reads a Person flags byte. It does not jump straight from the
common atom base prefix to Person fields or invoke the source-server Person
decoder. Leading conditional branches include an additional byte when bit
`0x80` is set, before the low-bit Person fields. The bounded entry review through
`0x140a42edb` also shows explicit field `0x838` zeroing and state-bit writes at
`0x1130`; exact suffix behavior is now implemented/documented in
native-person-suffix.md, with remaining nested-reader boundaries explicit.

## Person suffix order

After Sentient reception, read flags A; if A has `0x80`, read flags B immediately,
otherwise B is zero. The suffix then proceeds in this order:

- A `0x01`: ID/string, ID/string, `AuGlobeLLF`, ID/bool/string.
- A `0x02`: `AuHitNotify`.
- A `0x04`: packed short; low 15 bits update field `0x660` and restart its
  timer only on change, high bit replaces byte `0x839`.
- A `0x08`: read byte C and assign `0x838 = C & 0x3f`; otherwise C and
  `0x838` are explicitly zero (not sparse retention).
- A `0x10`: packed scalar plus integer `0xf80`; derive byte `0xf84` from
  bit 7 and field `0xf88` from bits 3–4 of the packed scalar's low byte.
- A `0x20`: byte `0xfa8`, packed byte D; assign `0xfa9 = D & 0x0f`, clear
  content list `0xfa0`, optionally read counted `AuItemContentId` entries
  when D has `0x80`, then read the scalar whose low byte becomes `0xfb0`.
- A `0x40`: receive `AuGear` under a write lock, then recompute weight `0x13a8`.
- Extended B: `0x01` domain; `0x04` job list; `0x08` integer `0x10d0` and
  short `0xf58`; `0x10` floating field `0x1408`; `0x20` floating fields
  `0xf60`/`0xf68`. B `0x02` has no observed reader here.
- B `0x40` sets state bit `0x200000` without an observed corresponding clear.
  State bit `0x8000` is replaced by B `0x80`, and cleared without extension.
- C `0x80`: `AuItemHash`, AFTER the extended fields.
- A `0x01`: trailing scalar whose low byte replaces `0x10e8`, not adjacent
  to the earlier A `0x01` identity fields on the wire.
- C `0x40`: final packed scalar; low nibble becomes `0x83d`, high `0x83b`.

Independent complete exact-symbol assembly is now retained in
`evidence/native-person-update-gnu.txt`. The `signed*`/misidentified collection
overloads above are **byte reads** (`QDataStreamrsERa`), not four-byte integers:
A10's packed value, A20's optional count and each content ID, A20's final scalar,
the trailing A01 scalar and C40's final packed scalar. A10's f80 and B08's10d0
are signed32; B08's f58 is signed16. B10/B20 use the **double** overload
`QDataStreamrsERd`; their actual wire widths still depend on the explicitly
selected stream precision. This corroboration resolves those former scalar
ambiguities, without claiming a complete Person receiver.

Gear's item fixup/weight semantics, polymorphic job bodies, item hash,
full inherited construction/application and semantic field names remain open.
Do not implement an opaque skipped subreader or borrow the source-server body.

Next: reconstruct its Sentient/Creature layers
and conditional subreaders; establish constructor/default and body state;
extend concrete native object dispatch without assuming Item defaults. No
native execution or live-server testing is involved in this investigation.

The default constructor is now retained as
`evidence/native-person-default-constructor.json`, Evidence ID
`ev_372afccd2f5bf68461eece6a46942f116f301dfb259ce0e34ef654f65a79f3b1`.
It calls DaSentient first; this record is not yet a complete Rust constructor.
The explicit receive-visible suffix subset is now available; see
[native-person-construction.md](native-person-construction.md). It establishes
fresh masked-update priors without inventing the remaining runtime object.
The ordered Unit/Creature stage exists with required application context hooks;
no concrete Person construction/dispatch or presentation is implied by those
component fixtures. See native-creature.md for the exact remaining boundary.

Gear wire composition is now implemented for the eight established item types;
see native-person-gear.md. Commodity2 replacement, remaining item types and
Person's required weight recomputation remain explicitly unsupported/unfinished.
Null slots are diagnosed and skipped without truncating the counted loop.

Person's inventory stage now composes eight real item families, native obsolete
filtering, duplicate commodity retention and verified null-factory continuation.
Other valid item types and commodity2 replacement fail explicitly. The inventory
report and subsequent late identity/nibble fields are tested in actual Person
suffix composition. See native-person-inventory.md for evidence and boundaries.

NativeJobList now composes all27 native job types, including Assassin0,
InterdictSystem20 and Story24, with valid
Qt7 UTC timestamps. All27 factory/reader/obsolete bindings are verified; other
time profiles remain explicit Unsupported. BountyHunter types are fully read
before obsolete filtering. Null factory stops propagate
as nested diagnostics, not guessed skipping. See native-person-jobs.md.

## Implemented Person dependencies

`world::native_person` now contains NativeDomain, NativeGlobeLlf and
NativeHitNotify, composed by NativePersonSuffixState along with Gear. These
do not yet form complete Person reception/dispatch or a presentation bridge.
Their static Ghidra records are retained completely with
limitations, together with independent exact-symbol GNU disassembly in
`evidence/native-person-helpers-gnu.txt`:

- Domain: `evidence/native-person-domain.json`,
  `ev_0af5413a56bb9f87961b98a82e44cd281f1322cf4c7bc89af5a9c1ab2ca328f8`,
  entry141361ce0. Clear before the unsigned8 count; each entry is AuId32 then
  nullable QString. Later duplicate keys replace earlier values. Rust BTreeMap
  preserves values/lookup, not Qt's hash iteration order or memory layout.
- GlobeLLF: `evidence/native-person-globe-llf.json`,
  `ev_09abc1f5bbd78e46c8e6331d889638c0e4dba99ecebb484bce1b7d5e00c49f2b`,
  entry14163cd10. Float field04 then float field00; the second call is a tail
  jump corroborated in GNU, not an unknown jump table. Rust fresh-value reads
  are atomic on malformed input, unlike native partial writes.
- HitNotify: `evidence/native-person-hit-notify.json`,
  `ev_24efd4c58f6d9197225f13412bb5f10221b9ac355d1e32584468d2e539f2a20b`,
  entry1411e1230. Three float temporaries are widened into double vector00.
  Raw unsigned byte18 controls signed32 field28 then20; raw unsigned byte1c
  controls signed32 field2c then24. Zero discriminants do not clear the omitted
  integers. Default Options indicate unobserved values, not constructor zeros.

All three support explicit TypeWidths/SinglePrecision/DoublePrecision profiles
where applicable. Float-overload destinations narrow even in double wire mode.
No finite-value filtering or semantic coordinate guesses are added. Successful
earlier Domain/HitNotify writes survive later Rust read failures; Qt failed-stream
and allocation/locking parity are not claimed. Fixtures cover every count and
both raw byte discriminants, repeated-key replacement, nullable/empty/non-ASCII
strings, preserved omitted integers, float narrowing/signed zero, exact trailing
alignment and all truncated prefixes. These records were reused from the saved
batch; no new Ghidra import or target execution was needed.

Verification: `just check` passes389 headless and410 client library tests,
9 CLI tests per profile,1 graphical-entry CLI test, formatting and strict
client-feature all-target Clippy. This verifies the implemented dependencies,
not the still-missing end-to-end Person/avatar path.
