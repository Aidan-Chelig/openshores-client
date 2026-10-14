# Native Person reception

Selected patched target: `reference/openshores-full/OpenShores.exe`, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
Complete REA `_ZN8DaPerson8RxUpdateER11QDataStreamb` decompilation is retained in
`evidence/native-person-update.json`, evidence
`ev_7c21d6bd5041896517ec6cd7baf6271ed190525ee97835989f62d203fb6f772a`.
The full pseudocode has been reviewed. This is static Ghidra 12.1.2 evidence,
not execution; its type-propagation warning requires scalar-width corroboration.

Independent GNU assembly of the entry at `0x140a42db0` establishes that Person
first calls **DaSentient::RxUpdate** at `0x140b22720`, forwarding its is-new
argument. It then reads a Person flags byte. It does not jump straight from the
common atom base prefix to Person fields or invoke the source-server Person
decoder. Leading conditional branches include an additional byte when bit
`0x80` is set, before the low-bit Person fields. The bounded entry review through
`0x140a42edb` also shows explicit field `0x838` zeroing and state-bit writes at
`0x1130`; their meanings and complete receive effects remain unresolved.

## Person suffix order

After Sentient reception, read flags A; if A has `0x80`, read flags B immediately,
otherwise B is zero. The suffix then proceeds in this order:

- A `0x01`: ID/string, ID/string, `AuGlobeLLF`, ID/bool/string.
- A `0x02`: `AuHitNotify`.
- A `0x04`: packed short; low 15 bits update field `0x660` and restart its
  timer only on change, high bit replaces byte `0x839`.
- A `0x08`: read byte C and assign `0x838 = C & 0x3f`; otherwise C and
  `0x838` are explicitly zero (not sparse retention).
- A `0x10`: packed scalar plus integer `0xf80`; derive byte `0xf84` from
  bit 7 and field `0xf88` from bits 3–4 of the packed scalar's low byte.
- A `0x20`: byte `0xfa8`, packed byte D; assign `0xfa9 = D & 0x0f`, clear
  content list `0xfa0`, optionally read counted `AuItemContentId` entries
  when D has `0x80`, then read the scalar whose low byte becomes `0xfb0`.
- A `0x40`: receive `AuGear` under a write lock, then recompute weight `0x13a8`.
- Extended B: `0x01` domain; `0x04` job list; `0x08` integer `0x10d0` and
  short `0xf58`; `0x10` floating field `0x1408`; `0x20` floating fields
  `0xf60`/`0xf68`. B `0x02` has no observed reader here.
- B `0x40` sets state bit `0x200000` without an observed corresponding clear.
  State bit `0x8000` is replaced by B `0x80`, and cleared without extension.
- C `0x80`: `AuItemHash`, AFTER the extended fields.
- A `0x01`: trailing scalar whose low byte replaces `0x10e8`, not adjacent
  to the earlier A `0x01` identity fields on the wire.
- C `0x40`: final packed scalar; low nibble becomes `0x83d`, high `0x83b`.

Subreader layouts, ambiguous scalar overload widths, floating stream precision,
constructor defaults and semantic field names remain unresolved. In particular,
decompiler `double*` alone does not establish eight-byte stream consumption.
Do not implement an opaque skipped subreader or borrow the source-server body.

Next: reconstruct its Sentient/Creature layers
and conditional subreaders; establish constructor/default and body state;
extend concrete native object dispatch without assuming Item defaults. No
native execution or live-server testing is involved in this investigation.
