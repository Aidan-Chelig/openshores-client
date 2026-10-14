# Native Sentient reception

Target: `reference/openshores-full/OpenShores.exe`, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
Complete, reviewed static Ghidra 12.1.2 evidence:
`evidence/native-sentient-update.json`,
`ev_f9792030525d718f1b2a6ce4a61b4329abc9661ba1416f112a59ebecb3735733`.
No executable was run. Decompiler types are recovered, not original source.

`DaSentient::RxUpdate` first calls `DaCreature::RxUpdate`, forwarding is-new.
It then reads one flags byte:

- Bit `0x01`: receive floating field `0x708`; absent explicitly writes zero.
  Stream width must be corroborated from the actual overload/precision profile.
- Bit `0x02` absent: clear the order list at `0x6e8`. No observed reset of
  field `0x6f0` on this branch.
- Bit `0x02` present: if the list does not already have exactly one entry,
  clear it and append an initialized order. Its position comes from the matrix
  translation at `0x278`/`0x280`/`0x288`, divided by the sum of fields
  `0x230`/`0x250`/`0x270`/`0x290` unless that sum is zero or one. Initialization
  samples wall-clock milliseconds and uses an empty string; this is local
  state, not a position or timestamp received in this suffix.
- Then read a scalar whose low byte becomes the first order's field `0x30`,
  receive its ID at `0x1c`, and receive its string at `0x20` only when the
  scalar is `0x29`. Other scalar values retain the existing string when the
  single-entry order was reused. Finally read a scalar whose low byte replaces
  field `0x6f0`. Exact scalar overloads remain to be corroborated.
- Bit `0x04`: call `AuSkillSet::operator<<` for field `0x7c8`, after order
  reception. Its body layout is not yet reconstructed.

No other flag-bit reader appears in this complete function. This is not a
complete Person decoder: Creature, skill-set body, constructors, order layout,
and scalar widths remain unresolved. Do not model list preservation as a
simple always-replace packet list or skip unknown nested bodies.

Next: inspect Creature reception, then corroborate widths and nested readers
before extending concrete native Person dispatch.
