# Goal: animate the in-game avatar

Status: active. User objective: add animations to the in-game avatar, retaining
the OpenShores experience and user-mounted assets. A moving connected avatar,
not merely an offline animated fixture, is the required final result.

Current supported normal-body inputs: Stand/Walk/Run, Dying/Dead skeletal posing
(0x15 whole-body render orientation remains incorrect; see the latest checkpoint),
received crouch/crouch-walk and Jump posing, arm-channel Wave/Point and
folded-limb codes0x18/0x19/0x1b/0x35.
Mixed admitted channels are preserved;
local movement, held crouch and optional arm gestures predict/report only over
received all-Stand.
Finite source head tilt now reaches normal-body rendering with connected
third-/first-person acceptance. Other actions, nonzero source pose parameters, guided
mode, winged and legless body animation remain explicit unsupported inputs,
not silent fallbacks.
Jump/death posing does not implement displacement, gravity, landing or corpse
settling. Held Caps Lock now predicts crouch over grounded adult source Stand;
native collision/growth physics and the crouched sole discrepancy remain open.
The goal remains active; checkpoints below record evidence, not a claim
of the complete OpenShores animation catalog.

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

Seventh checkpoint: an explicit source-profile adapter now reads the selected
person's separate ten-byte pose array, never its DNA prefix tag. It admits only
Stand (0x24), Run (0x25) and Walk (0x26), preserves mixed channels, and predicts
local movement only over an all-Stand request. Sparse updates retain the last
received channels. Unsupported actions, nonzero pose parameters and head tilt
produce a visible status rather than a silent Stand fallback; a later supported
update can recover without replacing the render owner.

The compositor dispatches each supported leg/arm channel independently and keeps
the existing native head/tail/torso stages. A received moving pose without local
velocity uses nominal animation rate 1: this is an explicit rendering policy,
not inferred server motion or native time synchronization. CPU receive tests
exercise actual packet ingestion, omitted fields, unsupported input and recovery.
Mounted received-DNA tests verify repeatable mixed-channel sampling and exact
all-Stand geometry/eye restoration through the new channel compositor. Both
mounted tests pass; 784 client-library tests and strict client Clippy pass.
Connected receive-driven captures, moving first-person and lifecycle regression
remain required. Broad native action support is not claimed.

The rebuilt connected keyboard-animation regression also passes with this
adapter enabled. Captures are under `target/connected-avatar-_g50_2zu`; opened
Walk-A and Run-B confirm intact textured assembly and the larger running stride.
Persistent-resource, changed-phase, exact stopped geometry/eye and server SQL
movement-report gates all pass. Terrain overlap remains visible and unchanged;
this run does not prove received mixed-pose GPU dispatch or ground following.

Receive-driven acceptance is now available as
`just test-connected-avatar-received-poses`. Its owned server wrapper sends
explicit sparse source-profile packets through the real scene writer; the
reference source is untouched. A disposable control file coordinates requests
only, never substitutes client model state. The client waits for actual received
mixed channels, captures two changed mesh phases, observes an unsupported-action
status, and requires exact Stand/eye recovery. Root position must remain fixed
and root/mesh/material/atlas/UV identities must remain unchanged throughout.
The first connected run passes all those gates. Both received phase images in
`target/connected-avatar-vjf_h_cu` were opened: one arm uses the larger Run bend
while the other stays neutral, and walking leg phases change without moving the
root. The final capture also passes received-appearance checks. Unsupported
0x12 is explicitly rejected and a subsequent server Stand update recovers.
This proves the admitted subset only, not the unsupported action catalog.

Moving first-person acceptance now runs with
`just test-connected-avatar-animation-first-person`. The normal keyboard
workflow remains active, but inspection is disabled and each warmed frame checks
the camera against the current animated authored eye (including the existing
terrain-clearance policy) and requires every local head batch to stay hidden.
The connected run passes all motion/phase/resource/stop/SQL gates; captures are
under `target/connected-avatar-re1e5w6k`. Run-B was opened and shows unobstructed
terrain, not skull interior. This downward-looking capture is not proof of
visible first-person limbs or terrain-following correctness.

Lifecycle acceptance additionally checks that animation ownership is absent
after removal/disconnect and exactly matches the restored/reconnected session,
player and root, with a supported standing status. This strengthened connected
rerun passes, including sparse appearance replacement, manifest removal/restore,
30 disconnected frames and authenticated same-ID reconnect. Its final capture
is `target/connected-avatar-9z4n0xft/live-lifecycle.png`. The current client-library
suite passes 784 tests (six ignored), and strict client Clippy passes. Broader
action dispatch and final preview/regression acceptance remain open.

Next action checkpoint: retained dispatch and bounded SHA-verified disassembly
now support isolated native face-down dead leg/arm stages. These are ported with
literal constants, native pair/side traversal and conditional ankle/DNA behavior;
see the locomotion ledger. Static anchors and two CPU tests pass. The runtime
still rejects source Dying/Dead until full height/torso/head composition, mounted
coverage and connected acceptance are verified. This is progress toward action
support, not a claim that dead animation is finished.

Full face-down action checkpoint: the compositor now admits native 0x12/0x15,
reusing existing height/tail/torso/head dispatch and the new leg/arm stages.
Preparation warms the dependencies off-thread before freezing the caches.
Mounted received-DNA tests pass finite, time-independent full-body samples and
exact Stand restoration. The explicit source adapter now accepts those two
codes; other unsupported actions still report clearly.

The expanded connected received-pose test passes mixed locomotion, Dying, Dead,
unsupported 0x13 and Stand recovery, retaining root/mesh/material/atlas/UVs.
Both death captures in `target/connected-avatar-uittt3kf` were inspected. They
show a face-down textured body; the root remains at its received standing
elevation, visibly above terrain. No death placement/physics or interpolated fall
is claimed. All 786 client-library tests pass (six ignored), strict client Clippy
passes, and mounted tests pass for both default and smaller received DNA.
The wider animation goal remains active; this is not complete action coverage.

Outgoing locomotion checkpoint: the source-profile client now sends ten-channel
Stand/Walk/Run reports from actual local tangential movement, including release
to Stand and a 0.5-second heartbeat. A report is marked sent only after the
runtime accepts it, so failed sends retry. Ownership resets across sessions.
Reporting is permitted only while the actual received request is all-Stand;
mixed poses, actions and externally requested locomotion are not overwritten.
This matches the supplied server's non-echo pose profile, not a generic policy
for servers that echo client pose state. It does not change native physics,
received pose interpretation or add a fabricated server animation clock.

CPU tests cover transitions, retries, heartbeat, received-action preservation
and same-ID/new-generation reset. All 787 client-library tests pass and strict
Clippy passes. Connected keyboard acceptance now additionally requires the real
server's pose-tracking log to observe Stand→Walk→Run→Stand. That run passes,
alongside persistent mesh phases, exact stopped pose/eye, received appearance
and persisted position reports. Captures are under
`target/connected-avatar-5c_g9ye5`. This is actual supplied-server receive evidence,
not a client-generated assertion that a report was delivered.

Outgoing/received coexistence acceptance now observes the supplied server's real
delta-tick decoder, returning its original result unchanged. The connected
received-action rerun passes: neutral/bootstrap and restored-Stand client reports
are observed, but no reports override mixed/Dying/Dead/unsupported states after
a 0.3-second grace for already-queued transport. The GPU pose/resource/recovery
checks also pass; captures are in `target/connected-avatar-oesnaupi`. This is a
bounded local test, not a claim about arbitrary server echo semantics.

`just test-native-avatar-animation` now runs eight existing SHA-bound static
anchor checks, the client-library suite and mounted received-DNA checks in one
dev-shell entry. The combined run passes: 787 library tests (six ignored) and
both mounted/ownership tests. It does not execute original binaries or launch
new disassembler analysis. Connected GPU recipes remain separate because the
offline suite cannot prove rendering or network behavior.

Wave dependency checkpoint: bounded static analysis and retained PositionArms
dispatch now support the native Wave arm stage, including pose-1 rhythm,
pair phases, owner pitch and wrist-bend branches. It is ported but not admitted
by the runtime until full composition/mounted/connected verification. During
that audit, a DhDNA object-header offset error in the recent death hip branch
was corrected (packed byte12, not20). A regression distinguishes those bytes;
788 library tests and both mounted received-DNA tests pass. Earlier death
captures must be rerun before claiming the corrected branch visually verified.

Wave is now integrated on received arm channels2/5, preserving all other pose
channels rather than fabricating a uniform standing-wave request. Unsupported
Wave codes on non-arm channels remain explicit errors. Worker preparation warms
Wave before freezing mount access. Mounted tests cover either/both arms,
0/125/0-ms phase changes, exact repeats, unchanged eye, rejection of non-arm Wave,
and exact Stand restoration.

The expanded connected receive run passes through mixed locomotion, corrected
Dying/Dead, left-arm Wave, unsupported action and Stand recovery. Wave-A/B and
Dead in `target/connected-avatar-fg7sv59n` were opened and inspected: raised arm
and wrist phases change without moving the rest of the standing body or eye;
the face-down death pose retains its deliberately unchanged root elevation.
Persistent GPU identities and actual server-decoder non-overwrite gates pass.
The combined offline suite passes nine static checks, 788 library tests (six
ignored) and both mounted received-DNA tests. Broader action support remains open.

Multi-arm coverage checkpoint: a mounted test now composes arm-only Walk, Run
and Wave over Stand on the SAME body across one/two/three arm pairs, either/both
sides and both wrist-bend branches (54 combinations, each sampled 0/125/0 ms).
After the initial Stand warmup, hand/foot caches have no mount reference. It
requires one rhythm sample per selected arm, finite matrices, unchanged
unselected bones, changing phases, exact repeats and exact Stand restoration.
The matrix passes and is included in `test-native-avatar-animation` along with
all six mounted standing/limb tests. The combined nine static checks, 788 client
tests, two received-DNA tests and six mounted limb tests all pass.

The full offline GPU preview regression also passes: all 11 head/torso,
base/many-body four-view and untextured captures were produced without renderer
errors, and both captured-image nonblank tests pass. Output is under
`target/avatar-textures-uhs_hfuh`. Base-front, many-oblique and many-front were
opened and inspected; the existing textured anatomy/geometry remains consistent.
These are standing previews, not proof of animated multi-limb GPU behavior:
CPU multi-arm coverage is not substituted for complete connected action support.

Jump dependency checkpoint: the native Jump leg stage is now ported from bounded
symbol disassembly, with SHA-bound literal/dispatch checks and f32 crouch-to-Stand
interpolation. Unlike Stand/Walk/Run, it always applies the interpolated ankle,
including DNA without ankle bend. Static and targeted CPU checks pass. Mounted
Walk/Run/Jump leg coverage passes one/two/three pairs, all selectors and both
ankle branches with phase changes, exact repeats and unchanged unselected bones.
Jump arm dispatch and full composition are still required before runtime0x1c
admission; no jump physics or in-game jump support is claimed yet.
The combined offline regression passes ten static checks, 789 library tests
(six ignored), both received-DNA tests and all six mounted limb tests. Strict
client Clippy and formatting/diff checks also pass.

Jump arm/composition checkpoint: native JumpArms spread and first-only override
are now verified and ported. The internal compositor combines legs, tail, torso,
head and arms with native height/reset order. Preparation warms Jump and then
discards mount access. Full mounted received-DNA samples pass 0/125/0-ms phase,
finite-eye, exact-repeat and subsequent Stand checks. Multi-arm coverage now
includes Jump (72 arm-action combinations); a dedicated override test confirms
only the first selected arm changes. All seven mounted limb tests and 790 client
library tests pass (six ignored). The source adapter still rejects Jump pending
explicit receive admission and connected visual acceptance; no jump physics is
claimed from CPU pose composition.

Received Jump admission checkpoint: the explicit source adapter now accepts
0x1c on the ten pose channels, preserving it instead of predicting Walk/Run.
Packet-ingestion and selection tests cover it. The connected owned-server
fixture requests uniform Jump after Wave, captures two changed GPU phases and
then requires unsupported-action handling plus exact Stand restoration.
Existing root/mesh/material/atlas/UV and outgoing-report non-overwrite checks
remain active. This verifies pose animation only, not keyboard jump input,
world displacement, gravity or landing. The first connected run passes. Jump-A/B
in `target/connected-avatar-6sh8tgso` were opened: the complete textured body
changes between tucked and extended limb phases with no assembly break or root
motion. The existing source-decoder non-overwrite, persistent GPU identity and
exact Stand/eye recovery checks pass, as do all 790 client-library tests (six
ignored). Broader actions and physical jumping remain outside this checkpoint.

First-person received-action checkpoint: `just test-connected-avatar-actions-first-person`
now runs the same source-driven mixed/death/Wave/Jump/rejection/recovery sequence
through the normal animated-eye camera. Warmed-frame assertions require a real
camera, exact expected eye position (with existing terrain clearance), preserved
mouse-look rotation and hidden local head batches. The connected run passes all
pose/resource/report/recovery gates and the new camera gates. Both Jump phases
in `target/connected-avatar-zm36w_7h` were opened: terrain remains unobstructed;
hands appear at the bottom during the extended phase, without skull interior.
Strict client Clippy passes. This is not native physics or coverage of actions
that the adapter still explicitly rejects.

Fold dependency checkpoint: native fixed Fold leg/arm stages are ported from
bounded disassembly, including conditional ankle, pair/side traversal and the
first-arm override's distinct float literal. The internal compositor supports
native numeric codes0x18/0x19/0x1b/0x35 using existing height/tail/torso/head stages;
user-facing action names are not inferred from limb shape. Worker preparation
warms these requests before mount access is discarded. Full mounted received-DNA
samples pass finite geometry/eye, exact repeats and subsequent Stand restoration.
The leg selector/ankle matrix and first-arm override test also pass. Combined
offline checks pass eleven static verifiers, 791 library tests (six ignored),
both received-DNA tests and all eight mounted limb tests; strict client Clippy
passes. Source admission and connected Fold captures remain required, so the
runtime adapter still rejects these codes instead of drawing an unverified pose.

Received Fold admission checkpoint: source codes0x18/0x19/0x1b/0x35 are now
explicitly admitted, preserved through real packet-ingestion tests and given
distinct numeric HUD statuses without guessed action names. The owned-server
fixture requests each after Jump and captures two samples per code while keeping
the same root/meshes/material/atlas/UVs. Native0x1b is intentionally static;
the other three change through existing timed torso dispatch. Mounted tests
confirm that exact distinction, repeats and Stand restoration. Connected
verification passes: all four codes are received with distinct statuses, the
expected timed/fixed samples, persistent GPU identities, no outgoing action
overwrite and exact Stand/eye restoration. All A captures plus Fold18-B under
`target/connected-avatar-1q9_rh52` were opened: limbs stay folded and the timed
codes have modest torso changes. The unchanged root remains visibly above
terrain; no seat placement or physics is implied by posing. The current library
suite passes 791 tests (six ignored), and strict client Clippy passes.

Point dependency checkpoint: the native Point arm stage is ported from bounded
static evidence, preserving pitch-minus-five-degree shoulder, five-degree elbow,
ordinary-arm side traversal, stored splay/lift and the absence of wrist/rhythm.
Unlike Fold's optional override, pitch affects every selected Point arm.
Internal arm-slot composition and worker warmup are connected; source0x1f is
still rejected pending receive/connected acceptance. Pure tests, either/both-arm
mounted samples and the expanded90-combination multi-arm matrix pass, including
fixed samples, unchanged unselected bones/eye and exact Stand restoration.
The combined offline suite passes twelve static checks,792 library tests (six
ignored), both received-DNA tests and all eight mounted limb tests. Source pitch
mapping and complete action support are not claimed.

Received Point admission checkpoint: the source adapter now admits0x1f only on
arm slots2/5, preserving the other received channels and rejecting non-arm Point
requests. Packet-ingestion tests exercise the real sparse update path. The
connected fixture now requests a left-arm Point after the four Fold codes,
captures two fixed samples, checks an unchanged authored eye/root and persistent
GPU identities, then verifies unsupported-action handling and exact Stand
recovery. Point still uses owner pitch0 in the admitted normal mode; nonzero
source head-tilt mapping and targeting interaction are not implemented here.
The expanded connected run passes, including all earlier action/recovery gates.
Both Point images in `target/connected-avatar-2mag2wpy` were opened and show the
same extended arm while the rest of the standing body stays unchanged. Exact
fixed mesh samples, authored eye/root, persistent GPU resources, no outgoing
action overwrite and Stand recovery all pass. The client-library suite passes
792 tests (six ignored); strict client Clippy passes. Finger articulation,
nonzero source pitch mapping and a user-triggered targeting action are not
claimed from this arm-pose checkpoint.

Received head-tilt checkpoint: the source float-radian field is now mapped
explicitly to bounded native body30 pitch and zero body34 roll, separately from
the outgoing delta-tick i16 encoding and native Creature runtime callbacks.
Normal-body composition passes established angles to PositionHeads and pitch
to Point/Wave. Idle optimization now notices angle changes. Mounted tests prove
positive/negative pitch and roll, deterministic repeats, Point/Wave pitch with
fixed head overrides, rejection and exact zero-angle Stand/eye restoration.
Real packet-ingestion tests cover sparse omission/change/reset and finite
limits. Third-person owned-server acceptance passes in
`target/connected-avatar-vd2ue1mk`; both tilt captures were inspected and show
up/down head motion without root movement or GPU resource replacement.

The first expanded first-person run (`target/connected-avatar-8_7whe38`)
passed action, tilt and camera/skull assertions, but failed final acceptance:
the redundant post-sequence600-frame capture warmup crossed the existing
60-second local runtime bound and lost the body. The harness now retains its
initial600-frame cold warmup and both37/83 action samples, transitions after90
rather than120 frames, and uses30 final camera-settle frames for an already
warmed animation sequence. The runtime session bound remains unchanged.
If the action catalog outgrows this bound, split connected sequences into
separately owned sessions rather than weakening samples or extending the bound.
First-person revalidation passes in `target/connected-avatar-prcnc1al`, including
all mixed/death/Wave/Jump/Fold/Point/tilt/unsupported/reset stages, an explicit
positive-to-negative eye change, persistent GPU identities, animated-eye camera
tracking, unchanged mouse-look orientation and hidden skull. Final capture and
received-appearance checks complete before the unchanged session bound. This
also revalidates first-person Fold/Point, which were added after the earlier
first-person checkpoint. The failed run above is retained as failure evidence,
not relabeled as a passing gate.

Offline regression passes13 digest-bound static checks (now including head
tilt),793 client-library tests with six ignored, both received-DNA/ownership
tests and all eight mounted limb checks. No public server or original binary
execution was used. The broader animation goal remains active: unsupported
actions/anatomies, native runtime effects, user-triggered gesture/aiming
interaction and physical action transitions are not proven complete here.

Local gesture controls checkpoint: Bio now has explicit prototype Wave/Point/
Neutral buttons. The saved Controls page supplies no Wave/Point shortcuts, so
none are invented or claimed as recovered originals. Buttons use the same
action handler as the connected acceptance driver. Prediction is permitted
only over received all-Stand, replaces arm channel2 only, preserves the other
predicted gait channels and head angles, and never mutates received Person
state. A per-session/avatar request survives sparse neutral omissions but is
cancelled after each received action/unsupported state, removal or ownership
change; a later Stand cannot silently resume an old gesture. Neutral returns
the arm to the current gait, not always to an unrelated standing arm.

Rendering and outgoing reports now share the composed selected pose, removing
the duplicate report-only gait classifier. A bounded explicit inverse table
admits only Stand/Walk/Run and the local left-arm Wave/Point source bytes;
arbitrary native pose words are not cast into wire enums. Reports retain retry/heartbeat
behavior and zero auxiliary fields; mixed gesture reports do not flood every
frame. Packet-ingestion/action tests verify received-state authority, sparse
omission, action rejection/cancellation, same-ID generation change, removal/
restore and explicit Neutral. The client library passes797 tests (six ignored).

`just test-connected-avatar-local-gestures` passes against the owned source
server. No received pose is injected or echoed in this mode: the wrapper only
observes the original delta-tick decoder and returns its original result. The
driver selects gestures through the normal action handler and uses actual
W/Shift keyboard input for Point+Walk and Wave+Run. It verifies changing Wave
and gait phases, fixed Point, root movement only while moving, persistent GPU
identities/UVs, unchanged received channels and exact neutral geometry/eye.
The actual source decoder observes all five composed channel vectors with
auxiliary i16/i32 both zero. This proves dispatch and protocol delivery, not a
mouse-click automation or source-server pose echo/remote-peer render.
Wave-A, Point-A and Wave+Run-B in `target/connected-avatar-ad4d35vu` were opened:
the selected arm is visibly raised/extended while the other limbs retain their
standing/running behavior. Finger articulation and mouse-driven targeting remain
unimplemented; these buttons do not fabricate either.

The complete received-action/tilt/unsupported/recovery regression also passes
after the local-control changes (`target/connected-avatar-qtdpjx9e`), including
actual decoder evidence that received actions are not overwritten by local
reports. This is independent of the local-gesture mode's no-injection proof.
The final offline bundle passes all13 static verifiers,797 library tests (six
ignored), both received-DNA/ownership tests and all eight mounted limb tests;
strict client Clippy and formatting/diff checks also pass.

Received crouch checkpoint: native `DaCreature::IsCrouched` identifies0x10/0x11.
The new SHA-bound verifier checks that predicate and both leg/arm-side dispatch
tables (including the second leg table's minus4 index): stationary crouch reuses
Stand stages; crouch-walk reuses Walk stages. Existing CGHeight RelativeStanding,
torso/head/tail dispatch determines the rest, not guessed joint angles. See
`native-avatar-locomotion.md` for evidence IDs and exact DNA-dependent factors.
Both received codes are admitted explicitly, warmed off-thread, and sampled
using cached-only hands/feet. Mixed channels remain intact; local gestures and
pose reports cannot overwrite crouch. The source-report inverse table still
rejects local crouch prediction until control/placement behavior is implemented.

Mounted received-DNA sampling passes fixed0x10 and animated0x11 at0/125/0 ms,
finite eyes, deterministic repeats and exact Stand/eye restoration. Real packet
tests include both received codes. `just test-connected-avatar-received-crouch`
uses a separate bounded owned-server session rather than extending the full
catalog sequence beyond the unchanged60-second runtime bound. Third-person
acceptance passes in `target/connected-avatar-2mulmqwp`: both phase captures,
unchanged root/mesh/material/atlas/UVs, exact Stand recovery and actual source
decoder evidence of no action-overwriting reports. Crouch-A and both crouch-walk
captures were opened: bent knees and differing gait phases are visible.

The fixture sends pose-only updates at standing root elevation. The images
therefore show a hover; this explicitly does not establish native crouch ground
placement. Held Caps Lock, movement-rate/collision changes and CG transitions
remain work, not a hidden render-only offset. Prone mapping was located but no
prone implementation or keyboard behavior is claimed in this checkpoint.
No original binary or public server ran; retained REA evidence and bounded GNU
symbol/table checks avoided a new Ghidra import.

The same crouch sequence with `OPENSHORES_TEST_AVATAR_ANIMATION_FIRST_PERSON=1`
passes in `target/connected-avatar-16gscyfc`, running animated-eye, mouse-look and
hidden-skull assertions throughout. The offline bundle passes14 static checks,
797 library tests (six ignored), both received-DNA/ownership tests and all eight
mounted limb checks. Strict client Clippy, formatting and diff checks pass.

Posture placement follow-up: Stand and both crouch codes share native zero
CGOffset/no-op render adjustment; adult nominal ground clearance is CGHeight(true).
Prepared animations now retain immutable Stand/Crouch clearance metadata computed
off-thread. Default received DNA measures2.957287/1.9715247; smaller DNA measures
2.5704176/1.7136118. These values do not translate received roots automatically.

A real source correction bug was found: cached local/report positions could
mask later authoritative transforms. World ingestion now returns explicit
transform receipt metadata, even for unchanged coordinates, without a second
decode. Selected-player receipts reconcile model/UI predictions; sparse/other
entity packets do not. Quantized echoes preserve f64 precision and newer unsent
input; differing receipts clear obsolete dirty reports and update heading while
preserving mouse pitch. This is an explicit source-profile reconciliation policy,
not native acknowledgement or out-of-order echo proof.

The new grounded-crouch fixture supplies authoritative roots using Rust's nominal
clearances. Initial first-person acceptance passes in
`target/connected-avatar-8g_px72y`, including exact root/Stand recovery, persistent
resources, animated-eye/mouse-look/skull assertions and no action overwrite.
Explicit receipt/reassertion and newer-unsent-input refinements then passed CPU
regression and final third-person revalidation in
`target/connected-avatar-ik2zmjdl`. Crouch-A and crouch-walk-B were inspected;
the received lower root is visible, with the contact discrepancy below preserved.
The pose-only fixture is retained separately. See
`native-avatar-world-placement.md` for scope and limits.

Contact remains imperfect: default stationary crouch has a measured skinned-sole
residual-0.38020396 at nominal clearance. No geometry-fit offset or collision
parity is claimed. Caps Lock input and native physical posture transitions remain
unfinished; this correction/metadata work is their prerequisite, not completion.

Final regression passes15 static verifiers (now including Stand/Crouch anchor
checks),798 client-library tests with six ignored, both mounted received-DNA/
ownership tests and all eight limb tests. Strict client Clippy passes. The local
gesture/keyboard Walk/Run/neutral workflow also passes on the final reconciliation
path (`target/connected-avatar-wty3t6be`), including actual source-decoder channel
delivery and persistent render resources. No public server or original binary
execution was used. The default library configuration passes749 tests (one
ignored); final formatting and diff checks pass.

Held-crouch checkpoint: Caps Lock now selects local0x10/0x11 over received
all-Stand, with optional arm gestures composed afterward. The matching prepared
adult (source growth counter0), visible globe-child actor and current terrain
are required. Near-ground positions follow the actual terrain mesh at cached
native nominal Stand/Crouch clearance; off-ground positions and received actions
are not snapped. Shift is capped to the existing source-slice Walk rate while
crouched, not claimed as a reconstructed native movement-speed modifier.
Chat/hand-cursor/Alt/Ctrl modes suppress crouch; vertical C/Z input exits it while
retaining vertical displacement. Pose reports remain explicit source-profile
mappings with zero auxiliary fields. Position reports now flush after posture
adjustment, avoiding a stale standing-height report ahead of the crouched frame.

Requests are bound to session/avatar/prepared appearance. Real corrections and
changed owners require release before a held request can reapply. Losing a body
undoes only the same actor's still-grounded local stance, never another session
or an already-cancelled correction. CPU tests cover these guards, radial
grounding/release at both poles and an equator, modal requests, off-ground/invalid
inputs, received authority and composed gestures. No mesh-bounds fit or frame
asset I/O was introduced; the known crouched sole residual remains unchanged.

`just test-connected-avatar-local-crouch` passes through actual Caps Lock/W/Shift
ButtonInput and normal controls, without server-pose injection. Stationary,
crouch-walk, release-to-walk, Shift+crouch and final Stand are checked against
native clearances and persistent root/mesh/material/atlas/UV identities. The
actual source decoder receives all expected channel vectors with zero auxiliary
fields. Three captures in `target/connected-avatar-cck8wodk` were inspected,
showing bent-leg stance/gait and standing recovery. First-person acceptance also
passes in `target/connected-avatar-616hxn6e`. The complete received-action,
tilt/unsupported/recovery regression passes in `target/connected-avatar-n4zbwzru`.

After restart, a fresh first-person run (`target/connected-avatar-hcjhu9_q`)
failed during warmup: the checker forced its pitch after camera evaluation,
allowing ambient desktop mouse events to create an inconsistent expected view.
View setup now precedes controls/camera evaluation; the opt-in driver owns its
keyboard/mouse inputs. The orientation assertion is unchanged. A new CPU
regression verifies fixture setup against the real avatar camera system.
Fresh first-person acceptance passes in `target/connected-avatar-j2nwvkff`, with
animated-eye, mouse-look and hidden-skull assertions throughout. The failed run
is not counted as passing evidence. Native collision/headroom/gravity, growth
transitions, broader action/anatomy support and the sole discrepancy remain open.

Final held-crouch verification: the offline bundle passes all 15 static
verifiers, 803 client-library tests (six ignored), both mounted received-DNA/
ownership tests and all eight mounted limb tests. The default library passes
750 tests (one ignored). Strict client Clippy and formatting/diff checks pass.
The fresh complete received-action/tilt/unsupported/Stand-recovery workflow
also passes with isolated fixture inputs in `target/connected-avatar-mderewgw`.
Only the owned local server was used; no original executable was run.

Side-facing death dependency: native left/right arm stages are now ported,
including asymmetric shoulder/wrist/elbow/lift operations, signed pair index,
ordinary-arm filtering and all selectors. The new SHA-bound static verifier
checks both body and concrete part consumers. CPU tests verify selection,
repeatability, distinct facing directions and wrist-only anatomy effects.
The client library passes805 tests (six ignored). This is a dependency toward
complete0x13/0x14 support: the leg stages and connected acceptance remain open,
so the compositor and source adapter still reject those codes. See the
locomotion ledger for source locations, exact operations and analysis-cost notes.
All16 offline static verifiers, both mounted received-DNA/ownership tests,
eight mounted limb tests, strict client Clippy and formatting/diff checks pass.
The optional fresh Ghidra pseudocode request timed out during whole-image
analysis and supplied no evidence; its session was closed and owned process
termination checked. The retained SHA-bound GNU disassembly supplies the static
evidence for this checkpoint instead.

Side-death leg checkpoint: both native stages and normal-body composition now
support0x13/0x14. The cached sampler warms both; received-DNA samples verify
their finite, repeatable geometry and eyes. A mounted regression covers one/
two/three leg pairs, ankle/no-ankle anatomy, first/later/all selectors,
unchanged other bones and exact Stand recovery. The client library passes807
tests (six ignored); all17 static verifiers, both mounted ownership/appearance
tests and the new mounted side-leg test pass.

The source codes remain gated for a concrete fidelity dependency: native render
adjustment rotates the whole body aroundY by +PI/2 for0x13, -PI/2 for0x14,
and PI for already-admitted0x15. Existing0x15 captures prove skeletal posing
but do not establish correct whole-body orientation. This newly identified
gap must be fixed alongside enabling side-death receipt; geometry, normals and
animated eye must share the adjustment while the authoritative root stays intact.
Raw dispatch, exact f64 constants and RotateY are retained and SHA-checked.

