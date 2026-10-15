# Native avatar integration goal

Implement the patched native Person path, not a source-protocol substitute.
Public servers remain out of scope; use offline evidence/fixtures and owned
loopback reference-server tests where the protocols actually overlap.

Required milestones and proof:

1. Reuse analysis: `native_session.py` and workflow documentation exist;
   transport tests pass and the selected target opens. **Pending:** measure
   first versus subsequent function queries in the same session. Do not claim
   a verified speedup from merely writing the driver.
2. Reconstruct nested readers: complete retained evidence for DNA, conditions,
   Creature pose/console bodies, Sentient skills/orders and Person helpers;
   corroborate scalar overloads and virtual bindings. Eight condition bodies
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
