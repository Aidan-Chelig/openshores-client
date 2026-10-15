# Assembled avatar: evidence and implementation ledger

Objective: [complete assembled avatar preview](assembled-avatar-goal.md).
CPU full-body source generation and the Stand skeleton are implemented. Final
posed render geometry, `preview-avatar body` and visual verification remain unfinished.

## Initial audit and coordinate boundary

Existing code provides ordered native part plans; verified head/torso geometry
merges; per-part deformations and attachment points;61 primary bone matrices;
many individual bone/pose operations; and the textured part renderer. It does
not yet compose a complete neutral-pose generator. The CLI head/torso views
remain unposed diagnostic parts, not a standing avatar.

Keep these stages distinct:

1. Decoded native stored vertices and group-membership coordinates.
2. Reconstructed part deformation / one-time merge baking.
3. Triangle-builder projection through the original seed object's matrix.
4. Bone skinning of those model-space coordinates using the posed primary
   matrices, narrowed to f32 for GPU upload.
5. Body/view/world projection and the inspector's display-only framing.

Applying object projection or generic parent-bone multiplication again at stage4
would double-transform vertices. Native part routines compose primary bone
matrices before upload; the vertex shader itself does not traverse parents.
The high-level order and full standing pose still need implementation.

## CPU skinning milestone

`NativeDpBoneTransforms::skin_shader_vertex` implements the supplied body shader's
four ordered influences. It accepts a projected/deformed f32 model position,
model-space corner normal and untouched native weight payload. Matrices narrow
from stored doubles to uploaded floats; positions use mat4 multiplication,
normals use the upper mat3, and the final blended normal is normalized. There
is no inverse transpose or automatic weight renormalization. Zero-weight
influences bypass their bone indices, including -1 and negative zero.

The result retains all four position components. In particular, it is NOT
`AuMatrix4x4::Project`: there is no perspective divide or zero-column shortcut.
A later renderer adapter must preserve the homogeneous semantics rather than
silently discard non-unit W. CPU/GPU FMA and normalization rounding may differ;
bit-identical GPU execution is not claimed.

Nonfinite inputs, active invalid indices, f32 matrix overflow, absent influences,
zero homogeneous W and zero/nonfinite blended normals return Unsupported. No
fallback bone/normal or fabricated neutral position is substituted. These are
explicit Rust preview safety boundaries, not claims that native GLSL validated
its inputs. Raw geometry, payloads and bone state remain unchanged.

Four unit tests cover known rotation/translation blends, inactive invalid bones,
normal translation exclusion, non-unit homogeneous W, preservation of weight
sums, matrix precision, mat3 versus inverse-transpose behavior and malformed
inputs. `native_body_mounted_skinning` independently checks the mounted
`Body.v.glsl` contract through the existing Qt/HAR readers, with no GPU, network
or reference execution.

Evidence: `evidence/native-dp-render-skin.json`, REA
ev_af9cc8fdb6fa22f65648aac8c12d75f8990df1203bd28fa9a39d6bacb6cb689a,
establishes primary-matrix float conversion and transpose=false uploads.
`native-glsl-body-build.json` and `native-vao-create-for-body.json` establish
the shader/attribute bindings. The mounted shader consumes boneIndex/Weight
directly and passes weighted homogeneous positions into the view matrix. The
user's shader source is not copied into the repository.

## Neutral-pose follow-up

Retained AnimatePose orders height calculation, selected matrix reset, legs,
tail, torso, heads, arms, and conditional wings. Existing PositionLegs/Arms/Torso
records are already retained; do not re-query them. Pose0x10 is a candidate
standing branch in those dispatchers, but selecting complete pose words and
reconstructing StandLegs/StandArms plus torso/attachment composition is still
required. Do not call an all-identity skeleton a verified standing pose.

StationaryPose's newly retained record has an apparent receiver/return ABI
ambiguity and must not be translated into a write to DpBody's first word.
This function is not used by the new skinning helper.

The old warm analysis process closed on a duplicate-output guard; see the
workflow update. Saved evidence remains available and the driver is now tested
to keep a live session open when duplicate filenames occur. No replacement
import was started merely to recover already retained results.

Next: implement the required standing-pose/attachment chain and reusable full
CPU part generation, compose skinning once, then extend the textured renderer,
camera framing and multi-view/two-DNA GPU verification. None of those remaining
completion gates is satisfied by the isolated skinning primitive.

## Standing-leg angle calculation

`NativeDpFootCache::leg_stand_angles` now reconstructs
`DpLegAndFoot::CalcStandAngles` (VA1412d8dc0). Owner DNA selects the measured
foot; part DNA selects ankle, exoskeleton and leg-vertex branches. Dimensions
come from owner fields409c/4080. The result is hip/knee/ankle angles only, not
a posed skeleton. The concrete caller's Bones resolution and subsequent
RotateHip, RotateKnee, RotateAnkle and SplayHip still require composition.

Retained full GNU records are `evidence/native-leg-calc-stand-angles-gnu.txt`,
`native-dna-leg-vertices-gnu.txt` and `native-dna-is-exoskeletal-gnu.txt`.
`tests/native_leg_standing.py` validates their contents against the supplied PE
with SHA-256285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80,
and pins all six embedded float/double constants. This is static reconstruction,
not execution of or runtime equivalence against the native client.

The implementation retains strict triangle inequalities, the alternate
exoskeletal/crowded-leg solution, foot-angle subtraction, native f32 squares
with f64 division, and the widened f32 half-pi constant used by the fallback.
It does not clamp invalid acos/asin arguments. Nonfinite inputs or final angles
return Unsupported as an explicit preview safety boundary. Four solution tests
and an exhaustive packed-count selector test cover these branches. Run
`just test-native-standing`; the full library suite passes 696 tests with one
unrelated opt-in test ignored, and library/test Clippy passes with warnings denied.

Further retained records: `native-leg-stand-gnu.txt`,
`native-dp-stand-legs-gnu.txt`, `native-dp-spread-legs-gnu.txt`. StandLegs first
computes SpreadLegs, then visits the contiguous leg suffix in reverse order.
Its selector distinguishes pair index0 from later pairs, NOT left from right:
field4c is the pair index for both sides, as initialization already proves.
Field48 is initialized to zero and later receives SpreadLegs' splay angle.
SpreadLegs groups by exact unposed ConnectLoc and counts all matching legs
before applying its selector; replacing that with guessed side-based offsets
would change native anatomy. These wrappers are implemented in the following milestone.

## Composed standing-leg phase

`NativeDpPartState` now exposes native hip, knee, ankle and splay operations and
their ordered `stand_leg` composition. Hip/splay update all three concrete Bones
outputs, knee updates outputs2/3, and ankle updates output3. Knee projects field50
through output2; ankle projects field68 through output3. Hip and splay obtain the
current attachment through output1. The native expanded X-pivot arithmetic agrees
with the already reconstructed shoulder/wrist helpers, which are reused without
replacing their operand order with a generic affine shortcut. Splay has its own
expanded Z-pivot and concrete IsLeft sign selection.

The Leg joint state now retains native splay field48, initialized to zero and
preserved by OneTimeCalcs. `NativeDpInitialization::spread_legs` counts exact
unposed connection groups before filtering by pair index, then assigns cumulative
angles in reverse part order. It retains the anatomy-dependent requested-angle
versus PI/(count+1) branch and does not clear unselected fields. Nonfinite map
keys/inputs are rejected as a Rust preview safety boundary. Map ordering is native
Z/Y/X, not an epsilon-based weld. `stand_legs` composes that phase with measured
angles and per-part rotations. Its native reverse-loop sentinel excludes indices0
and1, whereas SpreadLegs excludes only index0; ordinary initialized bodies have
torso/head before all legs, but the distinction is retained explicitly.

New full evidence: `native-leg-{hip,knee,ankle,splay}-gnu.txt`. The static checker
also pins both concrete IsLeft vtable slots and the 0x3eb2b8c2 spread-angle literal.
Seven CPU tests cover both sides/all three pair indices, translated pivots,
unaffected bones, rotation order, no-ankle behavior, zero/invalid-index handling,
group counts before selection, retained skipped state, triangle spacing, exact
attachment grouping and suffix termination. The opt-in mounted test composes
actual foot measurements, angle solutions and all selected bone writes for two
anatomies; it checks finite matrices, fixed hip attachments and unchanged non-leg
bones. These tests do not establish final ground placement or assembled geometry.

Verification: full library 703 passed, one opt-in ignored; library/test Clippy
passes with warnings denied; static checker and mounted standing test pass.
The client-feature library/binaries also pass (733 library,10 CLI and1 slice
test, with six opt-in tests ignored); formatting and diff whitespace checks pass.
`just test-native-standing` runs targeted CPU/static/mounted checks offline.
No original client or public server was executed or contacted; no new analysis
import was needed. The skill guided focused retained-evidence reconstruction.

Remaining: standing arms and complete pose-word selection/composition, reusable
all-part geometry generation, one-time projection/skinning integration, textured
full-body viewer and two-fixture multi-view GPU/visual verification. The complete
avatar goal remains active; this milestone adds no `preview-avatar body` command.

## Standing-arm calculations and spacing

`NativeDpPartState::arm_stand_angles` reconstructs
`DpArmAndHand::CalcStandAngles` (VA1412d2000). It resolves concrete Bones first,
measures owner HandLength(false), and adds the first reference bone's projected
origin Z to the requested height with native f64-to-f32 narrowing. This is NOT
the stored ConnectLoc's Y coordinate. The solver uses owner dimensions4088 and
40b4, includes hand length in the lower segment only without wrist bend, and
retains the 95%-reach and double-precision -1e-5 comparison. Raw shoulder/elbow
angles feed wrist compensation BEFORE the output sign/angle conversions.

The alternate exoskeletal/invalid-triangle branch can replace nonfinite initial
acos values with finite angles. Rust therefore validates final angles rather
than rejecting those temporary values. It does not clamp trigonometric domains
or substitute a fabricated pose. Nonfinite input/final values return Unsupported
as a preview safety boundary, not as a claim of native input validation.

`NativeDpInitialization::spread_arms` now stores native arm fields48 (lift) and4c
(splay), initialized to zero and preserved by OneTimeCalcs. It scans forward,
skips wing-bearing arms, groups exact unposed attachments in Z/Y/X order, and
increments both angles before side filtering. Selector0 is left,1 right,2 both;
unselected members still consume spacing increments and retain prior fields.
This differs from leg spreading, whose pair-index selector skips increments.
There is no group-count division in the arm routine. These routines compute
angles/state only; they do not yet compose the full standing-arm bone phase.

Complete static records: `native-arm-calc-stand-angles-gnu.txt`,
`native-dp-spread-arms-gnu.txt`, `native-dp-stand-arms-gnu.txt` and
`native-arm-relax-gnu.txt`. `tests/native_arm_standing.py` verifies them against
the digest-checked supplied PE, twelve constants and both concrete IsLeft
vtable returns. No fresh analysis import or reference execution was needed.
The reverse-engineering skill kept reconstruction focused on these dependencies.

Four CPU solver tests cover both wrist modes, origin-Z contribution, reach
capping, alternate branches and invalid domains. Four spacing tests cover
coincident anatomy, side selection versus pair selection, accumulation before
filtering, wings, retained state, exact grouping and nonfinite keys. The opt-in
mounted standing suite now also verifies both wrist modes with actual hand
measurements for all six arms, projected-origin offsets (including non-unit W),
and preservation of spacing through joint recalculation, without modifying
owner bones. This is a
measurement/angle check, not a visual verification of standing arms.

Next arm dependencies are SplayShoulder and LiftShoulder, then Relax's actual
shoulder/wrist/elbow/splay/lift order and StandArms' first-selected-arm override.
StandArms reads WalksOnHands to select its lift literal; its boolean override is
consumed only by the first selected ordinary arm, not every arm. Both wrappers
are retained for composition; they are not yet implemented or rendered.

Verification for this milestone: 711 library tests pass, one opt-in ignored;
both opt-in mounted standing tests and the arm static-evidence checker pass.
Library/test Clippy with warnings denied, formatting and whitespace checks pass.
Client-feature regression checks pass:741 library,10 CLI and1 slice test, with
six opt-in tests ignored. The expanded standing task's commands validate in
`just --dry-run test-native-standing`.

## Composed standing-arm phase

`splay_arm_shoulder` reconstructs the native Z-pivot with a negated angle on
the left. `lift_arm_shoulder` reconstructs the Y-pivot with a negated angle on
the right. Both modify all three concrete Bones outputs and obtain the pivot
from the current attachment, not rest coordinates. Unlike splay, LiftShoulder
has no zero-angle early return: it resolves the bones, performs the matrix
arithmetic and clears the represented matrix cache byte even for +/-0.

Native SplayShoulder's first output and LiftShoulder's last output use inlined
matrix products, while their other outputs call AuMatrix4x4::operator*. Rust
retains those specialized products, the constant-one Z term in the splay
consumer, SIMD addition ordering, nonaffine fourth rows and executed zero
products. Expanded pivots retain native arithmetic rather than flattening the
operations into guessed translations or a generic humanoid rig.

`relax_arm` now composes shoulder, wrist, elbow, stored splay, then stored lift.
An explicit optional owner field30 supplies the native override: elbow becomes
field30 minus the calculated shoulder and wrist becomes literal0xbd0efa35.
`stand_arms` selects the owner WalksOnHands lift literal, spreads arms first,
then visits selected ordinary arms forward. The override is consumed only for
the first selected arm; wing-bearing and unselected parts do not consume it.
These APIs are one animation phase, not an implicit matrix reset or a complete
AnimatePose/standing-avatar constructor.

Full static source records are `native-arm-lift-gnu.txt` and
`native-arm-splay-gnu.txt`, checked by the expanded digest-bound
`tests/native_arm_standing.py`. Retained Relax/StandArms records establish
composition order. The reverse-engineering skill guided this focused static
reconstruction; no new Ghidra import, reference execution or network test was
needed.

Four CPU tests cover axis/sign/pivot behavior for both sides and all pair
indices, untouched bones, zero-lift versus zero-splay semantics, explicit Relax
composition/override, nonaffine products and nonfinite zero products. The new
opt-in mounted test checks the full arm phase across40 combinations of side
selector, first-arm override, wing presence and WalksOnHands class, using real
hand measurements. It compares component composition, counts selected arms,
checks finite matrices/fixed attachments and verifies non-arm bones are unchanged.
All three mounted standing tests pass. These are CPU bone/measurement checks;
they are not assembled-geometry or visual standing-pose proof.

Verification:715 library tests pass (one opt-in ignored); client-feature checks
pass745 library,10 CLI and1 slice tests (six opt-in ignored). Library/test Clippy
with warnings denied, formatting, whitespace and task-command dry-run checks pass.

Remaining full-goal gates: verify complete neutral pose words and compose
height/reset/legs/tail/torso/heads/arms/wings in native order; generate all body
parts independently of the viewer; preserve one-time object projection and
skin normals/positions; integrate the textured full-body command; and inspect
multi-angle GPU captures for two supported fixture anatomies. Full-body preview
and the complete avatar goal remain unfinished.

## Neutral skeleton and reusable whole-body CPU generation

The native `AuPose::Name` table identifies0x24 as **Stand**: QString slot
stack0x40+0x24*8=0x160 receives the source string at142304911. Focused receiver,
slot-write and indexed-consumer disassembly is retained in
`native-neutral-pose-name-gnu.txt`. `tests/native_neutral_pose.py` pins the
string bytes and both leg/arm dispatch-table entries to the digest-checked PE.
It also checks the retained full AnimatePose disassembly and saved REA records:

- `native-dp-animate-pose.json`, ev_b2f6d8549fe666f3b3d70800cc0d20441ff51abfd74f026b6cf5f752a91ae03e.
- `native-dp-position-legs.json`, ev_2519dd7344a1badc224e6107dd2daf6acaefcb4edfcf9b8bb8abaa2783f3c74d.
- `native-dp-position-arms.json`, ev_17f36040ffbb5a56c997011ed3faac068c5298cb20355ac571cdd958fd71f315.

`pose_neutral_standing` now selects ten explicit0x24 words, mode=false,
speed=0 and neutral owner pitch30/roll34=0. Those owner values are fixture
inputs, not inferred production receive state. It computes CGHeight(false),
selectively resets active matrices, poses later leg pairs then the first pair,
poses the optional tail, torso and heads, then runs left and right arm passes.
No guessed body-height translation or generic parent-bone traversal is added.
All supported branches are non-rhythmic; a rejecting timer makes an accidental
timed branch an error rather than substituting a made-up clock. Winged and
legless anatomy are explicitly unsupported by this neutral composer pending
StandWings/tail-rhythm dependencies. Unsupported anatomy is rejected before
matrix writes. General animation, mode=true arm overrides and arbitrary owner
pitch/roll remain separate from this deterministic offline standing API.

Mounted tests compare the composition with its individual native stages for two
anatomies and prove repeat posing does not accumulate transforms. The second
fixture has two heads, two arm pairs, two leg pairs, a longer tail and changed
size/thigh proportions. Native planning emits one shared Neck22 mesh for that
two-headed anatomy, not two manually duplicated necks.

`NativeDpAvatarGeometry::generate` is a viewer-independent mounted CPU pipeline:
explicit DNA/appearance -> OneTimeCalcs -> ordered native skin plans -> decoded
resources -> head/torso merge stages or separate ordinary surfaces -> Stand
skeleton. It retains original decoded source meshes, native object state,
raw weight payloads, UVs, face-material memberships and skin controls. Normal
atlas selection/offsets accompany the generated body. Nonzero armor/helmet
appearance fields, crest merges and unsupported neutral anatomy fail explicitly.
This is normal-skin generation, not production cache lifecycle emulation.

Merged head/torso objects use their one-time baking and original seed matrix
projection. Ordinary arm/hand/leg/foot/tail/neck objects use the native unmerged
path: triangle-builder projection BEFORE per-vertex deformation. The new
`textured_unmerged_preview_geometry` adapter preserves that distinction and
shares UV/material/skin-control publication with the baked adapter. A CPU test
checks matrix/definition sequencing, untouched payloads and rejection of an
already baked object; it does not force merge baking onto limbs.

Mounted generation produces these complete model-space upload inputs:

| Fixture | Native parts | Surfaces | Vertices | Faces |
| --- | ---: | ---: | ---: | ---: |
| Existing head/torso DNA | 7 | 12 | 3602 | 5604 |
| Two-headed, additional limb pairs | 12 | 21 | 6661 | 10316 |

The test verifies every allocated part has a generated surface, consistent
UV/weight/control counts, valid faces, finite positions and successful shader
skinning of every face-referenced vertex using the posed skeleton. Unreferenced
helper vertices/raw payloads remain retained; they are not assigned invented
weights. The temporary test normal is only a bone-loop validity probe, not
evidence of final smoothed-normal correctness. Run `just test-native-avatar`.

These surfaces are deliberately still MODEL-space inputs alongside posed bones.
The next required stage is render-corner smoothing followed by position/normal
skinning exactly once, then full-body camera/material integration and actual
multi-view GPU captures. No final visual assembly, neutral ground placement,
up/forward convention or `preview-avatar body` availability is claimed yet.

Verification: 718 default library tests pass (one ignored), 748 client library
tests pass (five ignored), and 735 viewer library tests pass (three ignored).
The client CLI tests, four opt-in mounted standing tests, both mounted avatar
fixtures, static neutral-pose evidence check, formatting and warning-denying
library/test Clippy checks pass. These are CPU/regression checks, not GPU visual
acceptance of the assembled avatar.

## Reusable smoothed, skinned render corners

`model::crease_corner_normals` now owns the existing modern preview smoothing
algorithm without a Bevy dependency. The part viewer delegates to it, retaining
strict angle thresholds, exact-position seam adjacency, area weighting,
disconnected-fan separation and hard boundary/non-manifold edges. Degenerate
faces retain the explicit diagnostic +Z normal policy; this is not evidence of
native normal-generation parity. Invalid indices, nonfinite positions and
invalid angles return errors from the reusable API.

`NativeDpAvatarGeometry::posed_surfaces` computes those corner normals on each
complete MODEL-space surface before material partitioning, then uses the native
four-influence shader loop on each position and normal exactly once. Output
retains face-major ordering, source vertex references, UVs, skin controls and all
material memberships. Position output remains homogeneous `[x,y,z,w]`, including
non-unit W; no weights, original surfaces or native payloads are rewritten.
No attachment offsets, extra projection or parent hierarchy are introduced.

A synthetic fixture checks rotated smooth/hard normals, translation exclusion,
non-unit W preservation and unchanged source payloads across a material boundary.
Both mounted anatomies verify every render corner against the shared smoothing
and native skinning stages, including unit normals and exact UV/control routing.
The remaining renderer adapter must preserve the homogeneous result's meaning
when framing/uploading it. This milestone does not yet add the body viewer,
establish native camera axes or satisfy multi-angle GPU/visual acceptance.

Verification after this extraction: 721 default, 738 viewer and 751 client
library tests pass (one, three and five ignored respectively); the existing
client binary tests and both mounted avatar fixtures pass. Formatting,
diff whitespace checks and warning-denying default/viewer library/test Clippy
checks pass. No full-body GPU launch or capture is claimed.
