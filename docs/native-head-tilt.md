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
production skeletal rendering. Actual native movement-speed dependencies, model
animation callbacks and the production Creature context remain open. The source
rendering bridge below is separate from those native runtime requirements.

## Source-profile rendering bridge

The supplied source writer `protocol/atoms/daanimal.py` writes received head tilt
as a big-endian float and its self-test expresses that float in radians (lines
111 and209–212). This is not the outgoing delta-tick i16 angle encoding.
The source-profile player adapter now accepts finite tilt for its existing
admitted normal-body channels, bounds it to the verified native body30 limits,
and supplies body34 as zero. Missing sparse fields retain their previous value;
missing initial tilt defaults to zero. Unsupported channels and nonzero pose
parameters still fail explicitly.

This is a **rendering bridge**, not an implementation of the native Creature
runtime effects above: it does not invent movement-speed dependencies, restart
Creature timers, bypass parent callbacks by faking success, or claim native
SetInMotion. Its admitted channel table excludes the special general39/0f and
part17 constraints. The source-profile presentation already samples an owned
normal body; native parent/guided/Animalia animation remains separate work.

`pose_channels_with_head_angles` consumes established native body30/34 angles
without applying Creature constraints. PositionHeads receives both angles;
Point/Wave receive pitch on each selected arm. Other unguided limb overrides
remain absent. The cached sampler rejects nonfinite angles before bone writes,
keeps mount access absent, and preserves exact zero-angle wrappers. The UI idle
fast path compares angles as well as standing state, so sparse stationary tilt
updates and zero resets update persistent geometry and the animated eye.
Camera mouse-look orientation remains independently controlled, unchanged.

Mounted tests cover positive/negative pitch, roll, deterministic repeats after
rejected inputs, exact Stand/eye restoration and Point/Wave pitch with head
channels held at a fixed native override (so head movement cannot satisfy the
arm assertion). Packet-ingestion tests cover combined pose+tilt, omission,
tilt-only changes and explicit zero reset.

Connected third-person acceptance passes in `target/connected-avatar-vd2ue1mk`:
the owned source-server fixture sends real framed Stand+tilt, then tilt-only
negative/reset updates. Persistent root/meshes/material/atlas/UVs remain the
same; the eye and vertices change on stationary tilt and restore exactly on
zero. Both tilt images were inspected: the head visibly looks up/down while
the rest of the body stays standing. No original EXE or public server was run.

First-person acceptance also passes in `target/connected-avatar-prcnc1al`:
stationary tilt updates follow the animated authored eye, preserve mouse-look
orientation, keep the skull hidden, and restore exact Stand/eye on reset. The
negative update explicitly differs from the previous positive eye position.
This runs the complete admitted action sequence, not just isolated tilt. The
test capture reuses the sequence's initial600-frame cold warmup instead of
repeating it after completion, preserving the unchanged60-second local runtime
bound. The earlier first-person attempt reached passing action assertions but
timed out after that redundant warmup; it is not counted as passing acceptance.
