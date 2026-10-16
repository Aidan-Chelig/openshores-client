# In-world local-player avatar: progress ledger

Objective and acceptance criteria: [in-world avatar goal](in-world-avatar-goal.md).
This milestone is active. The connected renderer now attaches prepared local
avatar meshes. Connected front/back captures now verify the default received
body is drawn; ground anchoring and broader connected acceptance remain open.
Earlier checkpoints below describe their own scope.

Latest checkpoint: first-person avatar-interior occlusion is fixed using the
prepared native neutral eye point and local-view head suppression. An ordinary
connected GPU capture now shows an unobstructed terrain horizon. Body coordinates
are unchanged. See [camera evidence and remaining checks](native-avatar-camera.md).
Current client library: 777 passed, six ignored. Warnings-denied client Clippy
passed before the final small lateral-offset correction. Final-binary captures,
looking-down behavior and full offline/local-client regressions remain open.

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

## Connected full-body capture checkpoint

`just test-connected-avatar` uses the same disposable server/login/account path
as quick-dev. It waits for the received body to attach, warms 600 frames after
attachment, captures the real connected window to a new ignored
`target/connected-avatar-*` directory and exits. Output uses create-new semantics;
the installation and supplied reference server source are not modified.
Timeout/failure produces a non-success exit. A replacement render root restarts
warmup. The test rejects shader compilation errors and hierarchy warnings.

Shift+Backspace toggles the minimal full-body inspection camera during normal
connected play. Native mesh bounds frame only the camera; neither the body
transform nor authoritative/predicted position is adjusted. A CPU test checks
that fitting remains camera-only under a scaled, rotated southern body.

Two actual Vulkan captures were opened and inspected locally:

- `target/connected-avatar-twabd4n7/received-avatar.png`: rear view, complete body
  and no duplicate sphere. This run exposed missing visibility components on the
  parent root; those were fixed rather than ignoring the hierarchy warnings.
- `target/connected-avatar-8xzntru0/received-avatar.png`: front view after the
  visibility fix, showing face/eyes/hair, skin material, torso, arms/hands and
  legs/feet. No hierarchy or shader errors were emitted. This uses the account's
  actual default received DNA, not either offline fixture.

Both show an off-ground body over the coarse terrain. The disposable fixture
deliberately spawns above terrain; this is not proof of correct landed placement
or native-unit scale, since the camera fits the body. There is no ground snap or
bounding-box body correction. Captures remain local and ignored, not redistributed.

The client build, 771 client-library tests (six ignored), warnings-denied Clippy,
formatting and diff checks pass. Remaining acceptance includes established body
origin/ground contact, scale against world geometry, connected turning/movement,
a second received appearance and connected lifecycle cases. The milestone stays
active.

## Neutral anchor evidence checkpoint

[Neutral world-placement evidence](native-avatar-world-placement.md) now records
SHA-bound Stand CGOffset/pose-adjustment dispatch and native ground-clearance
call sites. `just test-native-avatar-anchor` verifies these with a focused static
check, without executing reference code or importing Ghidra. The rejected old
snapshot's profile mismatch is recorded; no broad replacement analysis was run.

`NativeDpAvatarGeometry::standing_ground_clearance` exposes the existing native
`CGHeight(true)` result for neutral adult validation. The mounted received-DNA
test measures it separately from `CGHeight(false)` and the drawn sole envelope,
and checks that changing the pose context is rejected. The difference is not
used to move the render body. A controlled received grounded/off-ground test is
still required; nominal ground clearance alone does not prove wire-origin or
landed visual acceptance.

Anchor checkpoint verification: the SHA-bound static check and both mounted
receive/preparation integration cases pass; 732 default-library tests pass
(one ignored). Client-library/integration-test Clippy with warnings denied,
formatting and diff whitespace checks pass. No new GPU acceptance is claimed
for this metadata/evidence-only placement checkpoint.

## Controlled landed/off-ground connected checkpoint

`just test-connected-avatar` now performs two authenticated client launches in
the same disposable world. The first receives a north-pole position at terrain
plus native `CGHeight(true)`; the second receives a database update five native
units higher. The harness measures clearance using Rust and the actual supplied
server DNA, not an independently copied Python formula. Before each capture,
both received and render-root positions must agree with the server fixture.
CPU coverage rejects an offset, forced snap, double scale and nonfinite values.

The inspection camera includes ground context. Its gold tangent cross marks the
actual rendered-terrain radial hit, and a gold vertical ruler has one native
world unit per tick. These are explicitly diagnostic additions; neither body
nor terrain placement changes. Both final captures under ignored
`target/connected-avatar-1ffbaeup/` were opened: `landed.png` shows nominal ground
contact with the known small sole residual, and `off-ground.png` shows the body
remaining about five ticks above that reference. The six-unit body height agrees
with measured native geometry. No shader/hierarchy errors occurred.

Mounted quick-dev/play seed their owned account at the same native nominal
clearance, replacing the arbitrary five-unit spawn. Installation paths are
resolved before the harness changes into its disposable runtime directory.
Remaining gates include a second received appearance, connected motion/turning,
southern placement, lifecycle workflows and normal camera acceptance; this is not
completion of the full milestone.

## Second received appearance and live-update checkpoint

`just test-connected-avatar` now includes a third authenticated launch after a
disposable SQL DNA update. Only size selector byte13 changes (`0x46` to `0x36`);
anatomy and normal skin remain supported. Rust measures each DNA independently.
Before capture the exact received bytes, normalized attached appearance key,
actual mesh Z extent and root scale must agree with the expected values.
Malformed/nonfinite expectations are rejected. Native height changes from
6.0297766 to 5.240967 units, not merely a camera-fit difference. All three cases
completed in the fresh `target/connected-avatar-3inlmrs_/` run. The earlier
`target/connected-avatar-u139s936/smaller-received.png` was opened and inspected:
the complete smaller body and skin are visible against the native-unit ruler.

`just test-connected-avatar-live` tests one continuous authenticated session.
`avatar_live.rs` sends two actual position/rotation reports through the production
runtime/serializer. The first echo must update the entity's received transform
while retaining the same root and mesh IDs. The second sparse received DNA update
must change the appearance key/root, release every old mesh, and leave one local
root with no duplicate marker. The replacement remains attached for 120 frames,
then receives the usual 600-frame capture warmup and exact DNA/height checks.

This workflow explicitly runs `tests/avatar_live_server.py`: it retains the real
supplied login/bootstrap and report receiver, observes their live report cache,
then emits source-profile transform echoes and a second-report DNA change. The
supplied normal writer emits zero rotations and no such DNA update; this fixture
does not claim unmodified-server echo support. Reference files remain unchanged.
An initial fixture dependency-injection failure was fixed by preserving the
original callable signature; the successful rerun completed all gates. Repeated
identical reports handle the source dispatch's first-seen-opcode logging branch;
the fixture counts distinct received values, not attempts.

`target/connected-avatar-wpdj3q7g/live-update.png` was opened and inspected after
the successful live run. It shows the complete updated textured body. Its origin
does not jump when the body becomes smaller, so the new soles remain slightly
above the previous ground plane: deliberately no terrain snap or height rewrite.
The image is local/ignored, not redistributed.

Verification: 773 client-library tests pass (six pre-existing asset/GPU cases
ignored); strict client library/binary/integration Clippy, formatting and diff
checks pass. The three ordinary connected cases and the explicit live-update
fixture pass on Vulkan. Connected southern placement, reconnect/removal/session
workflows, normal first-person camera acceptance and final offline regressions
remain required. The full milestone is not complete.

## Connected removal, restoration and reconnect checkpoint

`just test-connected-avatar-lifecycle` extends the explicit live fixture in the
same client process. After motion and the sparse DNA update, a third distinct
real position report causes the test server to withdraw only the selected player
from its actual generated manifest, then restore the full generated manifest one
second later. Other manifest IDs are preserved; no client-side event is injected.

On received removal, acceptance checks that the old root is absent, every owned
mesh/material/atlas asset is removed, and no child draw references the removed
meshes. On restoration, it requires one new GPU root and the same completed CPU
body Arc, proving deliberate bounded-cache reuse rather than geometry regeneration.

The client then requests a real disconnect. The render state must remain empty
for 30 disconnected frames before reauthenticating with the synthetic account
and selecting the same player. The successful new bootstrap must advance session
generation, match the current attached owner, restore the actual received default
DNA (not the earlier smaller sparse update), and leave exactly one local root.
Credentials retained for this opt-in workflow use zeroizing storage and are
cleared after successful reconnect; normal quick-dev authentication is unchanged.

The Vulkan run completed all lifecycle gates and the final DNA/height capture.
`target/connected-avatar-enxi7zms/live-lifecycle.png` was opened and inspected:
the complete default textured body is present after reconnect, with no duplicate
marker or stale smaller body. Captures remain local and ignored.

CPU cleanup tests also verify that the diagnostic GPU snapshot names the exact
owned handles before cleanup and becomes unavailable afterward. The existing
scheduler race tests remain the evidence for stale asynchronous completion during
session/selection changes; this connected scenario disconnects a ready body and
does not pretend to force a slow-worker race.

Verification: 773 client-library tests pass (six ignored); strict client library
and binary Clippy, formatting and diff checks pass. Connected southern placement,
normal first-person camera acceptance and final offline regressions remain open.
The milestone is active, not complete.

## Connected southern-hemisphere checkpoint

`just test-connected-avatar-south` runs south-pole landed/airborne captures and
the continuous motion/turning/sparse-appearance-update scenario. The disposable
SQL position uses the analytic terrain height at latitude -pi/2, with native
nominal clearance and an explicit five-unit off-ground variant. No new renderer
position, scale or gravity correction is introduced.

Source inspection identifies a relevant supplied-server defect:
`network/scene/connection.py` emits `(0,0,0)` rotation specifically for its selected
player's bootstrap, while using `_peer_upright_euler` for peers. The opt-in test
wrapper replaces only that negative-Z, zero-rotation person base record with
the source's `gravity_align_euler` result before it is sent. This is explicit
controlled-server evidence, not a claim that unmodified selected-player
bootstrap is correct at every latitude. It neither changes reference files nor
silently replaces received rotations in Rust.

Acceptance requires negative-Z position, model +Z pointing outward, and render
rotation matching the received raw transform. CPU checks reject global +Z up,
an additional renderer rotation and a northern point. The live case also checks
the outward bootstrap before any local rotation prediction, then verifies real
report receipt, root/mesh reuse during turning, and sparse DNA replacement.

All three Vulkan runs completed and their captures were opened and inspected:
`target/connected-avatar-lq49c67t/south-landed.png`, `south-off-ground.png`, and
`target/connected-avatar-r249tuf5/south-live-update.png`. The complete textured
bodies are outward-facing relative to the terrain/ruler; the off-ground body
remains five native units above the ground reference. Captures remain ignored
local artifacts. The source's incorrect zero bootstrap remains a documented
server limitation outside this explicit fixture.

Verification: 774 client-library tests pass (six ignored), with warnings-denied
client library/binary Clippy, formatting and diff checks passing. Normal
first-person camera acceptance and final offline/local-client regressions remain
open. The full milestone is not complete.
