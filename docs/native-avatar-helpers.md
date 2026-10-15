# Native avatar helper ledger

Target SHA-256 and static Ghidra profile match native-person.md. Complete MCP
records below retain provider limitations; no executable was run. These layouts
are not yet implementations except the SkillSet described in native-sentient.md.

- `evidence/native-hit-notify-reader.json`,
  `ev_24efd4c58f6d9197225f13412bb5f10221b9ac355d1e32584468d2e539f2a20b`:
  three float-overload values widened to doubles at 00/08/10, byte→u32 at18;
  when nonzero, i32 at28 then i32 at20; byte→u32 at1c; when nonzero, i32 at2c
  then i32 at24. Omitted pairs retain existing fields; they are not reset by
  this reader. Float stream profile still must come from enclosing context.
- `evidence/native-globe-llf-reader.json`,
  `ev_09abc1f5bbd78e46c8e6331d889638c0e4dba99ecebb484bce1b7d5e00c49f2b`:
  float at04 then float at00. Preserve the imported tail-jump warning; do not
  reverse wire order to match destination order or infer latitude semantics.
- `evidence/native-animalia-hitpoints.json`,
  `ev_36890ccdb4fd29f14a21ded142ed724f2fbf5fe636bfed0b5516b4443763f0c1`:
  signed short argument clamps at minimum -30 then stores at30. No upper clamp
  or additional body is present in this helper. This resolves the remaining
  Animalia SetHitPoints branch in the Creature ledger, not its whole update.

Other warm-session records (Domain, JobList, Gear, GearSlot) remain under
target/avatar-analysis-session-2 for full review before use. They do not prove
their nested dependencies or client integration merely by returning success.
