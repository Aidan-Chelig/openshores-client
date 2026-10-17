# Native avatar locomotion checkpoint

Status: leg/arm Stand/Walk/Run and admitted mixed received channels now animate
the persistent connected avatar. The active [animation goal](in-game-avatar-animation-goal.md) remains
open. No original executable/DLL has been run.

## Target and observations

Target `reference/openshores-full/OpenShores.exe`, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
Retained REA PositionLegs/AnimatePose evidence identifies the consumers; bounded
GNU disassembly supplies their concrete arithmetic without a new Ghidra import:

| Function | Observed code range |
| --- | --- |
| DpBody::WalkLegs | 0x1412f0bf0–0x1412f0d29 |
| DpBody::RunLegs | entry 0x1412f02c0; same selected trailing-leg traversal |
| DpLegAndFoot::Walk | 0x1412dac60–0x1412daf57 |
| DpLegAndFoot::Run | 0x1412da2b0–0x1412da653 |

Walk spreads at float bits 0x3db2b8c2; Run at 0x3c8efa35. Both traverse trailing
legs in reverse, stopping before the torso. Selector 0 means nonzero pair index,
1 means zero, 2 means all. This traversal differs from the earlier StandLegs
sentinel and is implemented separately.

Part consumers use concrete IsLeft vtable slot 0xa8, part DNA and pair index for
phase. Walk uses Rhythm pose 0x26; Run uses 0x25. Each samples DhDNA rhythm,
calculates standing joint angles, then applies hip, knee, conditional ankle and
splay in that order. There is no fabricated whole-body translation.

Let r be rhythm and s be stored splay. The hip adds (-3 - 10*r) degrees for Walk
or (-3 - 40*r) for Run. The degree constant is the promoted float literal, not
a newly computed pi/180. Dampening is sin(abs(s))*0.8, narrowed back to float.
Walk's knee amplitude uses standing knee/2, Run uses (standing knee + float pi)/6;
both multiply by (1 - dampening), then reduce the knee by amplitude*(1-r).
Ankle stays at its standing solution. Splay receives rhythm times dampening times
the reflected splay, divided by three for Walk or multiplied by half for Run.
The implementation retains observed intermediate float/double narrowing order.

Walk left/right phases differ by pi, with separate two-/three-pair literals.
Two-pair Run synchronizes left/right sides of each pair, with pi between pairs.
The one-pair left Run path additionally checks arm availability and WalksOnHands;
hand-walking selects zero instead of pi. The Rust helper retains this branch,
not a universal alternating-limb assumption.

## Verification and remaining boundaries

`python tests/native_leg_locomotion.py` validates the target SHA, literal bytes,
rhythm IDs, concrete side vtables, selector dispatch and joint call order.
These static checks do not establish original runtime or visual parity.
`mounted_walk_run_leg_samples_change_selected_bones_without_accumulation` passes
for one/two/three leg pairs, both gaits, all three selectors, and time samples
0/125/0 ms. It checks exact timer call counts, finite matrices, untouched
non-selected parts/bones, changed animated samples and deterministic repeats.

`locomote_legs` is a stage, not a complete AnimatePose implementation. Caller
must supply valid initialized measurements, current pose height, speed and native
reference-time domain, and reset/compose the full skeleton correctly. It neither
maps the source server's opaque pose byte to a native enum nor fabricates a
clock/speed interpretation. Arm gait, full-body composition, idle/action dispatch,
live movement selection, animated eye and persistent mesh updates remain open.

## Arm and full-body checkpoint

Bounded complete symbol disassembly of DpArmAndHand::Walk (`0x1412d8620`)
and Run (`0x1412d8350`) establishes their phase/angle consumers. Body wrappers
WalkArms (`0x1412edc20`) and RunArms (`0x1412eda70`) scan ordinary arms forward,
select by left/right side (not leg pair), and consume the optional owner-field30
override only once. `native_arm_locomotion.py` SHA-binds the literals, rhythm
IDs, override clearing and call order.

Arm phase uses zero on the left and pi on the right, with count/index-specific
two-/three-pair literals. Walk adds (5 - rhythm*10) degrees to standing shoulder;
Run adds (10 - rhythm*20). Walk elbow subtracts half the float sum
rhythm*elbow + elbow. Run subtracts that entire float sum, then, if not walking
on hands, adds one third of the float-pi-minus-elbow difference in double.
The optional first-arm override instead uses owner pitch minus shoulder and
wrist float bits 0xbd0efa35. Application order is shoulder, wrist, elbow, stored
splay and stored lift; existing native joint methods preserve this order.

Body Walk spreads arms at float bits 0x3e32b8c2; its lift is 0x3d0efa35 for
hand-walking or 0x3e32b8c2 otherwise. Run uses spread 0x3db2b8c2 and lift
0x3c8efa35/0x3e32b8c2 respectively. These are literal inputs to the existing
SpreadArms port, not a new per-frame shoulder offset.

`pose_locomotion` composes CGHeight(false), selected bone reset, both leg
selectors, tail, torso, heads and both arm sides in retained AnimatePose order.
It is explicitly normal unguided uniform Walk/Run with owner pitch/roll zero,
not arbitrary received action dispatch or production wing/guided-mode support.
It reuses the generator's initialized anatomy admission checks.

The mounted received-server-DNA test samples each gait at 0/125/0 ms on the
SAME body instance, skins every retained repaired batch, checks changed samples
and exact repeats, then restores Stand and compares every bone with the original
generated Stand. It passes, alongside 782 client-library tests (six ignored),
the arm static verifier, formatting/diff checks and warnings-denied client Clippy.
No original runtime was executed and no connected animated GPU output is claimed.

The paragraph above records the historical full-body checkpoint, not current
connected coverage; see the animation goal for subsequent GPU/camera/lifecycle
acceptance.

## Face-down dead limb checkpoint

Retained PositionLegs evidence
`ev_2519dd7344a1badc224e6107dd2daf6acaefcb4edfcf9b8bb8abaa2783f3c74d`
and `docs/evidence/native-dp-position-arms.json` dispatch native 0x12/0x15 to
DeadLegsFacingDown/DeadArmsFacingDown. This agrees with the supplied source
server's Dying/Dead codes, but does not establish arbitrary protocol equivalence.

Bounded complete symbol disassembly of `DpBody::DeadLegsFacingDown`
(`0x1412f00b0`–`0x1412f0295`) and `DpBody::DeadArmsFacingDown`
(`0x1412ebef0`–`0x1412ec086`) establishes the newly ported isolated stages.
`tests/native_dead_pose.py` binds the original SHA, literals, selectors, traversal
and joint calls. No original code executes.

Legs spread at float bits bc8efa35 and traverse trailing legs in reverse,
selecting later/first/all pairs like Walk/Run. Hip is bdb2b8c2 except when part
DNA byte 12 masked by 0x0c equals 4 AND stored signed splay is strictly below
float 0x3f490fdb: then hip is c0437a15. Knee is be32b8c2; conditional ankle bend
is 3f32b8c3; stored splay is applied last. This compares signed splay, not its
absolute value. Arms spread at 3edf66f3 with zero lift, scan ordinary arms
forward by side, then rotate shoulder c04b4bc3, elbow 3d0efa35 and stored splay.
There is deliberately no wrist rotation or LiftShoulder call in this stage.

CPU tests cover strict threshold/DNA-mask behavior and all arm side selectors,
finite changed selected bones, untouched others and deterministic reset/repeat.
Subsequent composition reuses the existing SHA-bound height, tail, torso and
head dispatch: uniform 0x12/0x15 use the height default (nonnegative half of the
measured dimension), zero torso-angle default, face-down tail and -half-pi head.
The caller still resets selected bones before applying stages in native order.
No world-space corpse offset or guessed fall transition is introduced.

Mounted received-DNA coverage now passes both full-body codes with cached-only
hand/foot dependencies: changed geometry and eye, finite vertices, identical
samples at 0/500 ms, and exact subsequent Stand restoration. Preparation warms
these actions before discarding mount access. The source adapter admits 0x12
and 0x15 explicitly, preserving mixed channels and giving uniform requests a
named HUD status. The connected fixture now requests Dying then Dead, checks
their fixed repeated meshes, and captures both before unsupported 0x13 and
Stand recovery. That connected run passes; both death images under
`target/connected-avatar-uittt3kf` were opened and show the textured body lying
face-down with unchanged render resources. The root deliberately remains at its
received standing elevation, so this fixture hovers above terrain: animation does
not implement death gravity/settling or invent a corpse placement correction.
No timed dying transition or broader death-direction/action catalog is claimed.

Offset correction: the first death-stage port mistakenly used packed byte 20
for part+0x24. DhDNA is embedded at part+0x10 with an eight-byte object header,
so that load is packed byte 12. The corrected implementation and CPU test now
explicitly distinguish bytes 12 and 20. Prior death screenshots precede this
correction and are not visual proof of the corrected branch. Mounted full-body
repeat/finite/Stand-recovery checks pass again; connected captures need rerunning.

## Wave arm stage (not admitted in-game yet)

Retained PositionArms evidence
`ev_17f36040ffbb5a56c997011ed3faac068c5298cb20355ac571cdd958fd71f315`
dispatches arm channel 0x30 to WaveArms. Bounded complete symbol disassembly of
WaveArms (`0x1412ed090`–`0x1412ed1f7`) and DpArmAndHand::Wave
(`0x1412d7140`–`0x1412d7340`) supplies the ported stage. Static anchors in
`tests/native_wave_pose.py` bind the same target SHA and literal bytes.

WaveArms spreads at float bits bfbde44e with lift be860a92, scans ordinary arms
forward by side, and passes actual owner pitch30 to every selected arm. Unlike
Walk/Run it has no left/right phase difference: two pairs use pi for index1;
three pairs use 2pi/3 and 4pi/3 for indices1/2. Other phases are zero.
Wave samples DhDNA rhythm pose **1**, speed1. For rhythm r and float degree d,
shoulder=(85+r)*d+ownerPitch; elbow=(15+10*r)*d; lift=storedLift+(10*r)*d.
Wrist-bending DNA uses r*float(10degrees)-float(6degrees); otherwise wrist is
float(-2degrees). Joint order is shoulder, wrist, elbow, stored splay, lift.
The Rust stage preserves observed f32 operation order.

Static anchors and pure phase/angle tests pass; 788 client-library tests pass
(six ignored). This is not full Wave composition or in-game action support:
leg dispatch, mounted stage sampling and connected acceptance remain required.

Wave composition checkpoint: 0x30 is now admitted on the two arm channels
(indices2/5), with every other channel preserved. It does not reinterpret a
uniform all-0x30 request as a standing wave: non-arm Wave channels are explicitly
unsupported until their dispatch semantics are implemented. Both the native
compositor and source adapter enforce that boundary before skeleton writes.
Mounted default received-DNA tests pass left-only, right-only and both-arm Wave
samples at 0/125/0 ms with changing phases, exact repeats, unchanged eye and
subsequent exact Stand restoration. Worker preparation warms both Wave arms
before caches become mount-free. The connected fixture now requests a left-arm
Wave over Stand, captures two phases and checks no eye/root/resource changes or
outgoing reports overriding the request. That connected run passes; Wave-A/B
and the corrected Dead capture in `target/connected-avatar-fg7sv59n` were opened.
The arm is raised with visibly different hand/wrist phases, while the other arm,
eye and standing body stay stable. The corrected death body remains face-down
at the unchanged received elevation. Exact Stand recovery, persistent assets
and source-decoder non-overwrite checks all pass. The offline combined suite
passes nine static anchors, 788 library tests and both mounted tests.

## Jump leg stage (not admitted in-game)

Retained PositionLegs dispatch maps 0x1c to JumpLegs. Bounded complete symbol
disassembly of JumpLegs (`0x1412efdd0`–`0x1412eff09`) and LegAndFoot::Jump
(`0x1412d9880`–`0x1412d99f9`) establishes the new isolated stage.
`tests/native_jump_pose.py` binds the original SHA, float literals, pair selectors,
rhythm ID, scalar widths and joint order. No original executable was run.

The body spreads at float 0x3e32b8c2, scans trailing legs in reverse, and selects
later/first/all pairs. Each selected leg samples rhythm0x1c, phase0, caller speed,
then obtains the real measured standing angles for caller height. With rhythm r,
standing angle s and crouch angle c, each joint uses f32
`(r*(s-c)+(s-c))*0.5+c`, retaining the observed operation order.
Crouch hip/knee are float bits c016cbe4/be860a92. Ankle crouch is 3eb2b8c2
when HasAnkleBend, otherwise bd0efa35. Application is hip, knee, **unconditional
ankle**, stored splay; reusing Stand's conditional ankle application would be
incorrect. No root/world translation is introduced.

The pure angle test covers rhythm extrema/midpoint and both ankle branches.
The mounted limb matrix now includes Jump alongside Walk/Run across one/two/three
leg pairs, all pair selectors and both ankle branches; changed phases, exact
repeats, finite bones and untouched unselected bones all pass. Jump arm/body
composition and connected acceptance remain open; runtime source0x1c still fails
explicitly rather than rendering a partial jump.

Arm dependency observation (not ported): bounded ArmAndHand::Jump
(`0x1412d8190`–`0x1412d8342`) also samples rhythm0x1c with phase0 and caller speed.
Shoulder interpolates from float(-5degrees) to measured Stand; elbow from
float(10degrees); wrist from float(10degrees) or float(-2degrees) according to
wrist-bend DNA. An explicit override instead uses ownerPitch-animatedShoulder
for elbow and float(-2degrees) wrist. Application is shoulder/wrist/elbow/splay/
stored lift. The body JumpArms wrapper, spread and override consumption still
need checking before implementation; these observations alone do not admit it.

Jump arm/full-body checkpoint: bounded JumpArms disassembly
(`0x1412ed8e0`–`0x1412eda62`) establishes spread float(-2degrees), lift
float(5degrees), ordinary-arm forward/side traversal and an override consumed
only by the first selected arm. The wrapper and concrete arm interpolation are
now ported, using Relax's existing shoulder/wrist/elbow/splay/lift path and its
override against the animated shoulder. Static jump anchors include both arm
routines and verify override clearing, owner pitch and f32 operation widths.

The internal compositor now supports 0x1c in native stage order, using existing
CGHeight, tail, torso and head dispatch. Worker preparation warms it before
freezing mount access. Mounted received-DNA tests sample full Jump at 0/125/0ms,
requiring changing surfaces, finite eye, exact repeat and later Stand restoration.
The multi-arm matrix includes Jump (72 combinations total across four arm
actions), and a separate three-pair test proves override affects only the first
selected arm for either/both sides and both wrist branches. All seven mounted
limb tests pass. Source admission/connected Jump acceptance still remain open;
the runtime adapter continues to reject0x1c. No root motion or jump physics is
implied by this completed native composition dependency.

Received Jump acceptance: source0x1c is now explicitly admitted and preserved,
including real packet-ingestion coverage. The owned-server received-pose fixture
adds two Jump phases after Wave. The connected run passes with unchanged
root/mesh/material/atlas/UV identities, changed geometry and exact Stand/eye
recovery. Both Jump images under `target/connected-avatar-6sh8tgso` were opened
and show intact tucked/extended body phases. No local pose reports overwrite
the received action. This implements received jump posing, not keyboard jump
input, root displacement, native physics or landing synchronization.

## Folded-limb composition (not admitted in-game yet)

Retained PositionLegs/PositionArms dispatch selects Fold for numeric codes
0x18/0x19/0x1b/0x35. Their user-facing action names are not established here;
do not label all of them as the same sitting action. Bounded complete symbols
FoldLegs (`0x1412efc20`–`0x1412efdc0`) and FoldArms
(`0x1412eb990`–`0x1412ebbab`) establish the
ported stages. `tests/native_fold_pose.py` SHA-binds literals, traversal,
conditional ankle and first-arm override. These are static observations only.

Legs spread at float 0x3eb2b8c2 and use reverse trailing-leg/pair selection.
Hip/knee/conditional-ankle literals are beb2b8c2/c021f770/3fc90fdb; stored splay
is last. Arms spread AND lift at 3e32b8c2, scanning ordinary arms forward by side.
Normal shoulder/elbow are c0384e89/40384e88 with no wrist rotation. The first
selected override instead uses ownerPitch+float(40384e89) elbow and bd0efa35
wrist. The one-ULP distinction from normal elbow is deliberate. Stored splay
and lift finish the arm stage. Neither limb stage samples a rhythm.

The internal compositor routes all four codes through existing height/tail/
torso/head stages; those stages can still animate while the limbs stay folded.
Preparation warms each code before caches become mount-free. Mounted received
DNA tests pass changed-from-Stand surfaces, finite geometry/eyes, exact repeated
samples and later Stand restoration. A separate leg matrix covers one/two/three
pairs, all selectors and both ankle branches on one cached-only body, preserving
all unselected bones and requesting no clock for the leg-only request. The arm
unit test proves override changes only the first selected arm and rejects NaN
pitch before writes. Runtime source admission and connected captures remain open.

Received Fold acceptance: all four numeric codes are explicitly admitted by the
source adapter and tested through packet ingestion. The expanded owned-server
sequence passes both captures per code, changing torso phases for0x18/0x19/0x35
and fixed0x1b, unchanged root/mesh/material/atlas/UVs, non-overwriting client reports
and exact Stand/eye recovery. All four A images and Fold18-B in
`target/connected-avatar-1q9_rh52` were inspected: folded limbs and modest torso
motion are visible. Root elevation is deliberately unchanged, so the pose floats
above this fixture's terrain; no seat/root-placement correction is introduced.
User-facing action names, seating interaction and physical transitions remain
separate from these verified received numeric pose channels.

## Point arm stage (not admitted by source adapter yet)

Retained PositionArms evidence
`ev_17f36040ffbb5a56c997011ed3faac068c5298cb20355ac571cdd958fd71f315`
maps each arm's0x1f to PointArms with its side selector. Bounded complete symbol
disassembly of PointArms (entry`0x1412ebbb0`, last branch`0x1412ebd6e`) supplies
the concrete port, with SHA-bound anchors in `tests/native_point_pose.py`.

Spread/lift are float bits bc8efa35/3db2b8c2. Ordinary arms are scanned forward
and selected by side. Every selected arm rotates shoulder by ownerPitch minus
float3db2b8c2, then elbow by float3db2b8c2, followed by stored splay and lift.
There is no wrist rotation, rhythm or first-arm-only override. The port reuses
Relax with zero wrist, whose native joint helper returns without writes for0.
Nonfinite owner pitch is explicitly rejected before spreading or bone writes.

Internal composition admits0x1f only on arm slots2/5, retaining other channels.
The source adapter still rejects it pending connected verification. Worker
preparation warms both Point arms before removing mount references. Pure tests
prove pitch affects every selected arm and leaves all others untouched. Mounted
received-DNA tests cover either/both arms, fixed0/125/0-ms samples, unchanged eye,
non-arm rejection and exact Stand restoration. The multi-arm matrix now has90
combinations across five actions, with Point requesting no rhythm and remaining
fixed. The full offline suite passes12 static checks,792 library tests (six
ignored), two received-DNA tests and eight mounted limb tests. No original
runtime was executed; nonzero source head-tilt/pitch mapping remains unsupported.

Received Point acceptance: source0x1f is now explicitly accepted only on arm
slots2/5, with packet-ingestion coverage and non-arm rejection. The owned-server
sequence requests left-arm Point after Fold, retains the neutral other channels,
and passes fixed samples, unchanged eye/root, persistent GPU identities,
non-overwriting reports and exact Stand recovery. Both Point images under
`target/connected-avatar-2mag2wpy` were inspected: the selected arm is extended
and the rest of the body remains standing. This is arm posing at owner pitch0,
not recovered source aiming/pitch semantics, finger articulation or a targeting
interaction. First-person checks for the expanded sequence remain to be rerun.

Head-angle bridge checkpoint: the compositor now accepts established native
body30/34 angles, preserving zero-angle wrappers. PositionHeads receives both;
Point/Wave receive owner pitch, while ordinary unguided limb overrides remain
absent. The explicit source float-radian adapter, its limits and its distinction
from native Creature timer/parent/callback effects are documented in
`native-head-tilt.md`. Mounted tests hold head channels at a fixed override to
prove angle-dependent geometry changes come from Point/Wave arms, then restore
exact Stand/eye. The complete expanded first-person action sequence now passes
in `target/connected-avatar-prcnc1al`, including Fold/Point and sparse stationary
head tilt/reset, animated-eye tracking, unchanged mouse-look and hidden skull.
This does not add local targeting controls, finger articulation or physics.

## Received crouch stages

Bounded GNU disassembly of `DaCreature::IsCrouched` at`0x14097eae0` reads
DpBody's general word48 (or Animalia's word6c) and applies unsigned
`word -0x10 <=1`. Thus0x10/0x11 are native crouched states, not guessed Fold
aliases. `tests/native_crouch_pose.py` binds this predicate and four dispatch
tables to the selected PE SHA256. Both leg selectors map0x10 to the same block
as Stand0x24, and0x11 to Walk0x26. Both arm sides do likewise. The second leg
table uses an explicit pose-minus4 index; the others use the pose directly.
Tables are at1422bf404/1422bf534 (legs) and1422bf080/1422bf1b0 (arms).

This agrees with retained PositionLegs evidence
`ev_2519dd7344a1badc224e6107dd2daf6acaefcb4edfcf9b8bb8abaa2783f3c74d`
and PositionArms evidence
`ev_17f36040ffbb5a56c997011ed3faac068c5298cb20355ac571cdd958fd71f315`.
The compositor reuses actual Stand/Walk stage implementations, not new guessed
crouch joint angles. Walk's limb consumers still use rhythm0x26 internally;
torso/tail retain their own0x11 dispatch/rhythm behavior. Native CGHeight's
existing RelativeStanding branch handles both codes, including the exact
2/3 multiplier when owner DNA byte1 bit8 is set, otherwise1.5, and the existing
half-dimension lower bound. This is not a universal hard-coded crouch offset.

The explicit source adapter admits both received codes and preserves mixed
channels. Preparation warms both before discarding installation access. No new
outgoing crouch prediction is admitted by the local source-report inverse table.
Mounted received-DNA tests check changed geometry, fixed0x10 versus animated
0x11, deterministic0/125/0-ms samples and exact Stand/eye recovery. Existing
SHA-bound height, torso, head and tail checks still apply.

This checkpoint renders a received posture at the actual received root. It does
not add Caps Lock controls, local movement-rate changes, collision-height or CG
ground-placement transitions. The saved Controls page documents held Caps Lock
crouch, but binding it before those placement rules would hide a physical gap
behind an invented visual offset. The owned pose-only fixture keeps its root at
standing elevation deliberately; any resulting hover is not proof of native
crouch grounding or a reason to double-adjust a real received transform.
