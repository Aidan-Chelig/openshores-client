# Neutral avatar origin and ground clearance

Prone/crawling follow-up: the adult globe-child sampler now includes the native
head-derived local draw translation. The actor root remains the source transform;
this resolves a render adjustment, not the remaining posture/contact physics.

Death-pose follow-up: the connected sampler now applies the separate native
render-only Y rotation for general0x13/0x14/0x15 to sampled geometry, normals
and the animated eye. The actor root remains the received/predicted world
transform. This changes draw orientation, not nominal collision clearance,
gravity, landing or corpse settling; see the final locomotion checkpoint.

Scope: adult normal-skin body-origin and ground-clearance checkpoints, initially
neutral Stand, with a Stand/Crouch follow-up below. Native observations are
static; mounted Rust measurements and bounded connected fixture proofs are
identified separately. The original game was not executed.

## Adult prone and crawling draw anchor

The `0x20`/`0x21` CGOffset dependency is now traced through its actual render
caller. Person/Creature vtables bind slot `0xf8` to `DaCreature::CGOffset()`.
The wrapper suppresses offsets for vehicle parents. In the normal nonvehicle,
no-growth branch, both general codes select negative `DpBody::HeadLocation()`.
Native HeadLocation selects the first head, or the second when the dual-head
DNA flag is set and field3a is false. It projects the selected head's stored
field50 through current bone7/12. This is not authored EyePosition and not a
mesh bound. Its general `0x37` and missing-head branches return zero.

`RenderEnvironmentObject` calls BodyPoseAdjustTransform at `0x1409a17ca`, then
CGOffset through slot `0xf8` at `0x1409a1a19`. The ordinary parent path copies
that matrix at `0x1409a2748`, loads the returned XYZ at `0x1409a27f5` and
post-translates at `0x1409a1c53`. Thus the local draw transform is rotation
followed by translation (`R*T`, acting on points as `R*(point+offset)`). The
actor transform itself is not changed by these draw operations. Vehicle,
creature-parent/rider and special seating branches have different gates and
are outside this connected renderer's existing globe-child scope.

`tests/native_prone_placement.py` binds these typed vtable entries, dispatches,
head selection/bone fields and caller operations to the executable digest.
Disassembly is retained in `evidence/native-prone-placement-callers-gnu.txt`.
These are static observations, not original execution or general wire parity.

The sampler now derives HeadLocation from the current posed skeleton on each
prone/crawl sample and applies its negative offset to fresh draw corners and
the animated eye. Weighted vertex XYZ receives offset multiplied by W; W,
source identity, UVs/materials and normals remain intact. Existing death rotations
retain their order. Prepared surfaces and bones are not adjusted a second time.
The source adapter requires received growth state zero for these general poses;
unknown/nonzero growth is explicitly unsupported. Arm-only prone/crawl channels
over another general pose do not change its draw origin.

This does not supply native grounded prone/crawl movement, headroom, growth
transitions or collision. The pose-only fixture still retains its standing root;
remaining hover/contact differences must not be corrected by mesh fitting.

Target: `reference/openshores-full/OpenShores.exe`, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
`just test-native-avatar-anchor` checks the digest, raw signed dispatch entries
and symbol-backed GNU disassembly. It executes neither the target nor a DLL.

## Static observations

- `DaCreature::CGOffset(DpBody const&, ...)`, `0x140978ac0`, reads general pose
  from body offset `0x48`. Stand `0x24` selects table `0x1421c5e34` entry
  `0x24-3`, target `0x140978b10`, which writes three zero doubles. Stand does
  not request a head-location or other nonzero CG offset.
- `DaCreature::BodyPoseAdjustTransform`, `0x140980830`, reads the same general
  pose. Table `0x1421c664c`, entry `0x24-4`, selects `0x1409808e7`, the return
  epilogue. This Stand branch applies no extra pose rotation or translation.
  `RenderEnvironmentObject` calls it at `0x1409a17ca`.
- `DaCreature::GroundHeightAboveCG`, `0x14097c670`, special-cases poses `0x39`
  and `0x0f`; ordinary Stand instead tail-calls `CGHeight()` at `0x14097c6d4`.
- The active-body, no-growth branch in `DaCreature::CGHeight()` requests
  `DpBody::CGHeight(true)` at `0x14097c57b`, calls `CGOffset` at
  `0x14097c595`, then subtracts its Z component. That component is zero for
  Stand. This is distinct from `AnimatePose`'s previously verified
  `CGHeight(false)` used to position bones.

The existing retained `native-dp-pose-height.json` record is REA Evidence
`ev_c744263a605d92e3c91bd0a3854a311a5900956ad1e6fed8c260dd1e63a46477`;
its pose-height implementation is documented in
[the native model plan](native-model-application-plan.md).
The additional caller/dispatch checks above are GNU static evidence, not new
REA decompiler records. The old local REA snapshot was rejected for an analysis
profile mismatch; no cold import was started to replace it.

## Mounted measurement and implementation

`NativeDpAvatarGeometry::standing_ground_clearance(mount)` reuses the existing
pose-height provider with `include=true`, and rejects a changed/non-neutral pose
context. It supplies nominal clearance metadata, not an offset to add to the
render root. Growth, other poses and vehicle-parent contexts are not inferred.

`just test-local-avatar-preparation` takes the supplied server's actual default
DNA through the receive adapter and measures:

| Quantity | Native model units |
| --- | ---: |
| Skeleton pose height, `CGHeight(false)` | 2.6520436 |
| Nominal ground clearance, `CGHeight(true)` | 2.957287 |
| Drawn minimum Z | -3.0233366 |
| Drawn maximum Z | 3.00644 |
| Minimum Z plus nominal clearance | -0.066049576 |

The body straddles its model origin; that origin is not the feet plane. The
native nominal clearance and authored/deformed sole envelope differ by about
0.066 units for this DNA. These measurements are diagnostics, not a reason to
move every avatar by its bounding-box minimum. No renderer translation, terrain
snap or native-unit rescaling has been added.

## Placement consequence and remaining proof

For a controlled nominally landed Stand fixture, test an authoritative body
origin at terrain radius plus the native ground clearance; test an otherwise
identical off-ground fixture higher by an explicit known distance. Render both
at their received coordinates. This is the next test hypothesis, not a completed
native wire-origin/terrain-contact proof. Check actual coarse-mesh terrain height
as well as analytic terrain: the two can differ on this diagnostic globe.

Remaining work must establish that the source-server bootstrap and correction
path preserve that origin convention, verify scale against world geometry and
inspect landed/off-ground GPU output. Preserve the small native sole residual
explicitly rather than silently applying a geometry-fit correction. The existing
first-person camera's fixed 1.7-unit offset and terrain-clearance clamp are
prototype camera policy, not evidence for a body-placement offset.

## Connected north-pole fixture verification

The controlled hypothesis above is now tested by `just test-connected-avatar`.
The harness obtains the clearance from the Rust received-DNA pipeline and seeds
the disposable server at the north-pole terrain radius plus that clearance.
After the first capture it updates the owned account to five units higher and
starts a second authenticated session. The reference server source is unchanged.

Both sessions verify received position and render-root position independently
against the seeded coordinates (0.01 native-unit tolerance for f32 wire/display
roundoff). An inspection-only gold ruler uses one world unit per tick, with its
base at the radial intersection of the actual terrain mesh. At the north-pole
vertex that mesh height agrees with the analytic fixture height. A 0.02-unit
line lift avoids z-fighting; it changes no body or terrain position. The camera
includes the ground reference in its view rather than recentering the body alone.

The inspected captures in `target/connected-avatar-1ffbaeup/` show nominal landed
feet at the ground cross, retaining the small native sole residual, and airborne
feet roughly five ticks above it. Body height is roughly six native world units,
consistent with its measured native Z extent rather than preview normalization.
Both cases pass origin checks; no shader or hierarchy errors were emitted.
This verifies this adult source-profile north-pole fixture, not all original
native environments or anatomy. Southern connected placement, appearance changes,
movement/turning and ordinary first-person-camera acceptance remain separate gates.

Mounted quick-dev/play spawns now use the same native nominal clearance instead
of the previous arbitrary +5. This only changes owned disposable server seed
data, never received coordinates inside the renderer. Other diagnostic workflows
retain their earlier fixture behavior.

## Southern connected frame verification

The negative-Z fixture now passes landed, five-unit airborne and live-turn/update
checks through `just test-connected-avatar-south`. Received model +Z must point
radially outward and the render quaternion must match the received rotation;
no additional gravity rotation is allowed. The live test verifies that bootstrap
frame before local prediction and retains mesh identities while turning.

The supplied selected-player bootstrap hard-codes zero rotation. The explicit
test-server wrapper supplies an outward orientation using its existing gravity
helper for the southern person record. This bounds the result to correct
source-profile received transforms; it does not prove unmodified selected-player
bootstrap at the south pole or original native environment parity. See the
[progress ledger](in-world-avatar-progress.md) for inspected captures and gates.

## Adult Stand/Crouch clearance and source correction bridge

The SHA-bound anchor check now also verifies0x10/0x11. Both share Stand's zero
CGOffset target140978b10 and no-op BodyPoseAdjustTransform target1409808e7.
Neither matches GroundHeightAboveCG's39/0f special cases, so the existing adult
no-growth ground rule remains CGHeight(true), not AnimatePose's false height.
`stand_crouch_ground_clearance(mount, crouched)` evaluates those two contexts
from the neutral normal-body anatomy. `PreparedAvatarAnimation::ground_clearances`
returns immutable cached metadata after preparation; it performs no runtime I/O
and never moves a render root. Default received DNA measures Stand2.957287 and
Crouch1.9715247; the smaller test DNA measures2.5704176 and1.7136118.

The source UI previously retained its bootstrap/local position even after a
received selected-player transform changed, while ClientModel preferred the
last sent report indefinitely. That could mask an authoritative crouch root.
World ingestion now exposes explicit transform receipts, including identical
coordinates, without decoding the atom twice. The model tracks selected-player
receipts separately from sparse omissions. A receipt that differs from the
last written report clears report caches; UI correction replaces local position/
rotation, updates heading, and clears the obsolete dirty prediction. It leaves
mouse pitch unchanged. Unrelated entities and sparse packets do not reset motion.

Matching f64 reports rebroadcast as f32 are treated as echoes, preserving local
precision and newer unsent input. This is an explicit source-profile policy,
not native acknowledgement reconstruction: no sequence number is invented and
an old echo cannot universally be distinguished from a correction without one.
CPU packet tests cover corrections, identical-coordinate reassertions, sparse
omission, other entities, quantized echoes, newer unsent input and rotation-only
updates. Local report values never overwrite the received WorldState transform.

`just test-connected-avatar-grounded-crouch` selects an explicit owned north-pole
fixture whose Stand/Crouch root positions use the Rust measurements. It does not
duplicate the height formula in Python or add a render offset. Set
`OPENSHORES_TEST_AVATAR_ANIMATION_FIRST_PERSON=1` to include camera/skull checks.
This is a controlled authoritative-transform test, not a native collision or
Caps Lock controller. The previous pose-only crouch recipe remains unchanged.

Nominal clearance is not exact skinned-foot contact. Default stationary crouch
currently measures minimum Z-2.3517287, leaving a-0.38020396 residual at nominal
clearance. This is a reconstructed geometry/contact discrepancy, not established
original-game parity. It is recorded rather than hidden with mesh-bounds fitting.
Native posture/physics transitions, growth and other parent contexts still need
their own implementation/evidence. The bounded held-crouch controller below is
separate from those native runtime requirements.

## Grounded held-crouch source controller

The source slice now uses held Caps Lock for local crouch prediction over received
all-Stand only. It requires the matching prepared adult body (source growth
counter0), a visible selected globe-child actor and the current terrain mesh.
The pre-input radial position must be within0.02 native units of that posture's
nominal clearance plane. Airborne/buried positions and received actions are not
snapped. Once grounded, ordinary walking and crouch follow the actual mesh's
radial surface; release restores Stand clearance. This changes the predicted
position that is reported, not a hidden mesh/camera offset or received atom data.

This is declared source-controller policy, not reconstructed native collision
physics. Crouch uses the existing slice Walk rate; Shift does not select Run while
crouched. Vertical C/Z motion exits crouch without cancelling the vertical input.
Chat/hand-cursor/Alt/Ctrl modes suppress local crouch. New owners/prepared profiles
and real source corrections inhibit a held request until release. Loss of a
prepared body undoes only a still-grounded same-actor local stance; old metadata
cannot move another session or an already cancelled correction.

The frame order is input -> posture/terrain adjustment -> position report ->
pose report -> persistent skinning -> camera. Explicit inverse source mapping
now admits predicted0x10/0x11, with optional arm gestures composed afterward.
No native movement-speed modifier, headroom/building collision, gravity, growth
transition or physical received-action transition is claimed. The measured
crouched sole residual above is unchanged; no mesh-bounds fitting was added.
