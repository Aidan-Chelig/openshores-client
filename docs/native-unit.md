# Native Unit reception

Target: `reference/openshores-full/OpenShores.exe`, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
Reviewed complete REA Ghidra 12.1.2 record `evidence/native-unit-update.json`,
`ev_00a239282a556bce6853bd7c3cd135247f3b78cf3262cc88b59edd379ca98182`.
Independent GNU assembly review covers the complete reader
`0x140ef7d20` through `0x140ef7e6a` (not an execution oracle).

Unlike Creature/Sentient/Person, Unit does not call an inherited RxUpdate here.
The generic packet dispatcher separately receives the common base first.
The body receive chain is Unit → Creature → Sentient → Person.

Unit reads an 8-bit flags byte (`QDataStream >> signed char`):

- `0x01`: QString at `0x3b0`, signed 32-bit integer at `0x3a8`, then
  one raw byte at `0x39c`, in that order. Omission retains these fields.
- `0x02`: `AuConditionList::operator<<` at `0x1408be450`, object field `0x408`.
  Its nested body remains unresolved, so it must not be skipped or presumed empty.
- `0x04`: three `QDataStream >> float` values, widened to doubles and written
  under the object's write lock to `0x330`/`0x338`/`0x340` only after all three
  reads. Omission explicitly writes all three zero, not sparse retention.

Other flags have no observed reader in this complete function. The assembly
pins overloads, not the stream precision configuration; the existing native
Qt version-7 profile must still be selected explicitly by any Rust decoder.
Constructor defaults and semantic field names are not established here.

Next reconstruct the condition-list reader, then implement the ordered Unit
layer in a module folder and compose it into the full native Person pipeline.
