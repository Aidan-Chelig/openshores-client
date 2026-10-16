# Native avatar locomotion checkpoint

Status: leg/arm Walk/Run consumers and normal uniform full-body composition
implemented, not yet connected animation. The active [animation goal](in-game-avatar-animation-goal.md) remains
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
