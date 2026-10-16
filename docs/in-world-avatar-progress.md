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

DNA-adapter checkpoint verification: 727 default library tests and 759 client-feature library
tests pass (asset/GPU-dependent ignored cases excluded). Client-feature library
Clippy with warnings denied, formatting and diff whitespace checks pass. No
renderer or local-server behavior has changed yet, so no new visual acceptance
is claimed.

## Background preparation and lifecycle checkpoint

The connected client now starts normal-body preparation off the frame thread
once assets and the visible selected player's received DNA are available. The
worker reuses `NativeDpAvatarGeometry`, Stand, the approved sleeves, angle
smoothing and the mounted atlas. The UI explicitly says that world placement is
pending; prepared CPU batches are not yet presented as an in-world model.

`AvatarContentKey` contains normalized DNA, installation-preparation revision,
smoothing angle and seam setting. Position, rotation and camera state are not
keys. Session generation and entity ID belong to `AvatarRequest`, separate from
the geometry cache. Each successful validated bootstrap advances a checked
generation even when its player ID is unchanged; unrelated scene/status updates
do not. A second internal selection revision also invalidates a pending request
removed and reselected before completion.

`AvatarPreparation` bounds work to one in-flight job and one completed cache entry.
Changed requests clear publication of old geometry immediately. Stale completions
are discarded rather than attached or cached; current failures are retained and
not retried every frame. Already completed matching geometry can be deliberately
rebound to new ownership without rebuilding. Mounted assets are shared through
an Arc so a worker does not clone the installation's backing containers, and an
installation change cannot invalidate memory used by an old worker.

Tests cover stale session/entity/DNA/mount jobs, movement-independent reuse,
settings validation/invalidation, worker failure, disconnect/removal and same-ID
reselection. A minimal UI-system test verifies missing assets start no source
worker and NativePreview clears source-avatar status. Both pure lifecycle and
source-bootstrap tests remain independent of the renderer.

`just test-local-avatar-preparation` reads `_default_dna()` from the supplied
server without opening sockets, sends those bytes through a source-profile
bootstrap and selected-player adapter, then prepares mounted geometry. This
passes for the actual test-account default: six native parts and seven posed
render batches, without fixture substitution or account changes. It also tests
same-ID bootstrap generation and overflow failure atomicity. This is CPU receive
and asset-preparation evidence, not a connected-server/GPU acceptance test.

Preparation checkpoint verification: 732 default library tests and 765 client
library tests pass (the pre-existing asset/GPU-dependent ignored cases are
separate). The two source-bootstrap/mounted preparation integration cases pass
through the supplied-server DNA helper. Client-library/integration-test Clippy
with warnings denied, formatting and diff whitespace checks pass. No live server
or connected GPU visual acceptance is claimed for this checkpoint.

## Renderer integration points and next work

`client_ui/scene.rs` currently despawns/recreates markers on every scene revision.
Mounted item meshes have a resource cache, but persons always use a sphere and
StandardMaterial. Reusing that rebuild loop for expensive avatar generation would
violate the goal's movement/lifecycle requirements.

Next: attach prepared batches to a persistent local-player render root, reuse the
body skin material without the offline viewer's display normalization, and
establish scale/frame/ground-anchor behavior before connected GPU acceptance.
No world placement, armor mapping, GPU avatar mesh lifecycle or connected visual
acceptance is claimed by this preparation checkpoint.
