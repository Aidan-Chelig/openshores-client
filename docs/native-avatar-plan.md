# Native avatar integration goal

Implement the patched native Person path, not a source-protocol substitute.
Public servers remain out of scope; use offline evidence/fixtures and owned
loopback reference-server tests where the protocols actually overlap.

Required milestones and proof:

1. Reuse analysis: `native_session.py` and workflow documentation exist;
   transport tests pass and the selected target opens. Same-session measurements
   now verify reuse: first query 665.655s; next three 0.348s/0.078s/0.079s.
2. Reconstruct nested readers: complete retained evidence for DNA, conditions,
   Creature pose/console bodies, Sentient skills/orders and Person helpers;
   corroborate scalar overloads and virtual bindings. All twelve condition bodies
   are implemented; the remaining dependencies are not complete.
3. Implement ordered Unit → Creature → Sentient → Person receive state in
   manageable module folders. Preserve sparse/reset distinctions and native
   ordering; unsupported cases must fail explicitly, not skip opaque bodies.
   Document Rust malformed-stream safety separately from Qt behavior.
4. Concrete Person construction, retained-object reuse, base reception,
   owner lifecycle and body dispatch integrated with existing native dispatch.
   Tests must exercise complete fresh and existing Person packets, not only
   helper codecs or mock callbacks. Constructor defaults require evidence.
5. Bridge native avatar state into minimal client presentation with an explicit
   protocol/profile boundary. Offline native packet fixtures must actually
   reach avatar presentation; source-server GUI success alone is insufficient.
6. Verification: truncation/unsupported cases and sparse repeated packets;
   full Rust/client checks; local reference gameplay/GUI tests where applicable.
   Record protocol mismatch limitations rather than call source packets native.

Current evidence ledger: native-unit.md, native-creature.md, native-sentient.md,
native-person.md, native-conditions.md and their complete docs/evidence records.
The legacy DhDNA wrapper result in target/native-dhdna-reader.json has been
reviewed: it delegates to DhDNA_ at `0x1412abe30`; it does not establish the
nested DNA layout. Preserve its terminal-flow limitation when retaining it.

Active batch started in target/avatar-analysis-session: DhDNA_ fields, Bounty,
ImplantDNA and ImplantSpecies. Keep that server alive for subsequent queries;
do not start a separate single-function CLI process for each dependency.

Verification after Unit scalar implementation (`fb813c5`): `just check` passed
format checking, 285 headless library tests, 306 client-feature library tests,
9 CLI tests in each configuration, 1 graphical-entry CLI test, and strict
client-feature all-target Clippy. This verifies the implemented components,
not the still-missing full Person dispatch/presentation pipeline. The first
shared analysis batch subsequently completed with the warm-query times above.
The former process handle no longer exists (verified after interruption).
Complete returned records remain saved; do not repeat those queries. A new
long-lived session is needed only for unresolved readers, batching them after
the single initial import rather than restarting per function.

Unit now has concrete condition-list composition (`NativeUnitState`), including
destructive replacement before count, unknown factory continuation, explicit
truncated-body rejection, and partial-list retention on later failures. This remains
a receive stage, not complete Person dispatch or avatar presentation.

DNA milestone: all 28 chromosome bodies, four gene bodies, both counted
chromatids and dynamic DhDNA/AuDNA dispatch are implemented for the verified
Qt7 scene profile. Static GNU records corroborate gene dispatch/read layouts
and 16-bit count behavior; existing complete REA records cover the chromosome
readers. The shared follow-up analysis process is live with GeneChain/factory
and three distinct gene readers queued. Headless verification now passes
302 library tests, 9 CLI tests and strict all-target Clippy. This does not yet
verify Creature/Sentient/Person or client presentation. Pregnant composition
subsequently passed all-flag/truncation fixtures and a real mixed-condition
Unit update carrying dynamic AuDNA, a trailing condition and the final vector.

Sentient suffix milestone: actual skill-map decoding, ordered scalar resets,
single-order reuse versus initialization and conditional string reception now
exist in native_sentient. Real matrix/clock context is required for initialization;
the suffix API does not skip or claim to implement Creature. Full `just check`
passes formatting, 310 headless library tests, 331 client-feature library tests,
9 CLI tests in each profile, 1 graphical-entry CLI test and strict client-feature
all-target Clippy. No public server or native executable was used for testing.

Creature dependency milestone: exact ten-byte-plus-short pose wire decoding,
Game and Shield station bodies are implemented. All 11 distinct station reader
records are retained/reviewed; 20 symbol-relative virtual bindings are mapped
but constructor corroboration remains pending. Matching-console fallback was
resolved in GNU assembly. Constructor and pose/sensor/route helper batches are
saved in the existing warm session for review, not reimport. Headless checks
pass 313 library tests, 9 CLI tests and strict all-target Clippy. A full Creature
receiver, its state/application effects and inherited Person dispatch remain
unfinished; these helper successes do not establish avatar presentation.

Console binding follow-up: all 20 mapped primary vptrs are now confirmed by
the corresponding constructor assignments; complete records for those and the
base/empty classes are retained with an index. Engineer/Helm bodies now decode
their packed state and ordered scalar fields, with all flag/packed-byte pairs
tested. Headless verification passes 315 library tests, 9 CLI tests and strict
all-target Clippy. Pose application and remaining console nested formats are
still being reconstructed; this does not yet close Creature reception.

Collection-station milestone: Surgery now composes the real conditions with
its verified Qt7 QPoint list; Nav composes three float vectors and the real
double-overload waypoint route; Power performs sparse ordered in-place updates.
All seven currently implemented station families are documented in
native-station-collections.md/native-consoles.md. `just check` passes formatting,
322 headless and 343 client-feature library tests, 9 CLI tests per profile,
1 graphical-entry CLI test and strict client-feature all-target Clippy.
The remaining Sensor/Weapon/Transporter formats require full and minimal blip
reception: factory/list behavior and 43 vptr bindings are retained, and all 41
distinct reader results are saved in the warm session for review. Native
Creature/Person construction, complete dispatch and presentation remain open.

Minimal sensor milestone: all 43 factory types dispatch to their four verified
minimal readers; the counted replacement layer clears before count, retains
completed entries on later errors, and explicitly reports/stops at null factory
types without opaque skipping. Full and minimal blips have separate APIs.
`just check` passes formatting, 327 headless and 348 client-feature library
tests, 9 CLI tests per profile, 1 graphical-entry CLI test and strict
client-feature all-target Clippy. Full +158 readers and remaining station
application still need completion before concrete Creature/Person reception.

Full sensor milestone: all 43 factory types now have typed fresh +158 body
decoders, distinct from minimal reception. The 37 distinct full-reader records
and spacecraft/AuPoint helpers are retained with an Evidence-ID index; exact
GNU symbol disassembly corroborates imported overloads and tail jumps without
rerunning Ghidra analysis. Fixtures cover all types/profiles, every truncated
body prefix, packed-byte combinations, mixed-body alignment and Rust malformed
input policies. `just check` passes formatting, 335 headless and 356 client
library tests, 9 CLI tests per profile, 1 graphical-entry CLI test and strict
client-feature all-target Clippy. These are nested wire bodies, not an assertion
of complete console application or Person presentation. Next compose the
remaining Sensor/Weapon/Transporter readers, preserving the boundary between
wire state and capsule/environment augmentation; then finish ordered Creature,
Person construction/dispatch and the actual presentation bridge.

Sensor-station wire milestone: Sensor, Weapon and Transporter now compose the
real full/minimal blip bodies with ordered scalar/packed fields. Transporter
omitted-vector retention and selected-blip destruction timing are preserved;
null full factories continue, while minimal-list unknown stops propagate in
must-use results. Complete GNU function records corroborate existing saved
REA evidence, without any new Ghidra import. `just check` passes formatting,
346 headless and 367 client library tests, 9 CLI tests per profile,
1 graphical-entry CLI test and strict client-feature all-target Clippy.
These APIs explicitly stop at the wire/local-augmentation boundary documented
in native-station-sensors.md. Capsule/environment augmentation, native station
construction/application, complete Creature/Person dispatch and presentation
remain outstanding; no source-server success is substituted for native proof.

Ordered Creature milestone: NativeCreatureReceiveState now composes real Unit,
DNA, pose and supported AuItem readers with required explicit application hooks
in native order. Model selection occurs every update; growth reset, item
replacement, body clock and unsigned tail semantics have offline coverage.
All 65536 flags, repeated body/sparse updates, every truncated prefix and all
signed-short HP values pass fixtures. Trace contexts verify call order only,
not runtime effects. Default body/Creature/Person constructor records and saved
pose/SetDNA/movement helpers are retained with an index; five new warm queries
returned in 0.079..0.104s without reanalysis. Concrete body/console application,
inherited native construction, Person suffix/dispatch and presentation remain
open. This milestone must not be counted as end-to-end avatar completion.
`just check` passes formatting, 355 headless and 376 client library tests,
9 CLI tests per profile, 1 graphical-entry CLI test and strict client-feature
all-target Clippy.

Pose data/application milestone: concrete DpBody/Animalia word comparisons,
Animalia final-pose normalization, required animation callback conditions and
fallback timer/marker updates now exist. Station-pose requests use the exact
body-dependent word, native occupant bytes and independently corroborated
jump-table type/variant mappings. `just check` passes363 headless/384 client
library tests, CLI tests, formatting and strict Clippy. SitAtConsole and inherited
Unit/Sentient constructors are retained/reviewed. The analysis process handle
is now missing; successful records remain reusable and three unresolved follow-up
requests were not accepted. Next reconstruct OpPose/InitPoses/CurrentGeneralPose
and concrete console/body effects, then complete Person reception/dispatch and
presentation. Required animation callbacks and typed sit requests are not
substitutes for these outstanding runtime effects.

Sitting calculations milestone: complete OpPose main/occupant tables and its
shared-u32 random branch, Animalia InitPoses, CurrentGeneralPose body precedence,
body-specific sitting argument construction and console marker rules are now
implemented. Independent complete GNU records/table bytes resolve these unseen
helpers without reopening Ghidra. Missing required shared random state fails
explicitly instead of inventing a seed. Tests cover raw bounds, signed indices,
table mappings, random overflow/sequencing and sitting-vs-fallback markers.
Actual console lifecycle/capsule notification/animation, the concrete Creature
context and complete Person dispatch/presentation remain open.

Typed console milestone: explicit constructor-established receive subsets now
initialize every station family, and NativeConsoleState composes all bound
wire readers for23 class/type cases including Empty's verified base no-op.
Primary+e0/+f0 bindings are retained and implemented, including Game's dynamic
type, Empty's nonzero predicate and literal-zero station getters. Power's sparse
groups start at17, not guessed zero. Full checks pass375 headless/396 client
library tests, CLI tests, formatting and strict Clippy. Enclosing matching/
sitting lifecycle, settings and capsule augmentation remain open before the
concrete Creature context and end-to-end Person/avatar gates can be met.
Verification: `just check` passes 368 headless and 389 client library tests,
9 CLI tests per profile, 1 graphical-entry CLI test, formatting and strict
client-feature all-target Clippy.

Sitting lifecycle milestone: NativeSittingState now composes typed console
creation, general-pose-only reuse, destruction/clearing, parent notification
ordering, actual body pose state and console marker updates. Offline trace
fixtures cover every byte station type, both body families, failure boundaries,
repeat updates and body-dependent received-word selection. Required external
callbacks are not final runtime effects; the concrete Creature context,
remaining Person reception, packet dispatch and avatar presentation are still
required. Retained REA evidence and exact GNU output were reused without a new
analysis session/import.

Person dependencies milestone: actual Domain, GlobeLLF and HitNotify readers now
exist with complete retained REA records and independent GNU corroboration.
Complete Person update assembly resolves formerly ambiguous scalar reads as
bytes (including the content-ID count/entries and late identity/nibble bytes),
and confirms double overloads for the three extended floating fields. These
are nested codecs, not a full Person suffix, inherited construction/dispatch
or presentation. Gear fixup/weight, job polymorphism and item hash still need
reconstruction; the concrete Creature context remains required as well.

Person gear wire milestone: real GearSlot/Gear composition now uses all seven
established item codecs, preserves ordered duplicate slots, clears before count,
and diagnoses null entries while continuing. Commodity2 replacement and unknown
item types fail explicitly; native weight recomputation remains open. Retained
REA records and exact GNU evidence establish FixItem's condition and distinguish
inventory's separate obsolete filter/count/insertion path. Full Person suffix,
inherited construction/context, dispatch and presentation remain required.
