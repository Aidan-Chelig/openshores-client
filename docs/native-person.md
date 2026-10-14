# Native Person reception

Selected patched target: `reference/openshores-full/OpenShores.exe`, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
Focused REA `_ZN8DaPerson8RxUpdateER11QDataStreamb` decompilation is running;
its output must be retained/reviewed before claiming complete Person coverage.

Independent GNU assembly of the entry at `0x140a42db0` establishes that Person
first calls **DaSentient::RxUpdate** at `0x140b22720`, forwarding its is-new
argument. It then reads a Person flags byte. It does not jump straight from the
common atom base prefix to Person fields or invoke the source-server Person
decoder. Leading conditional branches include an additional byte when bit
`0x80` is set, before the low-bit Person fields. The bounded entry review through
`0x140a42edb` also shows explicit field `0x838` zeroing and state-bit writes at
`0x1130`; their meanings and complete receive effects remain unresolved.

Next: review complete Person evidence; reconstruct its Sentient/Creature layers
and conditional subreaders; establish constructor/default and body state;
extend concrete native object dispatch without assuming Item defaults. No
native execution or live-server testing is involved in this investigation.
