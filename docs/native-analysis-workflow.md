# Reusing native analysis

Do not use one `rea decompile` process per function for this target. Each CLI
process owns an ephemeral Ghidra import and discards it on close; prior runs
spent about 640–655 seconds importing/auto-analyzing the whole executable per
query. Retained JSON is evidence, not a reusable Ghidra database.

When the connected REA MCP tools are available, prefer that persistent server:
check `binary_session` for an already matching target before `open_binary`, then
batch distinct unresolved procedures with `procedure_pseudo_code`. Keep the
same target/provider open across dependent reconstruction work. Export complete
Evidence bundles to new paths after each batch; inspect retained results instead
of querying a function again just to read its output. A provider-neutral Evidence
snapshot is not an analyzed Ghidra database for unseen procedures. Close the
provider when the investigation actually finishes, not after each function or
intermediate coding milestone. Do not start a separate CLI import alongside a
live matching provider. First-import time and warm-query time must be reported
separately; neither an observation timeout nor a missing progress notification
establishes that an active analyzer has stopped.

If connected tools are unavailable, or their request deadline cannot accommodate
a cold import, use `tests/native_session.py` with an existing REA launcher, the selected binary
and a **new** output directory. It uses the public MCP stdio interface and
keeps one server/target session alive for multiple sequential queries. Set
GHIDRA_INSTALL_DIR and JAVA_HOME from the development environment if needed.

```sh
python tests/native_session.py /path/to/rea reference/openshores-full/OpenShores.exe target/avatar-analysis-session
```

Enter one request per line, then send another when the response is retained:

```json
{"procedure":"0x1412abe30","output":"dhdna-fields.json"}
{"procedure":"_ZN11AuConBountylsER11QDataStream","output":"condition-bounty.json"}
{"close":true}
```

The first function still pays default full auto-analysis; subsequent queries
reuse that session. Query elapsed times are printed to verify reuse. Leave
stdin open between queries (use a PTY when driving interactively). EOF closes
the binary gracefully and deletes the provider's temporary project. Poll the
same running session; never restart because an observation window expired.
Do useful independent implementation/fixture work during initial analysis.

Every complete MCP response, tool catalog, open/close response and server log
is retained. Inspect isError, target digest, analysis profile, warnings and
full pseudocode before using a result. No source executable is executed or
modified. No public server is contacted. Existing evidence/output files are
never overwritten. This driver does not create a persistent on-disk Ghidra
database; a later server restart incurs one new import. REA evidence snapshots
are not a substitute for an analyzed database for unseen function queries.

Legacy tests/native_decompile.py remains for isolated requests only, not the
default reconstruction workflow. Transport tests: `python -m unittest discover
-s tests -p test_native_session.py`.

## Cold-import cancellation observed with connected tools

The connected client's first `procedure_pseudo_code` request for DpBody::SetDNA
returned `timed out awaiting tools/call after 300s` (300.004 seconds). The analyzer
initially remained live, so it was polled rather than restarted. Subsequently
its process and temporary project disappeared and the exported evidence bundle
contained zero records: this attempt did not produce usable pseudocode. The
error and empty bundle are retained in `target/native-model-analysis-3`.

Only after confirming that terminal state was the connected target closed and
one raw-stdio session started in `target/native-model-analysis-4`. The driver
has no per-request five-minute client deadline and sets the provider startup
timeout to thirty minutes for its own child only. This changes no global Codex,
REA or Ghidra configuration. Keep its PTY stdin open and batch unresolved queries
there; do not start another import while it remains live. A timeout alone still
does not authorize a restart: verify the specific process/session first.

## Measured reuse

The recovered cold-import session `target/native-model-analysis-4` completed
DpBody::SetDNA in672.161s, then served new constructor/model/volume queries in
0.074..0.172s without another import. Subsequent MakeParts, part generators,
bone locations and OneTimeCalcs queries also reused that same session. The
complete responses have matching target SHA256 and analysis-profile digest
`8e3a14075ec67e0acc0e527cf433a32154bc1e12800cc3154deb7d62e385ced2`.
Read `.structuredContent.normalized_result` and both raw/provider limitations
from those files; several methods have terminal-tail-call observations, and
decompiled apparent integer/void returns still require instruction-level type
verification. These are static results, not observations of the game running.
Keep this warm session open for the remaining model/animation dependencies.

The first batch in `target/avatar-analysis-session` returned these elapsed
times from one live server: DhDNA_ 665.655s (initial import/analysis), Bounty
0.348s, ImplantDNA 0.078s, ImplantSpecies 0.079s. All four MCP responses reported
isError=false and were retained completely. That process subsequently became
unavailable after interruption; its evidence remains reusable, not its database.
The next unresolved-reader session uses target/avatar-analysis-session-2.
This verifies reuse for new functions, not merely replay of a saved
answer. Returned pseudocode still requires review before implementation.

The second session continued serving new constructor/helper queries in
0.079..0.266s after its initial import, including SitAtConsole and Unit/Sentient
constructors. Its process handle was later authoritatively missing (not merely
an observation timeout); all completed records remain reusable. Three follow-up
requests for OpPose, InitPoses and CurrentGeneralPose were not accepted and
have no saved results. Do not treat them as queued/completed or repeat any of
the successful queries. For those unresolved functions, use independent static
evidence where sufficient, or start ONE new session/import and batch them there.
