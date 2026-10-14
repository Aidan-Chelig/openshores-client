# Patched native common item reader

Target: `reference/openshores-full/OpenShores.exe`, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
This is static evidence, not target execution. The complete focused REA/Ghidra
result for `_ZN6AuItemlsER11QDataStream` is retained in
[`evidence/native-item-common.json`](evidence/native-item-common.json), Evidence
`ev_03b5f17ef6c3315249c13db5e06f90e6c3ebccf005908bfd988bebb74f9f256b`,
Ghidra 12.1.2, 664065 ms. It corroborates the common-reader branch order and
defaults below. Its recovered operator spelling contains a placeholder and
the QString helper prototype omits the length argument; original disassembly,
not those recovered names/prototypes, establishes actual stream direction and
the omitted-name empty-string behavior.

`AuItem::ls` spans `0x1414d07a0..0x1414d095f`. Its wire order is:

| Field | Guard | Native destination/default |
| --- | --- | --- |
| byte flags | always | retains unknown bits 0x40/0x80 |
| signed short commodity | always | offset 0x08 |
| signed int | bit 0x01 | offset 0x0c; omitted = 1 |
| signed int | bit 0x02 | offset 0x10; omitted = 0 |
| byte | bit 0x04 | offset 0x14; catalog-dependent if omitted |
| QString | bit 0x08 | offset 0x18; omitted resets to empty |
| byte | always, after name | offset 0x20; zero coerced to 1 |
| signed int | bit 0x10 | offset 0x24; omitted = 0 |
| byte | bit 0x20 | offset 0x28; omitted = 0 |

Anchors: flag read `0x1414d07c4`, short read `0x1414d07cd`,
conditional int reads `0x1414d0947`, `0x1414d0927`, `0x1414d0907`,
byte reads `0x1414d0805`, `0x1414d0822`, `0x1414d08ef`,
QString read `0x1414d0815`, zero coercion `0x1414d082a`.
Omitted offset-0x14 calls `DbCommodity::IsObsolete` at `0x1414d089c`.
Nonobsolete items use commodity-record byte 0x108, with commodity 2 aliased to
161; obsolete items leave constructor/current byte untouched. No catalog
resolver is implemented here. None represents unresolved omission, not zero.

The Rust implementation lives under `src/world/native_item/`, uses raw offset
names until field semantics are established, preserves signed destinations,
distinguishes omitted/null/empty names, retains the unconditional wire byte,
and exposes its native zero-to-one normalization separately. Reads are atomic
on failure and leave subtype suffixes untouched. Invalid UTF-16 and names
over 256 KiB are explicit Rust safety rejections, not recovered native limits.

Do not route arbitrary subtypes through this reader. `DaItem::RxUpdate` at
`0x140b7ad70` conditionally reads an AuItem type byte then dispatches its
virtual reader; `AuItem::Create(u8)` at `0x1414f05e0` uses the jump table at
`0x1422e1338`. Type 1's table entry resolves to `0x1414f0be0`, which writes
`_ZTV6AuItem+0x10` (`0x14247c7b0`) as the address point; virtual slot 0x28
contains `0x1414d07a0`. This establishes plain-item dispatch, not all subtypes.

Additional established bindings, checked from original PE bytes:

| Type | Factory entry | Vtable address point | Reader |
| --- | --- | --- | --- |
| 6, AuItemState | 0x1414f0a30 | 0x1423d1ee0 | 0x1414d0a00 |
| 7, AuItemAmmo | 0x1414f09b8 | 0x1423c4720 | 0x1414d0a70 |
| 17, AuItemStateMessage | 0x1414f0f38 | 0x142438ed0 | 0x1414d0a30 |

All three call/tail-call the common reader first. State adds one byte at 0x4c;
StateMessage adds that byte and a QString at 0x50; Ammo adds nothing.
`NativeItemValue` preserves the subtype even when two share the same layout.
Factory type 8 constructs AuItemWeapon through its constructor; it is **not**
an Ammo alias just because a sequential disassembly window reaches the next
case. Weapon layouts, including ammunition and weapon-state variants, now have
separate extension readers; see [native weapons](native-weapons.md).

`NativeItemBody` decodes these seven types, omission of the item value, and
the native ownership branches: 0x80 reads an ID and two QStrings at offsets
0x3dc/0x3e0/0x3e8; otherwise 0x40 writes sentinel ID 1 without resetting the
strings; otherwise ownership remains unchanged. Flag 0x20 replaces boolean
0x39c on every update. String semantics and that boolean's gameplay meaning
remain unnamed. Unsupported subtypes leave the entire body unconsumed.
`NativeAtomPrefix::item_body` requires native Item opcode 0x16 and an
established type-specific boundary, refusing unknown effects/sync base layers.
It does not apply item state or connect native packets to the graphical client.
The source-profile `ItemState` is unchanged:
its field ordering is not interchangeable with the patched native reader.

Offline diagnostic, with a caller-selected common-record boundary:

```sh
cargo run --offline -- inspect-native-item-common 00000100dead
cargo run --offline -- inspect-native-item-value 060000010080dead
```

This reads commodity 1, omitted/default fields and wire byte 0 (effective 1),
leaving `dead` as an opaque suffix. Names are printed as UTF-8 hex with a
separate presence flag to preserve null versus omitted versus empty.
Unit tests exhaust all 256 flags, truncations, signed extremes, normalization,
surrogate pairs, and name safety bounds; these are synthetic layout checks,
not a Qt/original-runtime oracle or full native session compatibility.
Body tests also exhaust all 256 outer flags, ownership precedence and omission,
all 249 unimplemented subtype IDs, truncations, and base-layer dispatch guards.
`just test-native-items` additionally pins the native reader anchors to the
target digest and checks 256 independent Python struct/UTF-16 fixtures through
the Rust command, plus malformed and overbudget diagnostics. Static checks also
pin all seven factory/vtable bindings, subtype and outer-reader call sites.
The value diagnostic additionally verifies 36 independent subtype fixtures and
byte-exact preservation of all unsupported type IDs.

## Capture integration

`CapturedNativeFrame::item_body()` reuses the native atom's boundary guard and
subtype dispatch without altering the original payload or prefix-only loader.
Known malformed item bodies fail; unsupported types/base layers stay opaque.
The opt-in diagnostic `inspect-native-item-capture CAPTURE_FILE` prints bounded
summaries (subtype, commodity, ownership branch, boolean, consumed/remaining
bytes) alongside original frame hashes. It validates all requested item bodies
before stdout, so a late error cannot leave a misleading partial report. It
retains only summaries during that validation, not a second full collection
of decoded names. `inspect-native-capture` remains prefix-only and unchanged.

`just test-native-items` also checks a multi-frame independent fixture with
all seven supported subtypes, sentinel ownership, unknown DNA, effects/sync
boundaries, nonitem/unknown opcodes, byte-exact raw retention, and a malformed
last body. This is offline capture decoding, not native world-state mutation,
render integration, native capture framing parity, or a live-session test.

## Weapon trace

GNU disassembly of `AuItemWeapon::ls`, `0x1414d0a80..0x1414d0d44`,
establishes a second flag byte and mandatory signed short at 0x4c after the
common reader. Bits 1/2/4/8 enable groups; any of mask 0x1c reads another
short at 0x66, and bit 0x20 switches selected fields from byte to short.
Do not infer an ordinary repeated-group schema: the group selected by bit 8
reads into **0x6c** at `0x1414d0c71` or `0x1414d0d1f`, overlapping a field
also used by bit 4, rather than into 0x78. The omitted bit-8 branch clears
0x78 but not 0x6c. Narrow reads also write only part of 32-bit storage.
The implementation keeps original wire groups separate from final known
fields and applies these overlaps in native order on fresh constructor state.
WeaponAmmo calls this reader then reads four bytes at 0x80..0x83;
WeaponState calls it then reads one byte at 0x80. Neither is common-only.
