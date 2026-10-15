# Reusing native analysis

Do not use one `rea decompile` process per function for this target. Each CLI
process owns an ephemeral Ghidra import and discards it on close; prior runs
spent about 640–655 seconds importing/auto-analyzing the whole executable per
query. Retained JSON is evidence, not a reusable Ghidra database.

Use `tests/native_session.py` with an existing REA launcher, the selected binary
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

## Measured reuse

The first batch in `target/avatar-analysis-session` returned these elapsed
times from one live server: DhDNA_ 665.655s (initial import/analysis), Bounty
0.348s, ImplantDNA 0.078s, ImplantSpecies 0.079s. All four MCP responses reported
isError=false and were retained completely. That process subsequently became
unavailable after interruption; its evidence remains reusable, not its database.
The next unresolved-reader session uses target/avatar-analysis-session-2.
This verifies reuse for new functions, not merely replay of a saved
answer. Returned pseudocode still requires review before implementation.
