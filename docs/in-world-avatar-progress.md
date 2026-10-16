# In-world local-player avatar: progress ledger

Objective and acceptance criteria: [in-world avatar goal](in-world-avatar-goal.md).
This milestone is active. The connected renderer now attaches prepared local
avatar meshes, but connected GPU output, ground anchoring and globe-relative
facing remain unverified. Earlier checkpoints below describe their own scope.

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

`client_ui/scene.rs` despawns/recreates diagnostic markers on every scene revision.
Mounted item meshes have a resource cache. Reusing that rebuild loop for expensive avatar generation would
violate the goal's movement/lifecycle requirements.

## Persistent renderer integration checkpoint

`client_ui/avatar_rendering.rs` attaches prepared batches beneath a separate
local-player root, outside the diagnostic-marker rebuild loop. Movement updates
the root transform in place; request changes replace it. Only the successfully
attached player's marker is removed. Other entity diagnostics remain unchanged.
Missing/invalid globe-child transforms retain diagnostics and show a placement
status instead of inventing an ancestor transform.

The shared native skin plugin/material is registered in the client. Mesh expansion
and PNG decoding run in the appearance worker. Its bounded completed cache keeps
the render meshes, decoded image and skin offsets, not an additional copy of the
native body and posed corners. GPU assets are uploaded once per attached request;
replacement/disconnect/removal explicitly releases mesh, material and image assets
and despawns the body hierarchy.

The world mesh adapter retains native units and origin (including homogeneous W,
UVs and posed normals), without the offline preview's centering or size fitting.
The root applies the existing source-profile rotation and display scale once,
using predicted local position when available. It deliberately does not add a
body-height offset or snap to terrain. This is provisional placement, not proof
of the correct body origin, globe-facing convention or ground contact; the HUD
says that verification is pending.

CPU tests check raw coordinates/W, persistent root and mesh identity across a
position change, removal cleanup, child/resource cleanup and preservation of
unrelated markers. These are render-world/ECS tests, not GPU visual acceptance.

Next: establish body origin and globe-local facing, implement/report turning,
provide full-body connected inspection and verify real owned-server GPU output.
Second received appearances and the remaining connected acceptance cases are
still required. No armor mapping or completed in-world milestone is claimed.

Renderer checkpoint verification: 768 client-library tests pass, with six
pre-existing asset/GPU-dependent cases ignored. Both supplied-server-DNA
preparation integration cases pass. Client-library Clippy with warnings denied,
formatting and diff whitespace checks pass. No connected GPU capture was taken
for this checkpoint.

## Source-client turning checkpoint

Movement, camera heading and predicted body facing now share one tangent-frame
helper: local X right, Y forward, Z radial up. Clockwise UI yaw is converted into
the existing Rx*Ry*Rz Euler representation, rather than incorrectly treating it
as a global Z angle. Tests round-trip the encoded angles through the actual
placement matrix at both poles, the equator and an off-axis southern position.

Bootstrap initializes camera heading from the received rotation's projected
forward direction. The body retains received rotation until local input predicts
a new facing. Moving or turning updates that predicted rotation immediately and
sends it with the next position report. A dirty flag preserves short mouse turns
that finish before the report interval, instead of losing their final report.
Pitch remains camera-only. Bootstrap/disconnect reset prediction/report state.

The render-world regression now verifies southern-facing rotation changes while
retaining the same root and mesh handles. A control-system test checks immediate
south-pole body rotation and pending short-turn retention. All 770 client-library
tests pass (six asset/GPU cases remain ignored); warnings-denied client-library
Clippy passes.

This establishes the Rust source-client prediction convention, not original
runtime parity or proof that the supplied server persists/echoes orientation.
An initially received zero rotation is not silently replaced with inferred
gravity alignment. Ground origin/contact and connected GPU acceptance remain
open; the provisional-placement HUD status is deliberately retained.

The graphical client builds and `python tests/local_server.py quick-smoke` passes
against its disposable PostgreSQL/server world on loopback, using Vulkan on the
local AMD adapter. It mounts 4882 assets and reaches selected-player preparation,
but its 30-frame exit happens before body readiness. This is login/mount/GPU
startup regression evidence only, not body draw or visual acceptance. Source
inspection of `protocol/atoms/person.py::_build_daperson_xform_update` confirms
that writer emits three zero rotation floats; server orientation echo must be
tested explicitly rather than assumed from client prediction tests. Formatting
and diff whitespace checks also pass.
