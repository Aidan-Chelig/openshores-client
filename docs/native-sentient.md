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

## Verified suffix implementation

Complete SkillSet evidence `evidence/native-skillset-reader.json`,
`ev_6fa5996c89222ed010e89f8407bfea3766401348a6f50ff0f62e3f65d2cd681b`,
receives an unused byte, clears the hash, reads short count, then short skill
ID and signed i32 value per entry. IDs widen unsigned; duplicate IDs replace
the value. Independent `evidence/native-sentient-widths-gnu.txt` corroborates
the byte/short/i32 overloads and the low-word count decrement/zero test; high
count bits are not a signed-positive check. The same GNU record pins Sentient's
flags/order-kind/6f0 reads to byte overloads and 708 to the float overload.

`world::native_sentient::NativeSentientState::receive_suffix` now implements
these ordered stages with the real SkillSet decoder. It is explicitly the
suffix after Creature, not a complete inherited receive path. Unobserved fields
remain Option rather than guessed constructor defaults. New-order construction
uses the initializer visible in the full Sentient record: matrix translation
normalized by the sum unless zero/one, word18 and ID1c zero, empty string20,
sampled wall-clock time28 and kind30 zero. A required seed callback supplies
the real translation, four denominator terms and clock only when initialization
is needed; missing context must error rather than invent a position/time.
Single-entry reuse retains existing position, timestamp and omitted string.

Earlier mutations persist on later Rust errors. Skill clear occurs after the
leading byte but before count; completed skill entries persist on a later
truncated entry. This models ordering, not Qt failed-stream continuation.
Tests cover all 256 flags, all three float profiles, zero/one/two prior orders,
conditional strings, high-bit IDs, duplicate skills, high-bit counts, exact
normalization branches/non-finite arithmetic, truncated stages and omitted
skill retention. Creature and concrete Person dispatch/presentation remain open.
