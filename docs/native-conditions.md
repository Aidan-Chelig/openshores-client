# Native condition-list reception

Selected target: `reference/openshores-full/OpenShores.exe`, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
Reviewed full static Ghidra 12.1.2 evidence:
`evidence/native-condition-list.json`,
`ev_18e528f9cecea1628d77c0a42a1961d8283c16d03fe6bd38eab346a48663d10d`.

`AuConditionList::operator<<` (`0x1408be450`) first erases every old list
entry and invokes each non-null condition's virtual destructor. Only then does
it read a short count. Each iteration reads an 8-bit type and calls
`AuCondition::CreateCondition`; a non-null result receives its virtual body at
vtable offset `0x70` and is appended. A null factory result emits a warning and
continues with the next type: there is no observed unknown-body length or skip.
Zero count therefore clears the previous list, unlike the additive base effect
and controller layers. Prior completed appends precede later iterations.

Remaining unknowns: count overload signedness/loop interpretation, complete
factory mapping, each concrete virtual reader and constructors. Do not treat
these as Item/controller types or silently skip arbitrary condition payloads.
Rust malformed-input behavior must be documented separately from Qt streams.

## Factory mapping

Complete reviewed `CreateCondition` evidence is retained in
`evidence/native-condition-factory.json`,
`ev_cd21060fb6057b8729a6808bf12bf403ed5d7ead3cf23b2e34f5bb7840f79866`.
Observed types: 1 Poison, 2 Disease, 3 Paralysis, 4 ImplantDNA, 5 Acid,
7 Thought, 8 Fire, 9 Pregnant, 10 Bandage, 11 Bounty, 12 ImplantSpecies,
13 Antitoxin. Type 6, zero and all other values enter warning/null fallback;
they are not generic base-condition allocations.

Every known allocation zeroes qword `0x08`. Poison/Paralysis/Acid initialize
the word at `0x10` to `0x0100`; Disease zeroes that word and sets byte `0x12`
to one. Bandage/Antitoxin set byte `0x10` to one. Thought uses the shared-null
string at `0x10`. ImplantDNA allocates and constructs DhDNA for pointer `0x10`;
ImplantSpecies constructs AuSpecies in place at `0x10`. Pregnant and Bounty
have distinct additional zero/string initializers; full semantic defaults
remain unestablished. Fire has only the base-sized allocation (`0x10` bytes).
These observed constructors do not establish the virtual receive body layouts.

## Shared reader

Complete reviewed base-reader evidence: `evidence/native-condition-base.json`,
`ev_9db76c8fc28cdf8914d87bee588913e6d5167d46e7add7a805b07f2024c036f2`.
The reader delegates solely to `AuTime::operator<<` for field `0x08`.
It does not itself read a type byte or an additional condition flags byte.
The subsequently reviewed time-reader record `evidence/native-time-reader.json`,
`ev_fba3b904ef8e6ba0132615f32009391c4f2a902917202b298995dbc45461e753`,
delegates directly to `QDataStream >> long long`. Independent GNU assembly
`0x14163a380..0x14163a38f` corroborates the tail jump to imported
`_ZN11QDataStreamrsERx` at `0x142892710`. The decompiler's jump-table warning
does not establish a switch: this is a direct imported-overload tail call.
For the established big-endian native stream profile this is an eight-byte
signed timestamp, with no sampled clock, epoch adjustment or compressed Rx16
reconstruction. Preserve zero, negative and extreme values unchanged.

Independent COFF symbol inventory places base and Fire receive symbols at the
same entry `0x1408b63b0`. Other receive entries are distinct: ImplantDNA
`0x1408b63c0`, Poison `0x1408b65e0`, Disease `0x1408b6630`, Paralysis
`0x1408b6680`, Acid `0x1408b66d0`, Bandage `0x1408b6720`, Antitoxin
`0x1408b6760`, ImplantSpecies `0x1408b6b50`, Thought `0x1408b6b80`, Pregnant
`0x1408b6c60`, Bounty `0x1408b6db0`. Symbol aliasing alone is not the full
virtual-table binding proof or a reconstruction of their nested bodies.

## Scalar condition bodies

Full reviewed Poison evidence: `evidence/native-condition-poison.json`,
`ev_3c92f5122de57b110763de3bb0285c0c2759a7d988a111feb6a98ffa3c8c64f7`.
Poison receives full-width AuTime, byte `0x10`, byte `0x11`; it changes the
last byte to one only when zero. This is not boolean conversion: all nonzero
byte values remain unchanged.

Independent GNU assembly completely reviews these scalar readers, pinning all
byte reads to the signed-char Qt overload (raw byte bits are retained):

- Poison `0x1408b65e0..0x1408b6622`: timestamp + two bytes, last zero → one.
- Disease `0x1408b6630..0x1408b667b`: timestamp + three bytes at `0x10..0x12`,
  last zero → one; the preceding two bytes are not normalized.
- Paralysis `0x1408b6680..0x1408b66c2` and Acid
  `0x1408b66d0..0x1408b6712`: same wire sequence/normalization as Poison.
- Bandage `0x1408b6720..0x1408b6754` and Antitoxin
  `0x1408b6760..0x1408b6794`: timestamp + one byte, zero → one.

These establish direct method layouts, not yet the factory-created vtable
slot `0x70` bindings. Check those bindings before making the type dispatcher
authoritative. Complex DNA/species/string/pregnancy/bounty bodies remain open.

## Thought and virtual bindings

Full reviewed Thought evidence: `evidence/native-condition-thought.json`,
`ev_2698c41c47376db27e823a9b3ef9b33d017ba6fe43cd232bb20319941e0b0f53`.
It reads AuTime at `0x08`, then QString at `0x10`, with no extra flag byte.
The decompiler flags the final imported QString tail call as an unresolved
jump table; this is a provider warning, not evidence of extra switch cases.

Independent GNU `.rdata` reads now corroborate every factory vptr's receive
slot (`vptr + 0x70`, little-endian 64-bit pointers). These bind the direct
reader methods previously located to the actual list dispatcher:

| Type | Slot address | Receive entry |
| --- | --- | --- |
| 1 Poison | `0x1423d0990` | `0x1408b65e0` |
| 2 Disease | `0x1423dbc80` | `0x1408b6630` |
| 3 Paralysis | `0x1423fd710` | `0x1408b6680` |
| 4 ImplantDNA | `0x14240c830` | `0x1408b63c0` |
| 5 Acid | `0x142490eb0` | `0x1408b66d0` |
| 7 Thought | `0x1423dbd40` | `0x1408b6b80` |
| 8 Fire | `0x142490f70` | `0x1408b63b0` |
| 9 Pregnant | `0x1423ebeb0` | `0x1408b6c60` |
| 10 Bandage | `0x1423dbbc0` | `0x1408b6720` |
| 11 Bounty | `0x1423d08d0` | `0x1408b6db0` |
| 12 ImplantSpecies | `0x142442260` | `0x1408b6b50` |
| 13 Antitoxin | `0x1423fd650` | `0x1408b6760` |

Fire therefore consumes only the shared full-width timestamp. Eight condition
types now have verified body layouts; the four complex types still need their
nested readers. Unsupported known bodies must produce an explicit error, not
be confused with native null factory results.

Next implement the verified bodies and resolve remaining nested readers, then compose
the concrete counted layer into Unit reception.

Rust `world::native_condition` now decodes the eight verified bodies directly.
Four known complex types return Unsupported; native null-factory types return
None without consumption. The reader is atomic on errors (Rust safety, not Qt
error parity), uses the existing strict QString decoder, preserves signed i64
timestamps and nonzero raw bytes, and applies only the observed zero-to-one
normalization. Tests cover every scalar byte, every truncated scalar/string
prefix, null/empty/Unicode strings, extreme timestamps and all 256 type bytes.
This is not yet a counted list, Unit composition or complete Person dispatch.
