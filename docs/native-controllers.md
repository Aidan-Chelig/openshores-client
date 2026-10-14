# Native base controllers

Target: patched `reference/openshores-full/OpenShores.exe`, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
Independent GNU static assembly observations; not target execution or a REA
oracle. Focused REA Append corroboration is running.

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
AtomPosVec `0x1411e6560`, Pose `0x1411e6610`. Actual vtable-slot bindings and
complete receiver bodies must be reviewed before porting their fields.

Next: retain REA corroboration; establish concrete per-type receiver bindings
and wire formats; implement bounded list append and caller's empty-list disposal;
replace the synthetic controller callback in base-receiver integration tests.
Controller application/interpolation and Qt locking remain additional work.
