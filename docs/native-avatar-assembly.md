# Assembled avatar: evidence and implementation ledger

## User-approved modern seam repair

After the native deformation/definition audits, the user explicitly chose a
render-only seam repair rather than preserving the gaps as a fidelity limitation.
This supersedes the earlier prohibition on corrective rendering, not the requirement
to retain native geometry and honestly separate modern choices from recovered code.

The complete triangle-contact diagnostic (vertex/face, segment/face and edge/edge)
finds separation already in the unposed model-space surfaces of both fixtures.
Every arm/hand and leg/foot source is closed after exact render-f32 position
canonicalization, including UV-duplicate seams. There are no open cuff loops.

`avatar_seams.rs` therefore derives a separate combined render surface for each
limb. The actual native wrist/ankle point anchors a modern end-cap heuristic:
choose the nearest face centroid whose outward normal has dot >0.8 with the
surface-center-to-joint direction, then take its connected normal neighborhood
(dot >0.5 with the seed). Require a single nonbranching perimeter and closed
source topology. These thresholds and centroid selection are modern heuristics,
not reverse-engineered native operations or general support for every DNA shape.

Remove only those cap faces in the derived copy and bridge the two directed
perimeters with a minimum-cost monotone zipper. Reverse the second perimeter,
anchor at the closest endpoint pair and restrict progress difference to less than
half a turn to prevent a cross-edge being reused four times. All original vertex
positions, raw four-influence weights, atlas UVs and texture controls are copied
unchanged; no parts are translated. Sleeve faces inherit a cap material membership
from their advancing side. New sleeve UV interpolation deliberately spans existing
endpoint UVs; it is not a newly recovered native unwrap. Smooth the combined model
before posing/material partitioning, so sleeve normals share crease rules.

`seam_repairs()` exposes source surface indices, the combined vertex split,
removed cap counts, bridge counts and derived geometry. The original source
generation and `posed_surfaces()` remain unchanged. The body viewer opts into
`posed_surfaces_with_seam_repair()` by default; `--native-seams` selects the original.
Source installations are never modified. Mounted tests exercise all 12 joins
across both fixtures, verify unchanged attributes and reject degenerate triangles,
open edges and inconsistent winding. The full-avatar goal still requires visual
acceptance; this repair does not address the torso's existing thin/open surfaces.

Verification: all 12 repaired joins also retain closed edges after actual native
bone skinning and homogeneous division. All 723 default library tests, 742 viewer
library tests and 11 viewer CLI tests pass (asset-dependent tests are separate).
Viewer/library/CLI and attachment-test Clippy passes with warnings denied.
The 11 GPU captures in ignored `target/avatar-textures-362sjyke/` pass nonblank
and shader-error checks and were all opened for visual inspection: both anatomies
from oblique/front/side/back, the base untextured comparison, and head/torso
regressions. Cuffs are derived without part translation; no new detached limb
pieces were observed. Tiny textured features and modern sleeve UV interpolation
are not a claim of native runtime visual parity. Torso openings remain visible
in textured and untextured views, so the complete-avatar goal remains active.

Objective: [complete assembled avatar preview](assembled-avatar-goal.md).
CPU full-body source generation, the Stand skeleton, posed render geometry and
`preview-avatar body` are implemented. Both anatomies have inspected GPU captures;
the remaining torso-surface issue keeps full visual acceptance unfinished. The
following audit sections retain the historical sequence of the reconstruction.

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

## Full-body GPU adapter and first visual audit

`just preview-avatar body` now invokes the reusable generated/posed surfaces,
not a hand-arranged model. Optional positional task arguments select installation,
crease angle, fixture (`base`/`many`) and view (`oblique`/`front`/`side`/`back`).
The underlying CLI adds `body`, `--fixture` and `--view` while retaining head,
torso and untextured diagnostics. Body untextured mode uses the SAME skinned
corners/normals with an explicit white one-pixel atlas. It does not recompute
normals on posed geometry. R resets to the selected view; ordinary orbit, zoom,
lighting and wire controls remain available.

The renderer frames only drawn, posed corners. No original source/native cache
state or part attachment is modified. CPU output retains homogeneous coordinates;
the upload uses xyz/W for Bevy's affine model/world position and stores the
original positive W in vertex color R (previously always white). The skin
fragment already replaces vertex color before lighting, so alpha still carries
only the native top/bottom texture control. A focused vertex shader multiplies
the projected clip coordinate by that W, restoring `P * View * Display * native`
including perspective-correct interpolation. Silently discarding W would produce
the same point locations but different interpolation when W varies. CPU tests
verify this equality under a nontrivial view/projection, UV/control routing,
preserved posed normals and explicit rejection of nonpositive/nonfinite W.
This positive-W renderer restriction does not change the reusable native bone
loop's accepted finite weights or homogeneous output.

For these neutral fixtures, coordinates stay native XYZ with no corrective axis
rotation. The server's `gameplay/natives/village.py::body_axes` names X right,
Y forward and Z up (see `native-entity-transforms.md`); that alone was insufficient
to assert authored model axes. The native Stand/TiltHeadTo composition plus the
new front (+Y camera), side (+X) and back (-Y) captures now corroborate those
axes for these POSED avatars: +Y shows the eyes/nose/mouth and -Y the backs of
the heads, while +Z is the upright head direction. This does not generalize to
every source mesh's pre-pose authored axes. Relevant retained records remain
`native-dp-head-tilt.json` (ev_0b59362eb4555ddf39adab5030c8118f943e673f530ebbc4dc1ba656fedd67eb),
`native-dp-position-heads.json` (ev_7dbc25c94fb3c6fc3e20ab4337be151ef1606282bf2028fd81dcd7848778ffa6),
and the already verified neutral-pose records; no fresh native import was needed.

`just test-avatar-body-previews` captures both anatomies in four views, a white
untextured body, and head/torso regression views. The first complete run passed
all eleven nonblank/color-variation checks, with no reported shader errors,
on AMD Radeon AI PRO R9700 / RADV GFX1201, Mesa26.2.4, Vulkan. Local ignored
captures: `target/avatar-textures-fh5jjd7j/` (not redistributable installation
assets). Both fixtures' four views, the untextured body and the two part views
were opened and visually inspected. Foreground pixel counts:

| Fixture | Oblique | Front | Side | Back |
| --- | ---: | ---: | ---: | ---: |
| base | 73853 | 111628 | 45219 | 37868 |
| many | 46895 | 47673 | 44110 | 43410 |

Untextured base:40798; head:160462; torso:64748. The native plan produces
12/21 complete surfaces, split into14/24 material partitions respectively.
The extra heads/limb pairs and changed proportions are visible without
fixture-specific attachment offsets. Texture-region controls remain CPU-checked
and visible across all parts, with modern lighting intentionally retained.

**Visual acceptance is still open.** Captures show small apparent gaps near
some hand/foot ends, and thin/open torso geometry already present in the part
preview. The mounted test now reports nearest drawn-vertex distances between
same-part surfaces as a diagnostic: base arms~0.0040/0.0045, legs~0.0064;
many arms~0.0048..0.0051, legs~0.0184 (native coordinate units). This is NOT
triangle distance, proof of a gap, a tolerated-error threshold or native runtime
parity. Next compare joint/control-point coordinates and source/deformation
stages against retained native evidence, then inspect close views. Do not repair
these with bounding-box offsets or declare the complete-avatar goal finished
merely because smoke/CPU tests pass. No in-world ground placement is claimed.

Verification: 740 viewer library tests pass (four ignored), 753 client library
tests pass (six ignored), 11 CLI tests pass (one mounted test ignored), and the
client slice test passes. The explicit mounted CPU avatar test and eleven GPU
captures pass; static neutral/head-pose anchors pass. Viewer library/test/binary
Clippy passes with warnings denied, as do formatting and diff whitespace checks.

## Attachment investigation: ruled-out causes, not a visual fix

`tests/native_avatar_generation.rs` now checks every arm wrist and leg ankle in
BOTH anatomies. The shared native joint projected through the adjacent proximal
and distal bone matrices agrees within1e-10, and the distal mesh definition's
float translation equals that initialized wrist/ankle point narrowed to f32.
This checks actual field70/68 points and actual Bones out-reference indices;
it does not infer offsets from geometry bounds. The checks pass without changing
any native pose, weights, source vertices or attachment translation.

For the base fixture, diagnostic shared-joint coordinates (Rust CPU observations,
not native runtime captures) are approximately:

| Joint | X | Y | Z |
| --- | ---: | ---: | ---: |
| Left wrist | -0.067475602 | 0.332920179 | -0.047483702 |
| Right wrist | 0.067475602 | 0.332920179 | -0.047483702 |
| Left ankle | -0.223940306 | -0.000000009 | -0.050134106 |
| Right ankle | 0.223940306 | -0.000000009 | -0.050134106 |

`just test-native-avatar-attachments` prints source control points, actual native
group pivots (not first member positions), uniform/radial/height factors,
drawn-only loaded/deformed ranges and vertex-to-triangle separation upper bounds.
It uses only mounted Rust decoding/generation; no original executable/DLL runs.
Its base-fixture vertex-to-triangle upper bounds are about0.00396746/0.00451679
for arms and0.00367080 for each leg, smaller than some earlier vertex-to-vertex
distances. This diagnostic is deliberately NOT a full triangle intersection or
minimum separation test and cannot by itself prove an actual surface gap.

A static cross-version check rules out a mismatched source-artifact hypothesis
for this fixture: all18 decoded source resource byte sequences are identical
between the user installation and `reference/openshores-full/libAuBodyPartsRes.dll`.
The optional comparison never substitutes those reference assets into generation.
Reproduce with `OPENSHORES_HAZERON_INSTALLATION=reference/hazeron
OPENSHORES_PATCHED_BODY_RESOURCES=reference/openshores-full/libAuBodyPartsRes.dll
cargo test --offline --test native_avatar_attachments -- --ignored --nocapture`
(on one shell line, inside the usual Nix environment). The reference DLL is
identity-guarded by SHA-1; its documented SHA-256 is
3f74065a683e033e9e9a75c49eb2499d2ffffae144a2642333e332d61d67e6de.
The retained `native-patched-body-resource-registration-gnu.txt` shows its
qInitResources0 passing ECX=3. The general user-installation mount is unchanged;
this optional static comparison uses the symbol-bearing version3 resource reader.

The full exact-symbol GNU disassembly of the body triangle builder is now
retained as `native-body-triangle-upload-gnu.txt`, alongside the existing REA
record `native-body-load-body-model.json`
(ev_0ad9fb10afe5f344430f9e7a2f8c50c4508527e505d2d3c8e1f4cd1150b664c0).
`tests/native_body_triangle_upload.py` verifies the selected original PE digest,
full retained disassembly, three object-matrix Project results passed to
ApplyScaleFactor, the field369 baked bypass and stored normal-body influence
upload. Thus the projected-before-deformed ordinary-part path is retained, not
changed to merge baking as a visual workaround. No new Ghidra import was needed.
The existing ApplyScaleFactor and model-definition records also retain the
anatomical scalar/connection correction operations; no missing operation has yet
been established from that review.

These findings rule out a simple shared-joint disconnect and this fixture's asset
version mismatch. They do NOT establish that the apparent gaps are native art
quirks or complete visual parity. The remaining investigation is source seam
contact and its native construction/deformation path; do not replace the fixture
or add corrective offsets to conceal an unresolved mismatch. The avatar goal
remains active and visual acceptance remains unmet.

Verification for this investigation: both mounted anatomy fixtures pass the
new shared-joint/definition checks; the base mounted diagnostic passes with and
without the optional reference comparison; its synthetic triangle-distance
cases pass. All721 default library tests pass (one ignored), the SHA-bound
triangle-upload static verifier passes, and default library/test Clippy with
warnings denied, formatting and diff whitespace checks pass. Rendering is
unchanged, so no replacement GPU captures or corrected visual result is claimed.
