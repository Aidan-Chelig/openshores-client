# Native base effects

Target: `reference/openshores-full/OpenShores.exe`, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
These are independent GNU static assembly observations, not runtime or REA
oracle results. Focused REA effect-reader corroboration is running.

The complete `AuEffect::operator<<` (`_ZN8AuEffectlsER11QDataStream`,
`0x1411d82e0..0x1411d85a9`) reads one flags byte followed unconditionally by
`AuTime::Rx16` into offset `0x70`, passing global offset `0x280` as its time
adjustment. Optional fields follow in bit order:

| Flag | Wire reads | Native destination |
| --- | --- | --- |
| `0x01` | three floats widened to doubles | `0x28`, `0x30`, `0x38` |
| `0x02` | byte, float, float | zero-extended byte at `0x64`, floats `0x08`, `0x40` |
| `0x04` | byte | high bit boolean `0x68`, low seven bits integer `0xa8` |
| `0x08` | three floats widened to doubles | `0x48`, `0x50`, `0x58` |
| `0x10` | three floats widened to doubles | `0x78`, `0x80`, `0x88` |
| `0x20` | three floats widened to doubles | `0x90`, `0x98`, `0xa0` |
| `0x40` | byte, float | `0xac`, `0xb0`; omission explicitly zeros both |
| `0x80` | byte, float | `0xb4`, `0xb8`; omission explicitly zeros both |

Omitted bits `0x01..0x20` leave the respective existing fields unchanged.
RxUpdateBase constructs one temporary effect before its count loop and reuses
that value for every reader call; resetting every record to fresh defaults
would change the native sparse-update semantics. Its append comparison uses
only time `0x70`, fields `0xa8`, `0x68`, `0x64`, coordinates `0x28..0x38` and
floats `0x08`, `0x40`, not complete effect structural equality. Floating compares
reject unordered values (NaN); raw bytes or bitwise equality are not substitutes.
Effects are not cleared simply because the next transmitted count is zero.

`AuTime::Rx16` (`0x14163a3c0..0x14163a3f7`) reads a 16-bit wire value, samples
QDateTime current milliseconds, adds the caller's adjustment, clears the low
16 bits of that sum, then ORs in the wire bits. It does not choose the nearest
epoch across a 16-bit wrap boundary. Any Rust port must receive explicit clock
and adjustment observations instead of deriving time from packet identity or
monotonic movement timers. The x86-64 addition wraps at 64 bits.

This establishes field layout/order, not effect rendering, names for unknown
fields, global adjustment semantics or Qt malformed-stream parity. Next work:
retain REA corroboration; implement bounded decoding with explicit clock inputs,
reused temporary state and native subset deduplication; connect it to ordered
base reception. Controller wire formats remain a separate unresolved layer.
