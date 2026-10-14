# Patched native weapon items

Same target/digest as [native items](native-items.md); GNU disassembly is
static evidence only. The focused REA/Ghidra request for
`_ZN12AuItemWeaponlsER11QDataStream` completed in 646359 ms with Ghidra 12.1.2.
The complete validated result is retained in
[`evidence/native-item-weapon.json`](evidence/native-item-weapon.json), Evidence
`ev_11fd85799e8d49354bec8bec7e7c4045b8bc0c1062126be90a85303074d93826`.
It corroborates the flag/width branches, zeroing and overlapping write order
below. Recovered operator names contain a placeholder; disassembly/imports
establish actual stream direction. This is not native runtime observation.

Factory/vtable bindings from original PE bytes:

| Type | Factory entry | Address point | Reader |
| --- | --- | --- | --- |
| 8, Weapon | 0x1414f0998 | 0x1423de1a0 | 0x1414d0a80 |
| 9, WeaponAmmo | 0x1414f0960 | 0x14241e890 | 0x1414d0d50 |
| 12, WeaponState | 0x1414f0e60 | 0x14242b630 | 0x1414d0db0 |

Types 8, 9 and 12 call `AuItemWeapon::AuItemWeapon` at 0x1414df620. That
constructor zeroes every modeled weapon field, including the two halves of
0x50/0x5c/0x6c/0x78. Types 9 and 12 then replace the vtable address point.
The slot at address point + 0x28 selects each reader. All weapon readers
start with the established common AuItem record, not a common-only fallback.

The weapon extension reads a flag byte and signed short at native offset 0x4c.
Bits 1, 2, 4, 8 enable four groups in order. Between the second and third
groups it reads signed short 0x66 whenever **any** bit in 0x1c is set, including
bit 0x10 alone. Every group reads byte, signed short, variable-width value,
signed short, byte, signed short. Bit 0x20 selects a signed-short overload
for the variable value; otherwise it reads a byte and zero-extends it to a
16-bit destination. Rust retains the short's raw bits as u16.

Known destination offsets for each group:

| Guard | byte | short | variable | short | byte | short |
| --- | --- | --- | --- | --- | --- | --- |
| 0x01 | 0x4e | 0x50 | 0x52 | 0x54 | 0x56 | 0x58 |
| 0x02 | 0x5a | 0x5c | 0x5e | 0x60 | 0x62 | 0x64 |
| 0x04 | 0x68 | 0x6a | 0x6c | 0x6e | 0x70 | 0x72 |
| 0x08 | 0x74 | 0x76 | **0x6c** | 0x7a | 0x7c | 0x7e |

The last group overwrites the LOW half of 0x6c, preserving the HIGH half
written by the previous group's short at 0x6e. It does not write the low half
of 0x78. The omitted last group clears 0x78 but leaves 0x6c untouched. Fresh
construction matters: earlier short writes into wider destinations leave
unwritten halves as constructor zero. No native padding bytes are invented.
These offsets are observed; their combat/gameplay meaning is not established.

`NativeWeapon` retains original wire groups and flag bits; `fresh_fields()`
reconstructs only known final fields for a freshly constructed object.
This is the context used by native DaItem replacing its previous AuItem
pointer. It is not an implementation of ls on arbitrary preexisting storage.
WeaponAmmo adds four raw bytes at 0x80..0x83; WeaponState adds one at 0x80.
Atomic errors and unconsumed subtype/body suffixes are preserved.

Code and tests are split under `src/world/native_item/weapon/`. Offline
`inspect-native-item-value` exposes original groups and the four packed final
fields, alongside raw subtype suffix bytes. `just test-native-items` checks
all seven factory/vtable bindings, common and weapon reader anchors, 768
independent weapon/subtype fixtures and malformed records. Rust tests also
exhaust flags, every truncated prefix, subtype suffixes followed by ownership,
and native overlapping writes. These are static-layout/mathematical checks,
not execution of the patched client or native network compatibility.
