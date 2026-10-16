# Goal: animate the in-game avatar

Status: active. User objective: add animations to the in-game avatar, retaining
the OpenShores experience and user-mounted assets. A moving connected avatar,
not merely an offline animated fixture, is the required final result.

## Acceptance

- Retain the actual received appearance, native units, correct globe orientation,
  body origin and approved seam repairs. Preserve the recent elevated-camera
  controls and transitions.
- Connect native pose/rhythm/joint behavior to the persistent connected body.
  Idle, walking and running must respond to actual player motion, stop cleanly,
  and change pose without recreating the avatar/atlas every frame. Audit received
  pose/action fields and implement their animation path rather than silently
  replacing server poses with a generic walk cycle. Record any source-profile
  mismatch explicitly; an opaque source pose byte is not a native pose enum.
- Keep model-space smoothing, raw native influences, UVs/material controls and
  pose-independent mesh topology stable. Animated skinning must not accumulate
  transformations or recompute smoothing/asset decoding per sample.
- Follow the animated eye/head coherently; first-person must not become blocked
  by the skull. Elevated/full-body modes must expose the moving avatar.
- Appearance replacement, removal, disconnect/reconnect and stale worker results
  must still clean up correctly. No animation work may apply to an obsolete owner.
- Verify pose/rhythm correctness and repeated sampling with CPU tests. Capture
  multiple actual connected animation phases and inspect them; demonstrate motion,
  running and stopping with persistent render resources against the owned local
  server. Existing body previews/client regressions, formatting and lint must pass.
- Document native observations, deliberate modern choices, supported inputs and
  limitations separately. Do not claim complete animation support based on one
  moving joint or a fixture-only demonstration.

## Initial evidence and work sequence

The existing renderer bakes neutral Stand. Native joint rotations and portions of
torso/head/tail posing plus AuPose/DhDNA rhythm already exist. Retained REA
`native-dp-animate-pose.json`, evidence
`ev_b2f6d8549fe666f3b3d70800cc0d20441ff51abfd74f026b6cf5f752a91ae03e`,
establishes the stage order: CGHeight, selected bone reset, legs, tail, torso,
heads, arms and conditional wings. `native-dp-position-legs.json` maps poses
0x11/0x16/0x26/0x4b to WalkLegs; `native-dp-position-arms.json` retains arm
dispatch. These are static observations, not executed original-runtime evidence.
The concrete locomotion consumers and pose selection still require tracing.

1. Cache immutable smoothed model-space corners and raw influences; allow repeated
   bone sampling without regenerating normals or losing attributes.
2. Retain repaired render batches and native body/pose state through preparation;
   implement evidence-backed pose consumers and deterministic animation time.
3. Connect actual movement/received pose state to those consumers and update
   persistent render data/bones, including eye location and lifecycle ownership.
4. Run connected multi-phase acceptance and regressions, document and commit.

First dependency implemented: `NativeDpPreparedAvatarCorners` prepares normals
once and skins repeatedly with caller-supplied bones. Existing skinning uses that
same path. CPU coverage compares the original result, a second skeleton, unchanged
UV/source identity and repeat sampling without accumulation. This is not yet an
animated in-game avatar; the goal remains active.

Second checkpoint: `NativeDpPreparedAvatarSurface` retains fixed material and
source identity alongside cached corners. Native, distal-repaired and
proximal-repaired paths all prepare before sampling. The connected
`PreparedNormalAvatar` now retains these animation surfaces alongside the initial
Stand result. Repair contact decisions are made once in that initial pose, not
repeated mid-stride. The UI worker still consumes only the initial result; live
animation state/render updates remain to be connected.

Verification: 778 client-library tests pass (six asset/GPU cases ignored), both
mounted received-DNA tests pass, and all 11 mounted assembly/contact/topology/
standing tests pass for the existing anatomy fixtures. Received-DNA coverage
skins the cached repaired batches with the initial skeleton, a second translated
skeleton, then the initial skeleton again; positions change correctly while
normals, UVs, controls, materials, source identity and repeatability are preserved.

Third checkpoint: native Walk/Run leg consumers are implemented and verified
against bounded static disassembly; see [locomotion evidence](native-avatar-locomotion.md).
The new mounted test covers all one/two/three-pair selectors at repeated time
samples and passes. Full-body dispatch/arm gait, time/speed selection and the
connected animation update are still outstanding; no moving-avatar success is
claimed from this isolated stage.

Fourth checkpoint: arm Walk/Run, including native side phases, hand-walking
elbow branches and first-arm pitch override, is implemented. Normal uniform
full-body Walk/Run now composes all supported stages in native order and resets
bones before every sample. The actual received-DNA mounted test passes repeated
full-body phases, retained-batch skinning and exact return to Stand. 782 client
tests, arm static anchors and strict client Clippy pass. See the locomotion
ledger for exact limits. Live pose/movement selection, persistent mesh updates,
animated eye and connected animation acceptance remain required.

Fifth checkpoint: the client now retains an owned `PreparedAvatarAnimation`
behind the bounded prepared-body cache. Hand/foot measurements are warmed on the
preparation worker, then switched to cached-only mode with NO mount reference;
an unexpected new measurement fails explicitly rather than reading assets from
the frame loop. `NativeDpAvatarEye` retains authored metadata and samples current
head bones without reparsing source meshes.

`avatar_animation::animate` updates position/normal/homogeneous-control attributes
on the existing mesh handles using fixed per-partition corner bindings. It does
not replace the root, atlas, material, UVs or mesh IDs. Animated children bypass
static frustum bounds so a bent limb cannot be culled using Stand's bounds.
The local movement predictor publishes tangential movement speed; the initial
runtime path selects Walk/Run from the existing prototype walk/run speeds and
restores Stand on release. Pure vertical motion does not falsely walk. The timer
is monotonic per render owner; it resets on replacement/removal/reconnect.
Errors are sticky for that owner and reported separately in the HUD.

This is explicitly local prediction, NOT recovered native locomotion physics,
time synchronization or source-wire action-pose mapping. The source's opaque
person_pose byte is still not interpreted as native AuPose. That receive/action
audit, connected moving-phase captures, lifecycle regression and final acceptance
remain open. Initial CPU integration passes 782 tests; the mounted received-DNA
case additionally samples the mount-free runtime for both gaits, repeated phases,
finite animated eyes and exact stopped geometry/eye. Strict client Clippy passes.

Connected initial integration check: the rebuilt client completed the owned
elevated-camera capture with the new animation resource active in Stand.
Received DNA/origin/height gates passed and
`target/connected-avatar-nazciyuf/elevated-look-down.png` was opened: the textured
head/shoulders remain visible from above. This verifies launch/idle/camera
integration only, NOT moving GPU phases or animation acceptance.

Sixth checkpoint: `test-connected-avatar-animation` now drives normal W and
Shift+W input in one owned connected client. It requires two changed mesh samples
per gait, player motion, unchanged root/mesh/material/atlas/UV identities, and
exact return to the initial Stand geometry and authored eye after release.
The harness separately checks that actual position reports reached the supplied
server's SQL person coordinates. No client event or pose is substituted by the
test; only keyboard buttons are scripted.

The first full run passed all gates. All five images in
`target/connected-avatar-5l813vag` were opened: walking has modest opposed steps,
running has larger knee/arm motion, and stopping restores the complete body.
The front view also shows increasing overlap with terrain after moving; the
checkpoint is not treated as proof of ground-following/collision correctness.
A side-view rerun logs actual root clearance over rendered terrain to separate
that issue from pose sampling. No corrective body offset or snap was introduced.

Receive-path audit so far: the opaque `dna_tag` is NOT the animation input.
`src/person.rs` already receives separate ten-byte `poses` plus a u32 parameter
under creature flag 8. The supplied bootstrap repeats `_SAVE.person_pose` into
all ten slots, while its report receiver tracks the first reported pose for
bio-state ticking. `network/npc_state.py` uses the source NPC Dying/Dead codes
0x12/0x15. These channels need an explicit source-profile adapter and supported
action dispatch; this current runtime still only selects predicted locomotion.
Do not reinterpret the prefix byte or silently label the entire action catalog
implemented based on these walk/run captures.

Only the owned disposable loopback server is permitted. Never execute original
EXE/DLLs, contact public game servers, modify reference sources/installations or
redistribute game assets/captures. Reuse retained static evidence; investigate
concrete gaps with bounded analysis rather than repeating broad imports.

The side-view rerun also passed every gate; its four gait images in
`target/connected-avatar-6is7cxgr` were opened and inspected. They show intact
body/skin assembly, a modest walking stride and a larger bent-arm running stride.
Measured root clearance over the actual rendered mesh decreases from
2.958218 native units initially to 2.688207 after walking and 1.859523 after
running, against a nominal native Stand clearance of 2.957287. That independently
explains increasing terrain overlap as the prototype movement crosses rising
terrain; it is not evidence that animation should shift the body upward.
Ground-following remains an explicit limitation, not a hidden render correction.
The animation goal remains active pending receive/action dispatch, moving
first-person camera and lifecycle/final regression acceptance.

These implementation/connected-animation checkpoints are committed together
with the preceding requested camera controls. No captures or game assets are
included in the commit.

