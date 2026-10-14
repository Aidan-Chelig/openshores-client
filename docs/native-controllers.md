# Native base controllers

Target: patched `reference/openshores-full/OpenShores.exe`, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
The full REA Append result is retained as
[native-controller-append.json](evidence/native-controller-append.json), Evidence
`ev_c9c22176be33d6cd317d50d570dc3df921a7cab097f4f6390216849ad757411a`
(636055ms, Ghidra 12.1.2, matching patched target digest). Complete pseudocode
review corroborates the count/type loop, virtual receiver call, unconditional
append and warning-only unsupported-type handling. Factory/jump-table facts
below use independent GNU assembly; this REA result calls the factory without
establishing its internals. All findings are static, not target execution.

`AuSyncControllerList::Append(QDataStream&)` at `0x1411e72c0` reads an unsigned
byte count and then one unsigned type byte per entry. It calls Create, invokes
the returned object's receiver at vtable slot `0x18`, then appends its pointer
unconditionally. There is no deduplication or replace call in this path.
An unsupported type logs a warning and continues without consuming a guessed
body or inserting null. Count zero makes no list changes. Normal/logging paths
through `0x1411e746b` and exception tails through `0x1411e748f` were inspected.

The Create factory at `0x1411e6670..0x1411e684b` bounds types to `0..4`.
The signed relative jump-table words at `0x1422b22ec` resolve as follows:

| Type | Constructor branch | Established vtable/class |
| --- | --- | --- |
| 0 | `0x1411e6768` | AuSyncAtomPos |
| 1 | `0x1411e67f0` | AuSyncVector |
| 2 | `0x1411e6828` | AuSyncMotion |
| 3 | `0x1411e6690` | AuSyncAtomPosVec |
| 4 | `0x1411e6730` | AuSyncPose |

Out-of-range types return null. Symbol names identify candidate receivers:
AtomPos `0x1411e6310`, Vector `0x1411e6550`, Motion `0x1411e62f0`,
AtomPosVec `0x1411e6560`, Pose `0x1411e6610`.

Independent little-endian pointer reads at address-point plus `0x18` confirm
these bindings: AtomPos `0x1423f6788`, Vector `0x1423e5478`, Motion
`0x1423e53e8`, AtomPosVec `0x142424d68`, Pose `0x1423c9bd8`.
Complete receiver bodies establish Motion consumes no body bytes (single ret),
Vector reads three doubles via AuPoint (`0x141648040..0x141648074`), and Pose
reads float, unsigned byte widened to integer `0x0c`, raw byte `0x10`, short
`0x12` (`0x1411e6610..0x1411e6662`). AtomPosVec's full receiver through
`0x1411e65b7` first reads unpacked AtomPos, then flags: bit 0 writes boolean
`0x118`; bit 1 reads three doubles at `0x120..0x130`. Omission retains that
vector, whose factory initialization is zero. Other flags add no fields.

Both position receivers use **AuAtomPos::operator<<** at `0x1408b2160`, not
packed **AuAtomPos::Rx**. Its wire-call inventory through its return at
`0x1408b26cf` is AuId followed by two AuPoint readers: u32 parent, three
position doubles, three rotation doubles. No packed flags byte occurs here.
The remaining matrix arithmetic must be reviewed before claiming complete
unpacked native position-state reconstruction. Reusing PackedPosition::read
would therefore consume the controller stream incorrectly.

`native_controller/` now decodes all five verified body formats. Unpacked
positions retain parent/position/rotation double fields without pretending the
unreviewed matrix calculation is implemented. Pose fields retain native widths;
Motion consumes no bytes. AtomPosVec uses the factory-zero vector on omission,
preserves raw flags and exposes bit 0 independently. Unknown types return None
without guessed body consumption. Known malformed bodies preserve reader state
as Rust safety rather than claiming Qt malformed-stream parity. Tests cover
exact boundaries, every truncated body, all 256 AtomPosVec flags, unsupported
types, double precision and signed-short/raw-byte width distinctions.

`NativeControllerList::receive_base_layer` now implements counted append and
the base caller's empty-list disposal. It appends duplicate supported values,
reports unsupported type bytes in order without inventing their body lengths,
and retains previous entries on zero count. Empty completed lists become
unallocated; later Rust errors preserve allocation and prior appends. A private
20,000-entry cap is Rust safety, not native policy. Unknown-type warnings are
reported data for the adapter, not silently emitted external messages. Tests
cover duplicates, unsupported types, empty/null lifecycle, maximum byte count,
truncation and capacity partial effects.

The ordered base-receiver test adapter now uses real controller reception,
replacing its synthetic marker. A nonempty effects layer and duplicate Motion
controllers end exactly before untouched class-body bytes; all 256 base flags
use real zero-count layer handling. This remains test integration, not a claim
that the full native world/session owner is connected to playable UI.
Next: connect the concrete layers to native atom/world reception.
Controller application/interpolation and Qt locking remain additional work.
