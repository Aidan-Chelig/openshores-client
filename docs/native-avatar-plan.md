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
