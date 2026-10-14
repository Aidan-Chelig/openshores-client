# Native base effects

Target: `reference/openshores-full/OpenShores.exe`, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
The complete REA effect-reader result is retained as
[native-effect-reader.json](evidence/native-effect-reader.json), Evidence
`ev_414a7d282b7f0c38a4ba4563963b99a76aa186d3c4f1071096e8cc3c76e1648a`
(647671ms, Ghidra 12.1.2, matching target digest). Full pseudocode review
corroborates the reader fields and sparse/zeroing branches below. The Rx16
implementation and append-comparison findings use independent GNU assembly
and the previously retained base-receiver evidence; this focused REA result
contains calls to those helpers, not their implementations. No target execution.

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

`world::native_effect` now decodes these fixed-size flag fields into a reusable
value with explicit clock/adjustment inputs. Initialization follows the reviewed
RxUpdateBase temporary, including float defaults of one at `0x08`/`0x40`.
Incomplete inputs leave that value and reader unchanged (Rust safety, not Qt
malformed-stream parity). Non-finite floats remain raw, not prematurely rendered.
The explicit append-key comparison follows the native subset and IEEE behavior.
Tests cover all 256 flags, every truncated record, sparse reuse, explicit pair
zeroing, 64-bit clock wrap, non-nearest time reconstruction and NaN/signed-zero
comparison. These are synthetic Rust tests backed by static evidence, not a
native runtime oracle.

Effect rendering, unknown-field meanings and global adjustment semantics remain
unestablished. `NativeEffectList` now receives the unsigned-byte count, allocates
storage even for zero count, reuses one temporary within a packet, and appends
only values that fail the native subset comparison. A new packet starts a fresh
temporary without clearing retained effects. Its private collection is bounded
to 20,000 values as Rust safety, not a recovered native cap. Completed records
survive later errors; a budget failure can leave the decoded record consumed
without appending it. Clock inputs are explicit per record, including duplicates.
Tests cover sparse inheritance, packet-boundary reset, duplicate suppression,
zero-count retention, all 255 counted records and partial/error effects.

The ordered base-receiver test adapter now uses the real counted decoder,
including a nonempty effects layer followed by motion, a synthetic controller
callback marker and untouched class-body bytes. This verifies the layer boundary
without claiming controller decoding or playable native-owner integration.
Controller wire formats remain a separate unresolved layer.
