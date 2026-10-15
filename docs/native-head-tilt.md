# Native Person head tilt

Person's constructor-confirmed address point14248c5b0 has virtual+1e0 pointing
to140988990, `DaCreature::TiltHeadTo(float)`. It is not an unnamed scale/growth
setter. Complete independent GNU evidence and helper/constant bytes are retained
in [native-head-tilt-gnu.txt](evidence/native-head-tilt-gnu.txt).
`just test-native-head-tilt` pins that binding and operation anchors to the
selected PE digest. This is static shipped-artifact evidence, not execution.

`NativeHeadTiltState::apply` implements the verified body angle/timer gates.
General pose39 clamps positive requests to positive zero; pose0f clamps negative
requests to positive zero. Comparison with the stored vertical angle occurs
before body limits: signed zeros compare equal, NaNs enter the change path and
repeated out-of-range requests still enter it. The actual movement-speed modifier
is required before mutation, including on paths that later suppress animation.
Both model branches clamp vertical to the exact PE float words3fc90fdb/bfc90fdb
and set horizontal to positive zero. SSE second-operand NaN/signed-zero behavior
is preserved rather than using Rust min/max NaN normalization. Creature timer5e8
then restarts; it must remain Creature-owned across model switches.

AuPose::IsLocked is the exact unsigned predicate:3,5..15,57..73,76..135.
Locked poses, general4a/4b and special part-word17 gates suppress animation,
after angle/timer mutation. Dp uses part indices1/6/8; Animalia uses3. Immediate
Creature/Vehicle parents also suppress animation. Parent checks short-circuit
in native order. Otherwise Dp animation receives actual pointer690 presence and
the speed modifier; Animalia animation receives a separately started elapsed
timer. Both then call real SetInMotion. All these runtime effects are required
operations, not default successful no-ops. Earlier writes remain after Rust
errors; Qt locking, FP exceptions and concurrent model replacement are not
emulated.

Tests cover the lock predicate, each part gate, angle limits/NaNs/signed zero,
pre-clamp equality, pose constraints, parent gates, callback failure order and
partial state retention. Complete native Person packet fixtures now compose the
actual head-angle function through Creature's scalar stage in all float profiles.
Their speed/parent/animation observations are explicitly synthetic; this is not
production skeletal rendering. Actual movement-speed dependencies and model
animation, the production Creature context and client presentation remain open.
