# Native Person gear reception

Selected static target: reference/openshores-full/OpenShores.exe, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
No supplied executable ran and no public server was contacted.

Complete retained Ghidra records, including provider limitations:

- `evidence/native-person-gear.json`,
  `ev_f3bef3ccad56c9b405fbd439c6c53c0601683cdc71932d77be616cdc0cf7a710`.
- `evidence/native-person-gear-slot.json`,
  `ev_131a5cc77c3c5cd69da6316a48db34fb4c04adbc5679aa62a17c0143749904ff`.

Independent exact-symbol GNU disassembly in
`evidence/native-person-gear-gnu.txt` corroborates Gear1414c83e0,
GearSlot1414c6120 and const AuItem::FixItem1414f04b0. These are static
observations, not native execution/oracle results. The saved analyzed batch was
reused; these dependencies did not cause another Ghidra import.

## Implemented wire state

NativeGear clears its previous list **before** reading the unsigned8 count.
Each slot reads one packed byte: native byte08 gets its low nibble and byte09
its high nibble. Then it reads an item factory byte and actual item body.
Type0 produces a null slot; so do the verified null factory types2/3/11/15/21
and25..255. Gear warns and continues without appending them.
Successful non-null slots append in wire order, including duplicate positions.
NativeGearReceive surfaces null-slot indices as a must-use diagnostic result.

All seven established NativeItemValue types1/6/7/8/9/12/17 compose their actual
common/extension readers. Other valid factory types fail Unsupported, not guessed null or
opaque body skipping. The slot is atomic on Rust read failure; completed prior
Gear appends remain after a later failure. This safety policy is distinct from
Qt failed-stream/partial-storage and native allocation/copy/destructor behavior.

FixItem compares the native unsigned16 commodity field08 with2. Other values
return null replacement, preserving the original item. Commodity2 tail-calls
the commodity-based Create overload with commodity0xa1, actual byte20 and
native fields10/0c as its other arguments. This replacement requires constructor
and commodity-default reconstruction; it currently fails Unsupported rather
than returning the uncorrected item. The existing item value records preserve
wire observations and do not establish every constructor-derived native field.

## Remaining required application

Person calls Gear::Weight immediately after receiving Gear, storing float13a8.
That recomputation is **not** implemented by NativeGear::receive. It must not be
treated as zero or inferred from item count. Full Person dispatch/application
and avatar equipment presentation remain open, as do unimplemented item types
and commodity2 construction. These are explicit unfinished requirements, not
a redefinition of the end-to-end goal.

The separate inventory reader1414f11d0 is retained in
`evidence/native-person-item-hash-gnu.txt`. It clears before a signed32 count,
performs item factory/body reception, tests DbCommodity::IsObsolete before
replacement, and inserts surviving items by commodity. Its STL template uses
the non-unique-key emplace overload; do not collapse duplicate commodities into
a unique-key map without resolving that behavior. It is not implemented yet.

## Fixtures

Fixtures exercise all256 packed slot bytes for every supported item family,
all256 Gear counts, alternating null/non-null entries, duplicate positions,
trailing alignment, explicit unsupported factories/fixups, every truncated
prefix, clear-before-count and preservation of completed prior slots. They
exercise real item readers, not callback substitutes for Gear decoding.

The inventory milestone's verified byte-factory mapping is now shared through
NativeItemValue::factory_is_null, rather than maintaining separate classifiers.
Its full evidence/static verifier is indexed in native-person-inventory.md.
This source composition corrects Gear's former conservative Unsupported result
for known null types without changing how valid unimplemented bodies fail.
All256 factory bytes have Gear continuation/alignment fixtures; actual Person
suffix tests verify zero weight and following Domain reception for all null
factory classes. No new analysis/import or runtime claim is involved.

Follow-up verification: `just check` passes425 headless and446 client library
tests, CLI tests, formatting and strict Clippy; `just test-native-inventory`
passes the reused static factory/filter/insertion checks.

`just check` passes393 headless and414 client library tests,9 CLI tests per
profile,1 graphical-entry CLI test, formatting and strict client-feature
all-target Clippy. This is component verification, not complete Person/avatar
reception or runtime weight/equipment presentation.
