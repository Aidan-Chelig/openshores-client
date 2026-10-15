# Assembled avatar: evidence and implementation ledger

Objective: [complete assembled avatar preview](assembled-avatar-goal.md).
Full-body generation, a standing pose and `preview-avatar body` remain unfinished.

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
