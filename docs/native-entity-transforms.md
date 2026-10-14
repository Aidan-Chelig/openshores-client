# Native entity transforms: investigation ledger

Target: `reference/openshores-full/OpenShores.exe`, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
The observations below are static GNU `objdump` disassembly, not REA
decompiler evidence or observed execution. Addresses are PE virtual addresses.
The original executable was not run; no public service was contacted.

## Protocol boundary

### Native scene stream and dispatch

Static `DtClient::ReadPacket` at `0x1405e79a0` constructs a QDataStream from
the packet QByteArray at `0x1405e79c7`, then stores version `7` at stream
offset `0x18` (`0x1405e79df`). Qt's header maps 7 to Qt 4.0 and defines the
inline version setter as writing that field. This is before atom ID and base
reads at `0x1405e7ac7` and `0x1405e7b55`. Therefore the established native
scene path uses type-width floats/doubles, not a guessed Qt 5 default.
Sources: [Qt header](https://raw.githubusercontent.com/qt/qtbase/5.15/src/corelib/serialization/qdatastream.h)
and [Qt precision rules](https://doc.qt.io/archives/qt-5.15/qdatastream.html#setFloatingPointPrecision).
The two explicit precision setter calls found in GNU disassembly belong to
STL binary file read/write routines, not scene networking.

`DaAtom::Create` at `0x140b2ce40` uses a signed-offset jump table at
`0x1421f7d98`. Static byte/disassembly checks bind all 23 supported cases
through `0x28` to their actual constructors; unsupported entries go to the
warning/null path. These IDs are **not** the supplied server's opcode profile:
native person/item/globe are `0x05/0x16/0x24`, whereas the current reference
writers use `0x12/0x11/0x1f`. Native `0x12` constructs a designer, not a
person. Local-server tests never established those profiles were identical.

`src/world/native_atom/mod.rs` now supplies a separate native kind table and
unframed atom-prefix dispatcher. It uses the established scene type-width
encoding and preserves the remainder without calling source-profile body
decoders. Unknown/non-atom opcodes remain unclassified. This is not yet wired
into the connected source-server client: full native bodies, framing/version
pairing and native hierarchy application remain unfinished.

`inspect-native-atom HEX_UNFRAMED_PAYLOAD` exercises that dispatcher offline
with the established Qt 4.0 profile. Its bounded JSON output distinguishes a
known native kind/base prefix from an unclassified payload retained byte-exactly.
All 256 opcode values are checked through the CLI against an independent Python
fixture; all 23 supported kinds, unsupported/non-atom payload preservation,
truncated known prefixes and over-budget inputs are covered. The signed-offset
constructor table is separately pinned to the executable bytes. Type-specific
bodies are still opaque, including for known kinds; no source-profile fallback
is attempted when a native prefix fails.

`inspect-native-capture CAPTURE_FILE` replays native payloads in the existing
length-prefixed capture container. It reports per-frame prefix fields, untouched
tail lengths and hashes of complete raw payloads. Library records retain full
payload bytes, including unclassified packets, without self-referential storage
or inferred world state. Limits are 20,000 frames, 16 MiB/frame and 128 MiB
aggregate payload; tightened limits are tested, and aggregate allocation is
bounded before reading each payload. Empty capture, truncated framing/payload,
malformed known prefixes and unknown layers are distinguished in tests. The CLI
validates the complete capture before printing, avoiding partial success output.
An independent Python capture fixture verifies hashes and source immutability.
This is container/prefix replay, not complete native transport or body parity.

`src/world/native_prefix/mod.rs` now reads the established native ID/base prefix
through an explicit separate API, starting **after** the enclosing opcode/layer
dispatch. Static `RxUpdateId` at `0x140b27e30` reads AuId at `0x140b27e41`
then AuTime at `0x140b27e57`; AuTime at `0x14163a380` invokes the signed
64-bit QDataStream overload. These anchors are checked by the native transform
test task. The prefix reads packed position before optional create/modify
times, retains double coordinates, and exposes bit `0x20` as the in-motion
boolean. Bits `0x08` and `0x80` are preserved without inventing payloads in this
base reader; their wider-layer meanings remain unknown.

Unknown effect/sync layers stop prefix decoding explicitly: `TailKind` marks
whether the untouched remainder starts with effects, sync controllers, or the
type-specific body. Effects take precedence when both flags are set. The API
does not mislabel those unknown base layers as item/person data. All 256 base
flag combinations and every truncated known prefix are tested, including
position-before-time ordering, untouched trailing bytes and failure atomicity.
This is not native frame dispatch, ChangePos state application or complete
type-specific decoding; the working reference-server decoder is unchanged.

`inspect-native-base type-widths|single|double HEX_FIELDS_AFTER_OPCODE` exposes
this separate reader as an offline JSON diagnostic. It bounds input to 4096
bytes and reports exact consumption, field values, tail kind and untouched tail
hex. The explicit profile is mandatory; no opcode guessing or source/native
auto-detection is performed. The native transform task now also checks 768
independent Python ID/base-prefix fixtures through that command, including
unknown-layer tail preservation and malformed/over-budget rejection.

The current Rust common-atom decoder follows the supplied server writers:
base flags `1/2/4/8` mean parent/create/modify/six transform floats. This is
verified local-server compatibility, not established patched-client parity.

The native `DaAtom::RxUpdateBase` at `0x140b4e470` instead tests bit `1`
at `0x140b4e49a` and branches to `0x140b4e747`. That branch constructs an
identity `AuAtomPos`, calls `AuAtomPos::Rx` at `0x140b4e79a`, then calls
`DaAtom::ChangePos` at `0x140b4e7a5`. Bits `2` and `4` separately call
`AuTime` readers. Bit `0x10` enters the effect path; `0x20` supplies a boolean;
`0x40` enters the sync-controller path. This routine does not simply read a
parent ID followed by a six-float record under bit `8`.

`AuAtomPos::Rx` at `0x1408b2c50` reads an additional byte of flags at
`0x1408b2cc4`. Its bit `1` calls `AuId` at `0x1408b32c6`; bits
`0x10/0x20/0x40` conditionally read three rotation floats, defaulting omitted
components to zero. Position components also have conditional float reads;
bit `0x80` selects the double overload for present position components
(`0x1408b33ab`, `0x1408b3383`, `0x1408b323f`). Without that bit the
float overload is used. Rotation fields precede position fields on the wire.
Rotation floats are widened to doubles and passed directly to `sincos` at
`0x1408b2e94`, `0x1408b2eb9`, and `0x1408b2ede`: radians, not degrees.
`src/atom_position/` now decodes the serialized fields separately, with explicit
stream configuration, zero defaults for omitted components, and an optional
parent preserving omission versus explicit null. It does not yet reconstruct
the native in-place matrix update or integrate native atom dispatch. Do not
auto-detect profiles from ambiguous payload bytes
or replace the working reference-server decoder with a guessed native layout.

## Matrix conventions and world composition

`AuMatrix4x4::Rotate(AuPoint)` at `0x14176a8e0` takes three doubles and calls
`sincos` for X, Y, then Z. Its column updates mix columns 1/2, then 0/2,
then 0/1, leaving translation unchanged: postmultiplication by `Rx * Ry * Rz`.
The supplied server's `gameplay/natives/village.py::body_axes` independently
constructs the columns of that same product, with local X right, Y forward,
and Z up. This alone does not prove every model's authored axes or every atom
kind's placement behavior.

`DaAtom::EnvTransform` at `0x140b30290` maintains a cached matrix. The cache
miss obtains a parent at `0x140b30763`. One parent-dependent branch copies the
local matrix at offset `0x218`; another obtains a parent virtual transform and
multiplies it by the local matrix at `0x140b30811`. No parent produces identity.
Static vtable inspection now resolves the predicate to `CanRenderEnvironment`
and the transform call to `EnvTransform`. Composition stops at a parent that
can render an environment, not arbitrarily at every globe. Native class and
protocol-profile dispatch still need integration into Rust world state.

`DaAtom::SetPosAndOrient` at `0x140b2bc40` copies an `AuAtomPos` matrix into
offset `0x218`, then extracts its rotation via `GetRotation` at `0x140b2bce0`.
Native rotation is not simply three persistent scalar fields. The wire-reader,
change-position, interpolation, environment-boundary, and item-render paths
still need to be connected before changing the renderer.

## Next evidence

The focused REA/Ghidra request for `AuEther::ChangeParent` completed in
662878 ms. Its complete validated result is retained in
[`evidence/native-change-parent.json`](evidence/native-change-parent.json),
Evidence `ev_dfbbe0fef5b9c6fd2e622ca7b9edeca369bbf22ff043936c84b9a083aa836a46`.
It resolves the atom through two maps, updates old/new parent subatom sets,
has an `AddRunaway` path when a destination parent lock is absent, and selects
`SetPosByEther` or `SetPosAndOrientByEther` from the boolean argument before
invalidating environment transforms. This is static pseudocode, not runtime
observation; Qt container typing and helper semantics still require tracing.
Ghidra reports a terminal-flow limitation at the final invalidation call, not
proof of a non-returning helper. Missing-owner/runaway recovery remains
unported: native scene state must not replace it with a guessed copy.

The focused query for `DaAtom::SetPosByEther`
(`_ZN6DaAtom13SetPosByEtherERK9AuAtomPosPK7AuEther`, `0x140b41450`) completed in
652666 ms with Ghidra 12.1.2. Complete validated output is retained in
[`evidence/native-set-position-by-ether.json`](evidence/native-set-position-by-ether.json),
Evidence `ev_73da6cdfb3a16d534fe2ef3c746bc23d1fca3d98f0a7850a5c7875c295439684`.
It corroborates the copy/timer/old-parent callback ordering below. Its recovered
prototype mistypes the instance and incoming position and omits the unused
Ether argument; use the mangled signature and machine-code data flow, not
those recovered type labels, to identify operands. Ghidra also reports a
terminal-flow limitation at TickTock, not proof that TickTock is non-returning.

GNU disassembly now separately establishes the helper's fresh-position copy
and side-effect ordering. `SetPosByEther`, `0x140b41450..0x140b415d1`, saves
the OLD parent at `0x140b41472`, copies the incoming parent/matrix, conditionally
copies an existing inverse cache, then restarts the movement timer at
`0x140b4150d`. It checks the old parent at `0x140b4151c` and tail-calls
`TickTock` at `0x140b415cd` only when that old ID is null. It does not locally
perform the owner's later environment-cache invalidation. TickTock's global
registration/scheduling behavior is still unported; it is not a no-op.

The Rust fresh-wire primitive `set_fresh_position_by_ether_at` now copies the
validated fresh transform and owns its monotonic timer restart, returning an
explicit `tick_tock_required` obligation based on the old parent. It deliberately
leaves environment-cache validity untouched. The separate
`invalidate_environment` implements the native invalidator's leading valid-flag
gate and returns the normal-descendant invalidation obligation. Native
`InvalidateEnvTransform` checks flag 0x1d8 at `0x140b436eb`, clears it at
`0x140b4371b`, and lists/invalidates normal descendants only on that valid branch.
The ordinary local ChangePos branch reuses that cache gate but does not invent
an old-parent TickTock callback.

This is not full AuEther orchestration: caller-owned index moves, runaway
handling, TickTock, descendant traversal and their ordering remain required.
Only fresh wire transforms are accepted; this does not port arbitrary cached
inverse matrix copies. `SetPosAndOrientByEther` additionally extracts rotation
at `0x140b4169e`; that fresh copy/extract/timer primitive is now ported using
the native getter algorithm. No changes are wired into
the playable reference-profile client. Position state/helpers/tests are split
under `src/atom_position/state/`; `just test-native-transforms` pins these
anchors to the target digest. Unit tests cover old/new null and nonnull parents,
unchanged updates, deferred cache invalidation, double coordinates, timer
restarts and atomic validation errors. These are static and synthetic tests,
not observed original-client execution.

The projected input stage of `AuMatrix4x4::GetRotation` is now reconstructed
separately under `src/atom_position/rotation/`. See
[native rotation](native-rotation.md) for the exact origin denominator,
projection/cancellation order, finite safety boundary and reconstructed final
`AuPoint::CalculateRotation` algorithm. No generic Euler extraction is used
as a substitute for that native algorithm.

The focused REA/Ghidra request for `_ZNK6DaAtom8TickTockEv` at
`0x140b40610` completed in 646283 ms. Complete output is preserved in
[native-tick-tock.json](evidence/native-tick-tock.json), evidence
`ev_ace3aec82502ecbfb99baadb630d900cb65fd9e074a0dc7d26f5cc65de91a59a`.
This is static reconstruction, not observed target execution. The global byte
at offset `0x49b`, bit 1, enables elapsed-time logging; it does not disable
scheduling. `MustNotTickTock` releases both registrations. Otherwise virtual
slot `0x2a8` controls the ticker request, gated by a resolved solar system and
`IsTickingContents`; virtual slot `0x2c8` supplies a signed integer for the
tocker request, where zero, missing solar system, or nonzero solar-system byte
`0x3a9` causes release. Request paths dispatch global virtual slots `0x50`
and `0x60` with atom ID, atom type and solar-system ID (plus the raw signed
integer for the latter). Exact registration implementation, interval units,
class overrides and scheduler execution remain unported. Terminal-flow
limitations at release calls are not proof those calls cannot return.

Primary vtable bytes bind atom slots `0x2a8`, `0x2c8`, `0x2f0`, `0x4d0`
to IsTicker, IsTocker, AtomType and FindParentSolarSystem. AuGlobal slots
`0x50` and `0x60` bind UpdateOrAddTicker (`0x1415a5990`) and
UpdateOrAddTocker (`0x1415a5b80`). These GNU static bindings are pinned by
`tests/native_transform_vtables.py`. The latter's mangled unsigned argument
(`j`) establishes that all 32 bits must survive the decompiler's signed local;
negative-looking bit patterns must not be rejected or clamped to zero.

`src/world/native_tick_tock/` reconstructs the non-instrumented decision path
as ordered registration obligations. Caller-provided predicates and solar
snapshots stay explicit. It preserves getter order, retries a missing parent
for a nonzero tocker request, and reuses a resolved parent even when contents
are not ticking. AtomType is evaluated independently for each request. Tests
cover all gate combinations, high interval bits, lookup order and short circuits.
No scheduler hashes are mutated by this helper: release/request effects that
could influence later getters, concurrent locking, predicate implementations,
and scheduler execution still require integration. The decision helper is not
a replacement for fulfilling the old-null-parent callback in AuEther.

`apply_registrations` now shares the evaluator with the diagnostic plan helper,
but dispatches effects immediately through a required RegistrationContext
implementation. In the retained non-instrumented trace, ReleaseTicker or
UpdateOrAddTicker precedes IsTocker; collecting all actions and applying them
later cannot preserve effects visible to that getter. Solar predicates are
likewise called at their evaluation points, including the late `0x3a9` read,
rather than assuming the lookup snapshot is still current. Synthetic tests
change interval, atom type and solar suppression state after the ticker effect
and verify the subsequent decision. The trait intentionally supplies no no-op
default. Actual global registration storage, locks and the AuEther callback
remain unimplemented; this is the dispatch foundation, not a working scheduler.

### Existing registration field effects

GNU static ReleaseTicker (`0x1415a6bf0`) and ReleaseTocker (`0x1415a6d90`)
look up the ID without inserting it. On a match, writes at `0x1415a6c73` and
`0x1415a6e13` set node byte `0x38`, copy the 16 bytes at `0x142304550` into
node `0x28`, then set the owner cleanup flag (`0x430` or `0x448`). That
constant is two little-endian `0x8000000000000000` words, not zero or the
current time. ReleaseTickTock (`0x1415ac890`) performs the ticker operation
first, then the tocker operation, with separate locks.

Existing UpdateOrAddTicker writes solar ID and clears the release byte;
existing UpdateOrAddTocker additionally writes interval at `0x24` at
`0x1415a5c8c`. Neither existing-key branch replaces atom type or resets the
timer. Thus reactivating a released record retains invalid timer words. The
new `native_tick_tock/record/` primitives implement these established fields
without inventing the rest of the native record. Raw clock words remain raw;
no native-clock/Rust-Instant conversion is claimed. Owner cleanup flags and
missing-key insertion remain explicit unfulfilled integration work.

RemoveReleased is not unconditional erase: at `0x14160e3e8` the tocker hash
checks the release byte, then calls QMutex::tryLock(0) at `0x14160e3fa`; failed
locks retain the record, successful locks lead to erase at `0x14160e48f`.
Rust must not replace this with immediate deletion on release. Constructor
statistics/history, add-path cleanup, scheduler locking and timing need further
reconstruction. These are GNU static observations, not a REA runtime oracle.
`just test-native-transforms` pins the release constant and field writes.

The base AuGlobal registration metadata path is now implemented under
`native_tick_tock/store/`, split into access, mutation and tests. Fresh ticker
and tocker Add paths load the same invalid-timer constant at `0x14178ea97`
and `0x14178ef0d`. Existing keys update in place, preserve type/timer and do
not sweep. Missing keys insert, then sweep only if the corresponding owner
pending flag was set. The tocker call/clear is at `0x1415a5ce3`/`0x1415a5ce8`;
that flag clears even when a busy released entry survives. Ticker cleanup
also uses QMutex::tryLock(0), at `0x14160e25a`.

Rust-owned tables use real per-record mutexes and return their shared handles
to the eventual scheduler owner. Release does not block on those locks; cleanup
uses try_lock. Poisoned locks conservatively retain records (Rust policy),
and BTree traversal does not claim native hash-order/concurrency parity. Each
table is capped at 20000 entries, optionally tightened; rejected insertions
leave all flags/records unchanged. These tables cover established registration
fields only, not native statistics/history or derived global Add overrides.
They are not yet attached to a scheduler loop or the playable source-profile
client. Tests cover absent release, insertion, revival, independent tables,
busy cleanup, clearing the pending flag despite retained records, and budgets.

`RegistrationStore::apply_tick_tock` now connects the evaluator to those real
tables through `StoreContext`. Each predicate receives the current store,
including earlier successful effects. There is no intermediate plan replay or
no-op sink. A fallible evaluator stops at the first storage error before any
later getter or registration; earlier successful mutations remain applied.
That error handling is explicit Rust safety policy, not a native transaction
claim. Integration tests check real ticker/tocker insertion, later reads of the
new ticker, combined release, first-effect rejection and partial progress when
the second effect fails. Full world predicate recovery, timer scheduling,
AuEther callback attachment and playable-client integration remain unfinished.

A focused REA decompilation of `_ZN8AuGlobal17UpdateOrAddTockerERK4AuIdhS2_j`
completed successfully. Complete digest-validated output is preserved in
[native-register-tocker.json](evidence/native-register-tocker.json), evidence
`ev_2fac9b08d6e7df94e845e0b6f146a4e1eba600eeb641644dc8a12e0e67d2659e`.
It corroborates existing-key field updates, base hash insertion under the
write lock, pending cleanup and clearing the pending flag. On a missing key,
global slot `0x58` can override AddTocker; that override is not a call to the
base store and its effects remain unported. Ghidra omits the fifth argument in
the recovered indirect-call prototype. GNU assembly at `0x1415a5cbc` explicitly
places the raw interval into the outgoing stack argument before the tail jump;
do not infer that derived insertion drops the interval. Jump-table/indirect-tail
recovery warnings remain preserved in the full record. Ghidra 12.1.2, default
analysis, read-only ephemeral project; this did not execute the client.

### MustNotTickTock eligibility

GNU static non-instrumented `MustNotTickTock` (`0x140b3fdd0`) releases both
registrations for raw field `0x1c` equal to 2 or 3, or in signed-positive range
5..INT_MAX. Unsigned comparisons at `0x140b3fdfc`/`0x140b3fe09` do not include
negative values or 4. Missing owner (`0x350`), zero stored parent ID (`0x210`),
failed Parent lookup, and parent's raw field equal to 2 also return true.

Otherwise, false DoesQueryEnvironment (slot `0x668`) returns false. For a
nonnull query pointer (`0x98`), query field `0x34` must have bit `0x20` or
`0x08` to continue; neither means false, not release. A null query pointer
also continues. The parent's FindParentCapsuleInEnv (slot `0x470`) is then
queried: no capsule means false; a resolved capsule returns the inverse of
its boolean slot `0xb88`. DaSc binds this slot to IsDesignDataCacheValid;
the abstract DaCapsule table has a null slot, so do not invent a base default
or infer all overrides from that one concrete binding.

`native_tick_tock/eligibility.rs` ports this predicate over explicit native
observations. Its immutable snapshots do not claim native lookup/lock timing.
The raw status and query fields still have no established wire mapping; they
must not be synthesized from scene flags or atom kinds. Synthetic tests cover
negative/status edge cases, distinct own/parent exclusions, owner/parent
failure, query absence, both flag masks and capsule-result inversion. Static
tests pin the relevant instructions and base virtual bindings.

## Explicit normal-subatom invalidation

`src/world/native_subatoms/` implements the local ChangePos cache obligation
over explicit caller-established nodes and owner child sets. It does not yet
construct AuEther indexes from network records or implement reparenting.

GNU static observations in the same digest-bound patched executable:

- `AuEther::ListNormalSubatoms` (`0x1408c6400`) first requires the parent-lock
  map entry; child-set lookup occurs at `0x1408c64a4`, then AppendNormal at
  `0x1408c64b9`. Parent node presence alone does not establish availability.
- `DaAtomList::AppendNormal` (`0x140b78910`) tests the raw signed field at
  child offset `0x1c` at `0x140b7896a`; only zero-field members are appended.
  Its meaning and wire source remain unknown, so Rust requires it explicitly.
- List invalidation (`0x140b77920`) repeats that check and excludes members
  whose virtual `IsInstanceUniverse` slot `0x7b8` returns true. All 23 atom
  vtables are checked by `tests/native_transform_vtables.py`: only Arena and
  Designer return true. This differs from CanRenderEnv and does not exclude
  the actual Universe atom kind.
- Base invalidation (`0x140b436e0`) checks cached flag `0x1d8`, clears it
  before listing descendants, and stops immediately if it was already false.
  A directly invalidated root bypasses the list eligibility filters.

The iterative Rust traversal preserves those gates, including cycles and
shared descendants. Local position updates restart only the updated node's
timer and fulfill descendant invalidation only when its preceding cache gate
was true. Managed parent changes return an explicit proposal without mutating
nodes or indexes. Constructor budgets and missing-reference rejection are Rust
safety policy; ordered sets do not claim native hash iteration or concurrent
lock-interleaving parity. Synthetic tests cover filters, unavailable lists,
invalid caches, cycles, double precision and atomic errors.

`NativeNormalSubatoms::change_parent_in_available_lists_at` now implements
the established normal ChangeParent branch when both parent-lock/child-set
contexts are explicitly present. It removes the atom from the old set and
inserts into the destination set (unchanged parent skips those moves), applies
the selected fresh Ether setter, invokes a required TickTock callback only for
old null parent, then invalidates the root and normal descendants. The callback
sees the moved indexes and new position/timer while caches are still valid.
No owner list is invented from node existence or packed parent alone.

Geometry validation precedes index mutation (Rust safety policy). Missing
context returns OwnerContextRequired with all nodes/indexes unchanged; this is
an explicit unfulfilled native recovery branch, not a successful reparent.
Callback errors retain prior index/position effects and still clear caches;
that policy handles Rust registration safety errors, not native exception
semantics. Unit tests cover operation order, both setters, old null/nonnull
parents, unavailable context, invalid geometry and failed callbacks. Native
two-map atom resolution, lock interleaving, AddRunaway and diagnostics-driven
misplaced-set recovery remain required for full AuEther integration.

The normal branch can now call `change_parent_with_registrations_at` to
execute TickTock directly against RegistrationStore. Its world-owner adapter
creates a borrowing context only after index/position effects, and only when
the old parent requires that callback. Every getter can inspect that updated
hierarchy and the current registration store. No callback ticket is silently
discarded and no atom-kind-based scheduling predicate is invented. Integrated
tests cover real ticker/tocker insertion and combined release before descendant
cache cleanup, skipping context construction for nonnull old parent or missing
owner context, and retaining earlier effects when the tocker budget rejects
the second registration. This connects the established primitives, but native
world lookups/predicates must still be supplied by the eventual full owner;
it is not yet playable-client or scheduler-loop integration.

## Runaway pair collection

GNU static AddRunaway (`0x1408c4d10`) locks owner offset `0x28` and uses the
hash at `0x30`. Its first ID argument is the parent key, its second the child
value: node writes occur at `0x1408c4dca`/`0x1408c4dd0`. Existing equal keys
are scanned for the same child (`0x1408c4e5f`); that exact pair is a no-op.
Otherwise a new node is allocated even with the same parent key (jump at
`0x1408c4f52`), preserving other children. A child paired with another parent
is not globally replaced or deduplicated. No null-ID exclusion is present.
The saved ChangeParent trace passes the incoming parent as the first ID and
atom ID as the second on its missing-destination-lock branch.

`src/world/native_runaways/` ports this pair collection using parent-keyed
sets, with exact-pair suppression and a Rust-only 128000-pair safety budget
that callers can tighten. Duplicate insertion succeeds even at capacity;
rejected new pairs leave all state unchanged. Ordered iteration does not claim
native QHash order or concurrent locking parity. Tests cover multiple children,
one child under multiple parents, repeated pairs, null/max IDs and atomic
capacity rejection. Static binary tests pin key/value writes and same-key
allocation. Recovery/index attachment is not yet implemented by this queue.

`NativeNormalSubatoms::change_parent_to_missing_destination_at` now connects
this queue to the saved ChangeParent missing-destination-lock branch. The
caller must explicitly establish that native lock absence; it must not select
this path merely because a scene atom or child set was not supplied. A supplied
available destination context is rejected as contradictory. The path queues
the parent/atom pair first, then performs the selected Ether copy/timer,
old-null-parent TickTock callback, and cache invalidation. It deliberately
does not remove the old child-set membership or insert into the missing parent.
This follows the saved AddRunaway-to-setter jump, not the normal index-move path.

Both paths share preparation and final callback/cache helpers. Geometry and
queue-capacity rejection happen before state changes (Rust safety policy).
Callback errors retain the queued pair and updated position, but still perform
cache cleanup. Tests check queue visibility before callbacks, index retention,
orientation/timer effects, repeated-pair suppression, nonnull old-parent
callback skipping, validation failures and callback errors. FixRunaways repair,
native owner-map lifecycles and lock interleaving remain unimplemented; the
stable-owner recovery helper is described below.

The missing-destination branch also has a direct registration-store adapter,
`change_parent_to_missing_destination_with_registrations_at`. Runaway and
registration stores are passed as separate borrowed resources; they do not
imply parent-lock availability. Its world context is constructed only after
queue/position effects and only when old null parent requires TickTock. Tests
connect real insertion and release effects, confirm old indexes remain intact,
retain queued/earlier effects after second-registration rejection, and verify
queue rejection skips all getter/registration work. Registration integration
tests are split under `native_subatoms/registration_tests/`. Parent-arrival
recovery and playable-client attachment remain separate unfinished work.

A focused REA/Ghidra request for `_ZN7AuEther11FixRunawaysERK4AuId` completed
in 648436 ms. Complete digest-validated output is preserved in
[native-fix-runaways.json](evidence/native-fix-runaways.json), evidence
`ev_a829e6212979114d102c2da8aaa75edec0c1965247e1ba04e5b58cb8e16bf514`.
Ghidra 12.1.2, default analysis, read-only ephemeral project. This is static
pseudocode, not target execution; Qt container/prototype recovery limitations
remain in the full record.

FixRunaways first extracts and erases all pairs for the requested parent into
a temporary child-ID set, then attempts attachment. It looks up each child in
the primary atom map, then the secondary only when the primary lookup fails.
A primary atom whose stored parent differs does not cause a secondary retry.
Only a matching stored parent, existing parent lock and existing child set
allow insertion. Missing/stale objects and unavailable owner sets are consumed,
not requeued. This path does not copy position, restart timers, invoke TickTock,
invalidate environment caches, or remove stale old-parent set memberships.

`native_subatoms/recovery/` now reconstructs this stable-owner behavior, with
explicit bounded ID-to-node maps and a whole-parent queue extraction. It
reports consumed child IDs and newly inserted node handles. Ordered maps/sets
do not claim native QHash order or lock interleaving. Rust preflight rejects
dangling handles and link-budget overflow before consuming the batch; that
validation is safety policy over immutable observations, not a native rollback
claim. Tests cover primary precedence, secondary fallback, stale/missing child
consumption, unavailable sets, repeated attachment, unrelated parent batches,
unchanged position/timer/cache state and atomic invalid-context rejection.
Owner registration on parent arrival, native map lifecycles and playable-client
attachment remain unfinished.

## Atom registration routing (in progress)

GNU static AddAtomA (`0x1408cd560`) locks owner offset `0x18` and calls
AddAtomLocked (`0x1408cc0a0`). Its initial virtual slot `0x618` is InStarField:
true selects the secondary map at owner `+8`, false the primary map at `+0`.
All 23 supported atom vtables return constants: Galaxy, Sector, SolarSystem
and Universe are true; all others false. This differs from CanRenderEnvironment
and IsInstanceUniverse. NativeAtomKind::in_star_field and the offline prefix
diagnostic expose that predicate, with all bindings/returns pinned statically.

NativeRecoveryLookup::from_routed_nodes can now build its explicit two-map
context from caller-established IDs, node handles and kinds using that routing.
It retains primary precedence and allows an explicit cross-map overlap while
rejecting duplicate IDs within a map (Rust validation, not native replacement
policy). It does not implement AddAtomLocked's lifecycle by constructing maps.

The observed shared registration tail creates the atom's own child set and
parent lock, assigns owner pointer `0x350` at `0x1408ccbca`, then calls
FixRunaways at `0x1408ccbd9`. Duplicate-object branches include RemoveAtomLocked,
setting the old object's raw field `0x1c` to 6 and calling TickTock; ownership,
replacement/timer cleanup and auxiliary indexes require further helper review.
The focused REA request for `_ZN7AuEther13AddAtomLockedEP6DaAtom` completed
in 655786 ms. Complete validated output is preserved in
[native-add-atom.json](evidence/native-add-atom.json), evidence
`ev_c9f29fca307a3c0bd42e0851752f7694780f3a45149925a1329d93db94b7e676`.
Ghidra 12.1.2, default analysis, read-only ephemeral project; static pseudocode,
not target execution. The record corroborates null-pointer/null-ID rejection,
selected-map routing, same-pointer early return, replacement cleanup and the
shared owner tail. Replacement invokes RemoveAtomLocked, marks the old raw
status 6, invokes TickTock and retains a started elapsed timer in owner `0x48`;
it is not a simple overwrite of the ID map.

`native_subatoms/arrival/` ports the fresh shared owner tail after explicit
routed-map insertion. The atom must already exist at the supplied handle in
the correct native map, and its own list must not exist. Initial attachment
checks parent child-set presence, not parent-lock availability. Missing sets
queue the parent/atom pair. The helper then creates its own available empty
list and runs recovery against that newly established owner context. No
position, orientation, timer, cache or incoming-atom scheduling effect is
invented. A self-parent pair is queued and recovered in the same tail, matching
native behavior rather than imposing an unestablished cycle rejection.

Rust validates references, queue capacity and final link growth before changing
state. Tests cover child recovery, primary/secondary routing validation,
attachment to a child set without an available lock, missing-parent queuing,
self-parent recovery and atomic safety errors. Existing own lists are rejected
as requiring full registration lifecycle context, not silently replaced.
Owner-pointer assignment is represented by the explicit modeled availability
transition; raw pointers, locks, atom construction/map insertion, same-object
re-add policy and replacement/removal remain outside this fresh-tail helper.

`establish_fresh_atom_owner_with_assignment` adds an explicit assignment point
for caller-held owner metadata: after parent attachment/own index creation,
before FixRunaways. Its callback sees the new empty own set before recovery;
validation failures never call it. The original graph-only helper delegates
without external assignment. This ordering follows the retained AddAtomLocked
owner-field store and subsequent FixRunaways call, not a native runtime trace.
A separate lifecycle test removes an old parent handle, supplies a new routed
map explicitly, assigns the new handle's owner and recovers queued children.
Previously listed but unqueued children are not magically restored; all object
positions/caches remain unchanged and released registrations stay released.
This connects the removal and fresh-tail adapters, but does not implement the
intervening replacement status-6, TickTock and timed-retention steps.

`native_subatoms/retirement/` now ports that old-object stage separately as
`retire_replaced_atom_at`. After caller-established removal, it receives a
monotonic timer epoch, writes raw field `0x1c = 6`, constructs the explicit owner
registration context over the updated object and applies TickTock to the old
wire ID. Only then does it retain the epoch in `NativeRetiredAtoms`, keyed by
object handle rather than wire ID. Both recovered AddAtomLocked map branches
perform this sequence before incoming-map insertion. Context observations remain
caller supplied; no native getter is replaced with an inferred class default.

The retention table is bounded by the existing 20,000-object Rust safety limit;
existing handles replace their epoch even at capacity. Invalid handles or a new
entry exceeding capacity fail before status/context effects. Registration errors
retain the changed status and earlier registration effects but do not add the
retention record, an explicit Rust error boundary rather than native rollback.
Synthetic tests verify context construction sees status 6, registration-release
observations, two old handles with the same wire ID, timer restart, unchanged
position and safety preflight. No expiry threshold, destructor, Qt timer word
layout or mutex/concurrency behavior is invented.

`native_subatoms/lifecycle/` now connects the stages as `add_atom_owner` over
already constructed graph nodes. It rejects null IDs/missing incoming objects,
selects only the incoming kind's native map, and returns AlreadyPresent for an
identical handle before any owner, clock, queue or registration callback. A
different existing handle goes through owner removal, then samples the supplied
clock, changes status/executes TickTock and records timed retention. Only after
that does it insert the incoming routed entry, establish its own indexes/owner
and recover runaways. Fresh insertion skips old-object registration/retention.
The unrelated map remains independent; recovery still uses primary precedence.

`NativeAtomOwnerContext` combines explicit external detach/assignment with the
existing registration predicate context; no owner reference is inferred from
membership. Synthetic sequence tests cover both map routes and prove the
clock is sampled after detach, context sees old status 6 and detached ownership,
assignment sees the newly created empty set, and only queued children recover.
Other tests cover fresh insertion, same-handle re-add even with absent lists,
cross-map overlap, null/dangling identities and arrival-budget failure after map
insertion. Rust errors stop at the failed stage and retain previous writes;
this is not a transaction. Tests are split under `lifecycle/tests/`.

This implements the recovered stable-owner sequence, not full native world
dispatch. Constructors, wire-ID/handle establishment and caller metadata still
require integration. Inconsistent preexisting own lists with no selected-map
replacement are rejected by the fresh-tail helper after insertion rather than
reproducing native duplicate allocation behavior. Qt locks, copy-on-write,
diagnostic output, retained-object expiry/destruction and full native packet
construction remain outside the adapter.

The focused REA request for `_ZN7AuEther15DeleteTrashAtomEv` at `0x1408cd9c0`
completed in 640533ms. The complete retained record is
[native-delete-trash-atom.json](evidence/native-delete-trash-atom.json), Evidence
`ev_5e01f84659ed6bda262425e6a81e300e0a53d6fe3222fdccf0000f4b9007013c`,
bound to the patched executable digest above. Full pseudocode has been reviewed;
this is static Ghidra 12.1.2 evidence, not observed client execution. Cleanup
implementation below preserves the following ordering constraints.
Independent GNU static bytes (not a REA oracle) pin an initial selection branch:
subtract reference milliseconds, skip while signed difference is at most 19999,
query InTickTockQueue when older, restart the stored epoch if queued, otherwise
erase the selected retention entry. Normal non-instrumented execution then
calls the object's virtual slot `0x20`. These anchors alone do not establish
whole-function cleanup semantics, timer precision parity or hash iteration order.
`just test-native-transforms` verifies these anchors against the immutable
patched executable digest without executing it.

The complete REA body corroborates that the cleanup-call clock is sampled once,
then retention entries are scanned in native QHash order under the trash mutex.
Young entries are skipped without a scheduling-queue query. Old entries still
in InTickTockQueue get the cleanup-call timer copied into their retention record,
then scanning continues. The first old unqueued entry is erased, the remaining
count is captured, and the trash mutex is released. No further entry is selected
in that invocation. A null selected pointer returns without a destructor call;
otherwise non-instrumented normal execution invokes virtual slot `0x20`.

Global `0x49a & 0x08` enables additional reference checks (distinct from removal's
all-list cleanup bit `0x10`). FindAtomA resolving the same object prevents
destruction. Otherwise the routine scans child sets reached through the
parent-lock map and records the containing parent's ID. If that recorded ID
is zero, it scans the InStarField-selected atom map for the same pointer under
another ID. Nonzero containing-parent or alternate-map IDs prevent destruction;
zero remains the native sentinel even if a matching map/list key is zero. These
are checks, not reference cleanup. The retention record has already been erased
on every guarded branch: blocked destruction does not requeue/restart it and
returns the captured remaining count. Logging bit `0x4fb & 0x20` changes reporting,
not the initial retention-selection algorithm.

The Rust adapter must retain the one-selection limit and conditional guard
ordering, not sweep all expired objects or automatically repair references.
Native iteration order and reference-millisecond subtraction remain caller
observations until those dependencies are ported. `Instant` elapsed rounding
alone is not a Qt reference-millisecond oracle, especially at a submillisecond
boundary. InTickTockQueue's actual lookup predicate and full derived destructor
effects still need separate reconstruction; registration presence or released
flags must not be substituted for queue membership.

`NativeRetiredAtoms::delete_one_in_order` now ports the selection and guarded
disposal sequence under `native_subatoms/retirement/cleanup/`. It validates an
explicit complete iteration-order snapshot before effects (Rust safety), uses
caller-supplied signed reference-millisecond differences, and queries queue
membership only for entries aged at least 20000ms. Protected entries get the
single cleanup-call epoch copied into their stored timer. The first unqueued
old entry is removed before the null/reference checks and required destruction
callback. It returns the remaining count, restarted handles and selected
disposition; guards never restore a removed record or continue to another object.

The context explicitly supplies FindAtomA identity, the first containing parent
through lock-map iteration, and selected-map pointer lookup under another ID.
Calls are short-circuited in native order, including the zero-ID sentinel.
No reference cleanup, implicit BTree iteration choice or registration-based
queue predicate is introduced. Tests cover exact/negative/extreme ages, all
queued entries, ordered restart/selection, no later getters, guard precedence,
zero sentinels, erased tracking before destruction, empty stores and atomic
invalid-order rejection. These are synthetic adapter tests, not native execution.
Native queue lookup, clock profile, world index observations, derived destructor
effects and owner scheduler invocation still need connection to the Rust client.

The lifecycle integration regression now runs actual Rust replacement through
the retention table and multiple cleanup calls: initial calls protect the old
handle and restart its timer; a later call selects and disposes it after
registration cleanup. It verifies the new routed handle, assigned owner,
own list and position survive.
The test's disposal callback removes a plain graph node only; it is explicitly
not a port of native derived destructor side effects. Age observations
are synthetic and do not claim clock parity or scheduler execution. This connects the
implemented stages without supplying invented defaults to the production adapter.

That regression now uses the real RegistrationStore membership getter instead
of a synthetic protection boolean. A released ticker keeps the old handle
protected; a busy-mutex sweep clears its pending flag but retains the record,
so a second cleanup call restarts the old handle's timer again. Unlocking alone
does not sweep it. A new release plus another missing-key insertion performs
the actual sweep, after which cleanup disposes the old handle and preserves the
replacement. The test explicitly maps old handle 70 to wire ID 7, not the new
lookup handle 71. Only the clock and plain-node disposal remain synthetic.

Full independent GNU disassembly of InTickTockQueue at `0x1415a69d0` through
`0x1415a6bea` shows ticker-ID lookup at owner `0x438`, returning true on a found
key, followed only on absence by tocker-ID lookup at `0x450`. Neither record's
release marker is read. `RegistrationStore::in_tick_tock_queue` now implements
that stable key-presence predicate. Released, busy and cleanup-deferred records
remain queue members until their table actually erases them. Tests cover both
tables, separate sweeps, zero/max IDs and busy cleanup with cleared pending flags.
Static anchors are hash-pinned in `just test-native-transforms`; this is GNU
assembly evidence, not REA pseudocode or runtime observation.
The focused REA corroboration for `_ZN8AuGlobal15InTickTockQueueERK4AuId`
completed in 637128ms. Complete evidence is retained in
[native-in-tick-tock-queue.json](evidence/native-in-tick-tock-queue.json), Evidence
`ev_cb9e87cc500860308656cb383bbd9e5ebe277c95ddb49f5be1982aed3feeb17e`,
bound to the patched executable digest above. The complete Ghidra 12.1.2
pseudocode corroborates ticker-first, tocker-second lookup with no record release
field reads. This is static reconstruction, not target execution. Recovered
Qt detach/end-pointer comparisons remain provider/native COW details, not Rust
concurrency parity; the Rust getter represents stable owned tables. Native
locking/copy-on-write, scheduler execution and full cleanup-world integration
remain separate.

## Existing native wire-position receiver

`world/native_position_updates/` now connects a decoded NativeAtomPrefix's
fresh base position to the established local, normal-reparent and explicit
missing-destination paths, including real registration storage. Existing nodes
on the direct apply_at path must be keyed by the wire ID and have matching native
kind. Missing objects or
kind changes are rejected before mutation as requiring lifecycle handling;
the direct-ID path rejects unknown identities instead of guessing aliases.
`apply_resolved_at` additionally accepts a caller-established object handle
(for example, explicitly observed through NativeRecoveryLookup::routed_entry).
It validates the handle's kind, mutates graph membership/position by handle,
constructs owner context by handle, and applies registrations using the packet's
wire ID. Missing-destination queues also retain that wire ID, not the handle.
The receiver does not invent which lookup map or fallback native packet dispatch
should choose; resolution remains explicit. The original apply_at path delegates
with handle equal to wire ID and retains its prior behavior.

`NativeAtomIdentity` supplies that distinction to the missing-lock owner helper.
Its direct-ID convenience method preserves existing callers. Alias regressions
cover local/deferred updates, normal reparent, missing-lock queuing, real
registration IDs, owner-context handle and recovery's consumption of wire IDs.
No constructor, body decoding or automatic packet-driven replacement is inferred.

## Native object construction investigation

The focused REA analysis of the default base constructor `_ZN6DaAtomC2Ev` at
`0x14184a920` completed in 655826ms. Complete evidence is retained as
[native-base-atom-constructor.json](evidence/native-base-atom-constructor.json),
Evidence `ev_07e78390a2b30475edca11a4fe7e5e553ead74a59b2c2ab6c897e731d07ede70`,
bound to the patched executable digest above. Full Ghidra 12.1.2 pseudocode was
reviewed and corroborates the independently inspected GNU constructor stores.
This is static reconstruction, not execution of the supplied client.
Independent GNU full-function review through the normal
return at `0x14184ac5a` shows initial ID `0x14 = 0`, owner `0x350 = 0`, position
parent `0x210 = 0`, identity local matrix at `0x218`, zero orientation fields
at `0x1f8..0x208` and invalid movement-timer words at `0x320`.
The qword constant loaded from `0x14224be08` and stored at `0x18` is
`0x0000000500000000`, establishing raw field `0x1c = 5`, not a zero default.
This matters to normal-subatom filtering and TickTock eligibility; existing
explicitly supplied graph snapshots must not silently acquire zero constructor
status. Hash-pinned static checks cover the relevant stores and constants.

`NativeBaseAtomInitialization::from_default_base_constructor` now exposes the
established lifecycle/position subset: null wire ID, raw status 5, absent owner,
zero received-time storage, null parent, identity local and environment matrices,
explicit zero orientation, false environment-cache validity and invalid movement
time. NativePositionState::default retains its separate unobserved-orientation
baseline; constructor initialization does not silently change existing snapshots.
Tests check every matrix element, known zero versus unknown orientation, invalid
timer versus a first unowned position update's restarted timer, and retained raw
status/owner/ID while applying that position-only stage.

Native atom dispatch and construction now live under `world/native_atom/`, with
the constructor model/tests in their own subfolder. This is not the entire
native allocation: QObject/Qt locks, unmodeled fields, derived atom initialization,
packet ID/time assignment and receiver status transitions remain separate.
No derived kind is guessed, nor is a constructor snapshot automatically inserted
into the world or registered with an owner.

Native identity decoding now lives under `world/native_prefix/`. Its separate
`NativeIdentityPrefix` reads just the 12-byte ID/time stage and stops before
base flags. NativeBasePrefix delegates to it without changing wire layout.
`NativeBaseAtomInitialization::apply_received_identity` assigns both values,
including raw null IDs and decreasing/negative times, without inventing a stale
filter or changing status, owner or position. Full GNU review of RxUpdateId
`0x140b27e30..0x140b27e5b` establishes only AuId::ls into `0x14` followed by
AuTime::ls into `0x1e8`, with no acceptance branches. Helper overloads preserve
32-bit identity bits and signed 64-bit time. These are independently hash-pinned
assembly observations, not a REA runtime oracle. Rust truncated-input rejection
is atomic, unlike native direct field writes/Qt stream error handling.

Tests cover raw width extremes, null identity, negative/decreasing time,
untouched trailing bytes, all truncated identity prefixes and retained constructor
state. Packet-driven acceptance remains separate: a focused REA analysis of
`_ZN8DtClient10ReadPacketERK10QByteArrayR10QTcpSocket` at `0x1405e79a0` is running
with full output reserved as `target/native-read-packet.json`. That pending
result must be reviewed before claiming native receive/creation policy.

DestinationContext remains caller-established. Unresolved context returns the
unchanged reparent obligation, and insufficient normal-list context remains
OwnerContextRequired rather than being counted as success. Position omission
does nothing. The native base-position path selects position-only setters,
retaining separate orientation scalars rather than inventing an extraction.
Opaque body layers, metadata fields, timestamps and item updates remain outside
this receiver; no source-profile fallback or timestamp filtering is introduced.
Tests decode synthetic native payload bytes before applying local/reparent/
runaway effects, preserve double coordinates and opaque tails, check real
registration updates, and cover omission, identity errors and unresolved owners.
This connects wire decoding to owner primitives, not complete packet dispatch
or playable native-client integration.

NativeCapture::replay_positions now applies retained prefixes through that
receiver in frame order, against caller-owned graph/runaway/registration state.
The caller supplies destination observations and monotonic time per position;
wire timestamps are not converted to local timer epochs or used as an invented
stale-update filter. Non-atom frames and position omissions are counted without
body interpretation. Outcome records retain their frame index/atom ID and can
contain unresolved hierarchy obligations; they are not all counted as applied.

Replay checks actual frame/payload budgets again because the capture's frame
vector is public. On failure it returns the frame index, prior outcome records
and the underlying error, retaining earlier state effects and stopping before
later context evaluation. A failing frame may retain partial effects under the
receiver's callback-error policy. Tests cover decreasing wire timestamps,
double precision, opaque byte preservation, unresolved outcomes, partial replay
on missing objects and pre-mutation frame-budget rejection. This is opt-in
position-stage replay, not complete native packet/world reconstruction.

`NativeCapture::replay_resolved_positions` extends that replay with fallible
caller-selected wire-ID-to-handle resolution. It invokes the resolved receiver,
while outcome IDs remain the captured wire identities. The original direct-ID
replay delegates to this implementation, preserving its API and budget/error
rules. Resolution failures identify the failing frame, retain earlier outcomes
and state, and stop before later lookups. Position omissions still skip caller
resolution entirely. No lookup-map fallback, object creation or replacement is
inferred from a capture.
Alias capture tests use an explicit routed lookup from wire ID 1 to handle 10,
verify exact raw payload preservation, skip a no-position frame, and stop on a
missing wire ID before a later update can overwrite prior position. A separate
test retains an unresolved reparent outcome without moving the aliased object.

An additional failure regression fills the tocker budget before replay: the
failing reparent frame retains its new position, parent membership, timer and
successful ticker write, while returning only earlier completed outcomes.
Cache cleanup still runs and the next frame's context is never evaluated.
Replay tests now live under `native_capture/position_tests/`. The receiver also
rejects a position-bearing null wire ID even if a caller supplies a contradictory
node keyed at zero; position omissions remain a no-op.

The position receiver's lifecycle regression now decodes child and arriving
parent native payloads, applies the child through the missing-destination path,
then establishes the fresh parent owner and recovers its queued child. Explicit
fixture metadata supplies the currently unestablished status/owner observations.
The missing-parent TickTock predicate releases real registration records before
cache cleanup. Parent arrival consumes the queue without replaying movement or
changing position epochs, cached-valid flags or registrations. Opaque tails
remain byte-exact, and repeated recovery is empty. These are synthetic integration
tests, not observed native-client execution; tests are split under
`native_position_updates/tests/`.

The focused REA request for `_ZN7AuEther16RemoveAtomLockedEP6DaAtom` at
`0x1408c9950` completed in 654353ms. Its complete retained record is
[native-remove-atom.json](evidence/native-remove-atom.json), Evidence
`ev_5a2365e286e2eda884b683f18c28f11210c9aa3277e391a2ef142ca8960cad4b`,
bound to the patched executable digest above. The full pseudocode was reviewed;
this is static Ghidra 12.1.2 evidence, not target execution or recovered source.

Removal clears the supplied object's owner pointer at `0x350`, calls global
ReleaseTickTock for its wire ID, then erases that ID from the map selected by
InStarField. The recovered lookup is keyed by ID, with no observed comparison
against the stored object pointer. It reads the supplied object's stored parent
and removes that object pointer from the parent's child set if present. Global
flag `0x49a & 0x10` enables a further scan removing the same pointer from every
child set (with diagnostics for unexpected membership); this is conditional,
not the unconditional normal path. Finally it deletes/removes the object's own
parent-lock entry and child-set entry keyed by its wire ID.

No recursive child destruction, child reparenting, runaway-queue cleanup or
position/cache changes appear in this function's recovered body. Replacement's
old-object status change, TickTock call and timed retention belong to the caller
AddAtomLocked, not to this removal primitive.

`NativeNormalSubatoms::remove_atom_owner` now implements these stable-owner
effects using separate wire ID and object handle arguments. A required caller
callback clears the externally held owner reference before real registration
release, routed map erasure and child-list cleanup. The graph retains every
object node and its unchanged position/status/cache; it does not touch runaways.
The result distinguishes erased map handle, direct parent removal, diagnostic
extra memberships and the discarded own child set. Null IDs are not rejected
by this primitive: unlike AddAtomLocked, the recovered function has no null-ID
guard. Missing Rust object handles fail preflight before the callback or writes.

Synthetic tests exercise both map routes, overlapping IDs with different
stored handles, unavailable parent locks, conditional stale membership cleanup,
retained child objects and runaway pairs, registration release, absent entries
and invalid-handle preflight. This remains an explicit adapter, not full native
world lifecycle dispatch. The Rust graph combines child-set storage and lock
availability, so deletion of both native entries is represented by removal of
that combined entry; separate lock destruction, Qt copy-on-write/concurrency,
logging and replacement timed retention remain unimplemented. Owner identity
is caller-owned rather than guessed from graph membership. Implementation and
tests are separated under `native_subatoms/removal/`.

Static `DaAtom::ChangePos` at `0x140b44d70` compares the incoming AuAtomPos
parent ID with the stored parent at `0x140b44db9`. If it differs and an owning
AuEther is present, it delegates to `AuEther::ChangeParent` at `0x140b44eaf`
instead of taking the local-copy path. Otherwise it copies parent and local
matrix, restarts a movement timer at `0x140b44e65`, and invalidates the cached
environment matrix and normal descendants when that cache was valid. These
are GNU static observations; the local-copy/timer/cache branch is ported,
but managed reparenting remains explicit and unimplemented. Native RxUpdateBase initializes a fresh AuAtomPos before reading it,
so an omitted packed parent there starts null, not automatically the entity's
previous parent. The field-only codec retains omission separately and does not
pretend to implement these state effects.

A focused REA/Ghidra analysis of `ChangePos` completed in 647716 ms. Complete
validated output: [native-change-position.json](evidence/native-change-position.json).
Evidence ID: `ev_d923b01d16b006a4eb70b9ab784b256baf7d59bce5607ba02de98ca274ebb7c7`.
It corroborates the parent-change delegation, local matrix copy, movement-timer
restart and conditional normal-descendant cache invalidation. Ghidra 12.1.2,
default profile; the ephemeral analysis closed without modifying source bytes.
Recovered pseudocode is not original source or runtime observation. The bridge
notes a terminal-call flow-model limitation at the ChangeParent tail call;
that does not establish that ChangeParent is a non-returning function.

`src/atom_position/state.rs` now ports the fresh wire-position local-copy and
delegation decision. A managed parent change returns a reparent obligation
without mutating old position/cache state. Same-parent or unmanaged updates
copy the double local matrix and report timer restart plus conditional
normal-descendant invalidation. A fresh omitted parent becomes null, not the
previous parent. These obligations are not silently treated as completed:
AuEther reparenting, descendant traversal and cached
matrix inverse handling remain unfinished. Only fresh decoded wire positions
are accepted by this path, not arbitrary preexisting AuAtomPos matrices.

`NativeAtomPrefix::apply_fresh_position` connects decoded native prefixes to
that state primitive without parsing opaque layers or mutating the source-
profile client. Tests cover managed/unmanaged changes, identical updates,
valid/invalid environment caches, omission/null-parent behavior, preserved
doubles, invalid input atomicity and a decoded-packet-to-local-state path.

The state primitive now owns a monotonic position-update epoch. Every successful
local-copy update restarts it, even when geometry is identical; reparent
delegation and rejected geometry leave it unchanged. An explicit injected
`Instant` path supports deterministic tests. Age is unknown before a known
update and if a supplied clock precedes that update; no wire timestamp or
wall-clock arithmetic substitutes for the native elapsed timer. Tests cover
age/reset behavior, identical updates and failure/delegation preservation.
Velocity integration and interpolation remain unimplemented.

A focused REA/Ghidra decompilation of `DaAtom::EnvTransform` completed in
661421 ms; the complete validated JSON is retained in
[native-entity-env-transform.json](evidence/native-entity-env-transform.json).
Evidence ID: `ev_f3402ecf8edc72405a9486ec6281b539fa6604ac58f2db298d3cf9a6fbb5f2a6`.
It corroborates the cache, parent-dependent local/cumulative branches and
identity-without-parent observations above. The recovered multiplication
prototype omits an argument;
operand order is anchored by the static assembly, not guessed from that
prototype. Ghidra 12.1.2, default-analysis profile digest
`8e3a14075ec67e0acc0e527cf433a32154bc1e12800cc3154deb7d62e385ced2`;
pseudocode is not original source or runtime observation. The bridge reports
verification against 12.1.4, with 12.1.2 accepted as an observed version.
The ephemeral analysis closed on completion; original bytes were not modified.
Next inspect the complete `AuAtomPos::Rx` and `ChangePos` paths, establish an
explicit protocol profile, and test native packed records independently of the
reference server's six-float fixtures. Keep the existing minimal local client
working while adding native behavior.

## Codec verification boundary

`tests/native_transform_vtables.py` reads immutable PE bytes and resolves
primary vtable slots `0x348` (`EnvTransform`), `0x4a8`
(`FindParentPlanetInEnv`), `0x608` (`GetTransformRelativeToParent`) and `0x7b0`
(`CanRenderEnvironment`) for nine explicitly selected classes. The primary
Itanium ABI address point is 16 bytes after the vtable symbol; section-zero
unresolved symbol aliases are excluded. These are static byte/symbol checks,
not additional REA Evidence or runtime dispatch observations.

`DaAtom`, `DaItem`, `DaPerson`, `DaWorld`, `DaWorldGlobe`, `DaWorldLL`, and
`DaVehicle` use `DaAtom::CanRenderEnvironment` at `0x14193f630`, which returns
false (`31 c0 c3`). `DaSector` at `0x1419469e0` and `DaSolarSystem` at
`0x14190d5a0` return true (`b8 01 00 00 00 c3`). Solar systems additionally
override `EnvTransform` at `0x140bc3c60`; that implementation has not yet been
traced. No behavior for uninspected classes is inferred from this list.

The extended static check now also inspects the `CanRenderEnvironment` slot
for every one of the 23 native factory classes and verifies the machine-code
constant return value. Exactly arenas, designers, sectors and solar systems
return true. `DaArena` uses `0x141941fa0`; `DaDesigner` inherits
`DaDesignBase` at `0x141906050`. Design planes are false, not designers.
`NativeAtomKind::can_render_environment` ports this complete selected-image
capability mapping, and the native CLI reports it. Independent opcode fixtures
check the Rust values while vtable/code-byte checks bind them to the executable.
This predicate does not establish all classes' own EnvTransform overrides.

`src/atom_position/environment.rs` ports the three established base
composition branches with explicit `ParentEnvironment` input: missing parent
produces identity, a boundary parent uses local unchanged, otherwise compose
parent environment * local. Tests distinguish the branches using a rotated,
translated parent (not just commutative translations), and check finite-input
and overflow safety. Missing parent ignores local, matching the observed native
branch. Parent lookup, class dispatch, caching, interpolation and scene wiring
remain separate unfinished work; this API does not silently assign profiles.

## Direct-globe item rendering

Static `DaItem::RenderItem` disassembly calls slot `0x4a8` at
`0x140b85840`, retaining the returned planet in R14. Its non-ring-section path
initializes an identity matrix at `0x140b86156` and calls slot `0x608` at
`0x140b861bf`, passing that planet as the ancestor. The mapped base routine
`GetTransformRelativeToParent` at `0x140b2e710` compares atom identity with the
ancestor, returns success unchanged when equal, otherwise left-multiplies the
local matrix into the supplied accumulator and recurses into the parent.
The ring-section path is distinct and not ported here. Source/assembly anchors
are checked by `just test-native-transforms`.

The current GUI already limits displayed items to direct children of its home
globe. For those mounted item meshes, `src/client_ui/placement.rs` now applies
the received local rotation using the established `Rx * Ry * Rz` convention.
It builds a Bevy quaternion from the explicit matrix columns, avoiding an
assumed Euler API ordering, and scales only translation (mesh coordinates are
already display-scaled). It does not re-align objects radially or change their
reported transforms. Bevy-axis tests include nonzero combined angles and a
south-pole orientation; unrepresentable placements are rejected.

This is source-compatible direct-globe placement, not full native packet,
nested-parent, original item animation, ring-section, material or model-axis
parity. The local GUI smoke checks actual mounted entity transforms against
received fields, while synthetic unit tests provide nonzero-angle coverage.

`WorldState::transform_relative_to` now exposes the corresponding local-chain
composition for explicitly requested ancestors. It excludes the ancestor's own
transform, applies parent rotations to child translations, and leaves missing
nodes/transforms and unrelated ancestry unresolved. A bounded traversal rejects
cycles even if callers mutate the public entity map directly. This operates on
the current source-profile raw local fields; it does not choose a native frame
profile or broaden graphical visibility to unverified nested entity kinds.
The world implementation and tests now live under `src/world/`. The source
profile is further separated into `atom.rs` (opcode/prefix/body decoding),
`state.rs` (sparse entity state and scene ingestion) and `capture.rs` (bounded
capture replay), with the existing public API re-exported by a small `mod.rs`.
These files explicitly label the source-server profile; refactoring does not
select the patched-native protocol or change receive-order merge behavior.

The standalone codec tests all 256 flag combinations under three explicit
QDataStream configurations, every truncated prefix, trailing-field boundaries,
negative zero, explicit null parent, and finite/narrowing safety checks. They
are synthetic byte-layout tests, not execution of the original client.

`just test-native-transforms` additionally checks 768 independent Python
`struct` fixtures through the Rust CLI and pins reader call/flag anchors to the
exact patched executable hash. It checks malformed/trailing records too.
This is not a Qt runtime oracle or native in-place matrix parity test.

Inspect a single packed record without network access:

```sh
cargo run --offline -- inspect-atom-position type-widths 13F12345673F000000C0000000
```

This literal is parent `0xf1234567`, rotation `[0.5, 0, 0]` radians and
position `[-2, 0, 0]` under the explicit legacy/type-width stream profile.
Other supported profiles are `single` and `double`. The command requires one
complete record, rejects trailing bytes, and bounds hex input to 53 bytes.
It does not accept an entire scene packet or select a native network profile.

`PackedPosition::local_matrix` and the matching `dump-atom-matrix` command
construct a fresh column-major `T * Rx * Ry * Rz` affine matrix. Translation
is not rotated, angles are widened from their float destinations, and no
extra radial/gravity alignment is invented. This conversion matches the
supplied server's `body_axes` for all 768 profile/flag fixtures. Additional
`gravity_align_euler` source fixtures verify local +Z points outward at both
poles and off-axis locations after float narrowing. Unit tests check the
rotation basis is orthonormal and a positive Z quarter-turn has the expected
sign. These establish local convention/source agreement, not native
environment composition, interpolation, or arbitrary existing-matrix updates.

Qt's float/double overload names do **not** alone prove the number of wire
bytes: Qt 4.6+ precision settings affect both overloads, while older versions
use their type widths. The codec therefore requires a caller-selected
`FloatEncoding`; raw diagnostic callers remain explicit. The native atom
dispatcher now selects type widths from the scene reader evidence above.
Source: [Qt 5.15 QDataStream precision documentation](https://doc.qt.io/archives/qt-5.15/qdatastream.html#setFloatingPointPrecision).
DoublePrecision float fields are still narrowed to f32 before storing them,
matching the native float destination. Non-finite values and f32 overflow are
rejected without consuming input as explicit Rust safety limits.
