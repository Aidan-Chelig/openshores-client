# Ordered native Person suffix

NativePersonSuffixState::receive_suffix implements the Person-specific ordered
stages after Sentient. It is not a replacement for Unit/Creature/Sentient,
native base construction, object dispatch or presentation. The active goal
still requires those stages to be connected to the real client.

Target identity and retained REA/GNU evidence are indexed in native-person.md.
The full Person update assembly at140a42db0 corroborates flags, call order and
stream overload widths. This milestone reuses those records; it did not open
another Ghidra analysis or execute the native target.

## Implemented stages

Flags A are followed immediately by B only when A80 is present. Identity IDs,
nullable strings, the actual GlobeLLF reader and bool precede HitNotify. A04
reads a short: only a change in its low15 bits updates660 and restarts668;
its high bit replaces839 even without a restart. A08's byte C always replaces
838 with its low6 bits, explicitly zeroing838 when A08 is absent.

A10 uses one packed byte, then signed32 f80, before assigning f84/f88 masks.
A20 reads fa8 and packed D, sets fa9's low nibble, clears content_fa0, optionally
reads a byte count and raw byte IDs, and finally reads byte fb0. Without A20
the list/scalars are retained; without D80 a present A20 clears the list.
Completed content IDs remain after a later read error.

A40 composes NativeGear's actual nested item readers. Empty resulting Gear
sets float13a8 to positive zero, including lists containing only null slots.
Independent complete Gear::Weight assembly at1414c6fe0 is retained in
`evidence/native-person-gear-weight-gnu.txt`: the float accumulator starts at
zero, traverses entries in order and calls each item's virtual+298 before each
float add. Nonempty Gear currently fails Unsupported **after** wire reception,
before writing weight or proceeding to the domain. No weight is guessed.

Extended fields execute in native order: actual Domain, then the unresolved
JobList boundary, signed32 10d0/signed16 f58, double-overload1408, then
double-overload f60/f68. SinglePrecision narrows these wire values to float
then widens; TypeWidths/DoublePrecision consume and preserve f64. These are
distinct from GlobeLLF/HitNotify's float destinations.

B40 sets state1130 bit200000 without clearing it on omission. B80 replaces
bit8000, clearing it when B80/the extension is absent. All unrelated bits are
retained. C80's unresolved ItemHash comes **after** these state writes, before
the trailing A01 byte10e8 and C40's final83d/83b low/high nibbles.

## Explicit boundaries

Options represent unobserved values, not native constructor defaults.
`with_prior_fields` requires caller-supplied actual prior660/state1130 because
comparison and masked updates cannot be faithful with guessed zeros. Missing
prior660 fails after consuming A04's short, without applying it. Missing
state1130 fails at its read-modify-write stage after preceding completed reads.
The full inherited construction path is still required; this API is not it.

JobList and ItemHash fail Unsupported at their native call positions, leaving
their payload untouched. Nonempty Gear weight and commodity2 replacement are
also explicit Unsupported cases. These are temporary unfinished requirements,
not alternate successful interpretations or ignored packet fields. No opaque
skip or successful no-op callback bypasses them.

Earlier scalar/list/identity mutations survive later errors. The existing
fresh GlobeLLF/item slot safety policies remain explicit; Qt failed-stream,
allocation/copy/destructor and lock parity are not claimed. Rust exclusive
borrowing replaces locking only for this isolated state API. Real QObject,
model/capsule effects, shared state and dispatch ownership are still pending.

## Offline proof

Fixtures compose actual Identity/GlobeLLF/HitNotify/Domain/Gear readers, not
callbacks simulating those decoders. They exercise supported A/B combinations
in all three profiles, exact late-tail/sentinel alignment, every packed short
with equal/unequal prior values, repeated sparse/reset updates, all content
counts and packed D values, every packed scalar/C byte, and every truncated
prefix of a supported all-field suffix. Explicit boundary tests assert payload
position and earlier/later mutations for jobs, inventory and nonempty weight.
These prove the implemented suffix scope, not complete Person packets or
avatar presentation.

`just check` passes402 headless and423 client library tests,9 CLI tests per
profile,1 graphical-entry CLI test, formatting and strict client-feature
all-target Clippy. No public server or native executable was used.
