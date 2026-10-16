# In-world local-player avatar: progress ledger

Objective and acceptance criteria: [in-world avatar goal](in-world-avatar-goal.md).
This milestone is active. The connected renderer still draws primitive person
markers; the following work is preparation, not completed in-game rendering.

## Received appearance boundary

- `client_runtime/worker.rs` receives the owned session's scene updates;
  `ClientModel::apply` records `bootstrap.avatar_id` as the selected player and
  merges later updates into `world.entities`. Person DNA already survives in
  `PersonState::dna`, including omitted, Qt-null and byte-array distinctions.
- The supplied server's `network/scene/connection.py` bootstrap `_pb2` writes
  `person_pose` immediately before its 24 DNA bytes. Thus the Rust field named
  `dna_tag` cannot be treated as a native anatomy discriminator or required to
  equal Stand0x24. This is a source-code observation, not a claim about native
  packet layout. No native runtime or public server was used.
- `ClientModel::local_player_dna()` now reads only the selected source-profile
  player while Playing. Pending state returns None; explicit null, wrong length
  or a non-person identity returns an error. It never selects another available
  person or substitutes the offline fixture. NativePreview and disconnected
  phases do not expose stale source-player DNA through this adapter.
- `LocalPlayerDna` retains the exact received bytes and a separately normalized
  `NativeDhDna`. `NativeDhDna::from_received_bytes` shares the existing receive
  normalization (clear byte5's high bit and byte23), with atomic length validation.
  The entity identity is ownership information, not a geometry-cache key.
- Tests cover omitted/null/empty/short/long DNA, exact byte preservation,
  normalization, opaque pose-byte changes, selected-entity changes, wrong entity
  kinds and inactive/native protocol phases. Armor/helmet owner-field mapping is
  not supplied by this DNA-only adapter; it must not be inferred from unrelated
  source gear flags.

Checkpoint verification: 727 default library tests and 759 client-feature library
tests pass (asset/GPU-dependent ignored cases excluded). Client-feature library
Clippy with warnings denied, formatting and diff whitespace checks pass. No
renderer or local-server behavior has changed yet, so no new visual acceptance
is claimed.

## Renderer integration points and next work

`client_ui/scene.rs` currently despawns/recreates markers on every scene revision.
Mounted item meshes have a resource cache, but persons always use a sphere and
StandardMaterial. Reusing that rebuild loop for expensive avatar generation would
violate the goal's movement/lifecycle requirements.

Next: connect mounted-asset readiness to asynchronous appearance preparation,
keyed by normalized received appearance and asset/render settings. Keep session /
entity ownership separate so stale work cannot attach to a reselected player.
Then attach a persistent local-player render root, reuse the body skin material,
and establish scale/frame/ground-anchor behavior before connected GPU acceptance.
No world placement, armor mapping, avatar mesh lifecycle or connected visual
acceptance is claimed by the DNA adapter checkpoint.
