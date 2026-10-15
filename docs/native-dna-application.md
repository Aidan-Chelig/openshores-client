# Creature SetDNA model application

`native_apply_creature_dna` implements the outer DaCreature::SetDNA operation,
separate from DNA wire reception. The saved REA record is
[native-creature-setdna.json](evidence/native-creature-setdna.json), Evidence
`ev_7edc9be594fc13465f92fcf5b9c4e4a67d79d4a115439f4a1a9561a771fbaec5`.
Complete independent GNU disassembly, including the DhDNA exception/cast branch,
is retained in [native-creature-setdna-gnu.txt](evidence/native-creature-setdna-gnu.txt).
These are static observations, not original source or executed behavior.
`just test-native-dna-application` checks the selected target digest, branches,
helper/call anchors and actual PE bytes of the float1 update scale.

The prior metadata comes from the current Animalia first, otherwise DpBody,
otherwise chromatid0/handedness1/HP0. Dp chromatid is full byte64 !=1, not a
low-bit test; handedness is byte3a XOR1, retaining other bits. Animalia's raw
32-bit metadata is not normalized to guessed enums. A consistent actual model
snapshot is required; this is not a fabricated packet-default record.

AuDNA updates existing Animalia or constructs it with prior chromatid/handedness
and growth-stage parameter (old494==0). The saved general pose comes from its
word6c, otherwise Dp word48, otherwise standing24. HP is restored with lower
clamp-30, Dp is destroyed if present, timer5c8 restarts, InitPoses feeds actual
Animalia::Pose(false,1), then timer630 restarts. The real nested model/animation
effects are mandatory context operations.

DhDNA updates existing Dp with float scale1 or constructs it with first char
parameter (chromatid==0) and second bool parameter (handedness==0). Those names
deliberately avoid inventing semantic enums. HP is restored with lower clamp-30,
Animalia is destroyed if present and timer5c8 restarts. This branch does not
perform the Animalia pose reset or timer630 restart. Typed NativeDna variants
replace the native dynamic casts; unknown base subclasses are not fabricated.

Rust errors stop at the failing required operation and preserve earlier effects;
C++ exception recovery and Qt locking/failed-call parity are not emulated.
Tests cover all65536 Dp configuration byte pairs, both/neither model presence,
body priority, reuse/creation, growth bytes, HP extremes, pose initialization,
timer order and rejection at every operation. Their model context records
calls only: actual nested morphology/animation, production Creature context and
minimal-client presentation remain unfinished. This implements required outer
application ordering, not a successful no-op replacement for those effects.
