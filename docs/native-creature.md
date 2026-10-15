# Native Creature reception

Target: `reference/openshores-full/OpenShores.exe`, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
Full reviewed static Ghidra 12.1.2 record: `evidence/native-creature-update.json`,
`ev_005d4333a675a1e1d4ef9f42991fe3da987b22ab01a80fcf8fd20cc5e1f8d463`.
Recovered types/virtual calls require corroboration; no runtime was observed.

## Ordered receive stages

Creature calls **DaUnit::RxUpdate** first, then receives a short flags word.
It is not directly the common atom base followed by Creature fields.

- Flag `0x100` selects Animalia storage (`0x4a0`) versus DpBody (`0x5b0`).
  Every update ensures the selected body exists and destroys the other one,
  even if no DNA or pose follows.
- `0x001`: read packed scalar; bit 0 and bit 2 affect body configuration.
  Receive `AuDNA` for Animalia or `DhDNA` for DpBody, then call SetDNA;
  Animalia additionally receives SetDNA/SetLeftHanded effects.
- `0x002`: read floating value and invoke virtual setter at vtable `0x1e0`.
  Person binds this to TiltHeadTo(float); see native-head-tilt.md for the
  verified head-angle/timer/animation operation and remaining runtime boundary.
- `0x004`: read signed short hit points. DpBody clamps only the lower bound
  to -30; Animalia uses its SetHitPoints helper.
- Sample old console type (default `0x3d`) before pose reception.
- `0x008`: receive pose scalars in a fixed loop followed by a short. Pose
  application depends on body, station-pose handling, movement speed and state
  `0x690`; successful pose change or differing short can restart a timer and
  replace `0x5c0`/`0x492`. Loop shape/width are now corroborated below.
- `0x010`: receive console type; an existing matching console pose may consume
  its virtual body. Otherwise, if the creature's virtual context is available,
  construct a temporary console, receive it and destroy it. Therefore console
  bodies cannot be assumed absent merely because no persistent console exists.
  Without `0x010`, an existing non-console-pose object is destroyed, while a
  console-pose object is retained. Matching-console updates can also load and
  restore client console settings.
- Unconditionally receive scalar at `0x496`, after console handling.
- `0x020`: receive scalar `0x494`; absent explicitly zeroes it.
- Unconditionally destroy and null item pointer `0x698`; `0x080` optionally
  reads type, creates an AuItem for nonzero type and invokes its virtual reader
  if creation succeeded. Omission does NOT retain the previous item.
- Sample wall-clock time into the selected body's timestamp. Flag `0x040`
  subtracts exactly 86,400,000 milliseconds; it consumes no additional field.
- `0x200`: receive two scalars, widened from their low bytes into `0x5b8` and
  `0x5bc`; omission retains those fields.
- Finally derive byte `0x495` from parent vehicle SeatAt(current location),
  or zero it when no vehicle parent exists. This is not another wire byte.

Temporary console factory cases observed: 1 Engineer, 2 Weapon, 3 Helm,
4 Surgery, 5 Nav, 7 Power, 8 Sensor, 9 Shield, 10 Transporter;
`0x0b` ScCommand, `0x0c` ScEngineer, `0x0d` ScWeapon, `0x0e` ScHelm,
`0x0f` ScSurgery, `0x10` ScNav, `0x12` ScPower, `0x13` ScSensor,
`0x14` ScShield, `0x15` ScTransporter; `0x21`–`0x23` ConsoleGame.
`0x11` explicitly skips construction. Other values enter diagnostics, not an
opaque-body skip. Decompiler control-flow around the matching-console result
is now checked against assembly in native-consoles.md; context lookup and
concrete console construction still need reconstruction.

## Remaining implementation dependencies

Unit and both DNA wire readers are implemented. The ordered receive stage
below now composes them; remaining work includes concrete pose/model effects,
console context/application, inherited construction and native object dispatch.
The existing source-server Person codec does not resolve these native layers.
Next finish pose/console application and nested readers in concrete state in small
module folders; do not silently accept unresolved suffix bytes.

Animalia SetHitPoints is now independently reviewed in native-avatar-helpers.md:
it clamps the signed short at -30 and stores it, like the DpBody lower bound.

## Pose wire block and console dependency map

Independent GNU record evidence/native-creature-pose-console-gnu.txt pins the
pose loop at 14099b190/14099b358: destination starts at stack60, advances four
bytes until stack88, while each input uses the byte overload. Thus ten bytes
widen unsigned into ten native words, followed by one raw short. It is not a
40-byte wire pose. Both Animalia and DpBody use this same 12-byte wire shape.
DpBody passes the station helper a pointer at pose+12 (three words in), whereas
Animalia passes pose+0; downstream application must preserve this distinction.
`NativeCreaturePose` implements only this atomic wire read. SetStationPose,
body Pose, movement-speed effects and timer updates are separate unresolved
application stages, not implied by reading the packet successfully.

See native-consoles.md for 20 constructor-confirmed primary-vptr bindings to 11 shared
station readers, their complete evidence IDs and resolved matching-console
control flow. All station wire families are now implemented; Sensor/Weapon/
Transporter local augmentation remains separate in native-station-sensors.md.

## Ordered Unit/Creature receive implementation

`NativeCreatureReceiveState::receive_qt7` calls the real `NativeUnitState`
reader before receiving the Creature short. It composes the real AuDNA/DhDNA,
pose and AuItem codecs in the native order above. It requires an explicit
`NativeCreatureReceiveContext` for every application effect; there is no
production default or silent no-op context. This is an ordered receive stage,
**not** complete Creature/Person object dispatch or presentation.

The context must implement model selection/lifecycle, pre-DNA DpBody
configuration, SetDNA effects, +1e0, hit points, old console type sampling,
station/body pose application, console reception/settings/local augmentation,
wall clock, body timestamp and parent SeatAt, or return Unsupported. Typed DNA
and pose values are supplied after their real wire readers; the context cannot
substitute an opaque skip. Console reception remains context-dependent because
matching/temporary/null-context branches consume different bodies. The pose
hook receives current receive state, including the previous growth byte, since
new494 is not read until after console handling.

The state deliberately models receive observations, not a guessed complete
DaCreature constructor. Default leaves Unit/body/scalars unobserved. Selection
ensures the requested model on every update; a changed kind replaces the
receive-visible body subset. Both default body constructors establish HP0 and
timestamp0. DpBody bytes3a/64 start0; reception of packed DNA configuration
changes them to bit2/inverse bit0 **before** reading DhDNA. Native model/bone
construction and dependent defaults still belong to a concrete context.

HP reception is signed-short with only the lower -30 clamp in both modes.
Float-overload +1e0 narrows to f32 for every wire profile, retaining raw
non-finite/signed-zero values. Item698 is always cleared after496/494; type0
consumes no body, verified subtypes use NativeItemValue, and unimplemented
factory/body types return Unsupported at that discriminator without guessed
consumption. No old item is retained on such an error. Body timestamp sampling
occurs after the item and before optional 5b8/5bc; flag40 subtracts 86400000 with
the observed two's-complement machine arithmetic. Optional5b8/5bc widen bytes
unsigned and otherwise retain their old values. SeatAt is last, not wire data.

Earlier stage mutations survive later Rust errors. Fresh nested DNA/item/pose
decoding keeps its documented atomic policy. Context failures abort immediately;
the receiver does not consume a guessed remainder or invoke later effects.
These policies do not claim Qt failed-stream continuation/destructor parity.

Fixtures compose actual Unit identity, Fire condition and vector, both DNA
bodies, a concrete Game console body and a real plain item. They cover all
65536 flags, all profiles/configuration bytes, repeated sparse updates/body
switches, every truncated prefix, failure at every required context effect,
unsupported item/console cases, all signed-short HP values, non-finite floats,
timestamp wrapping and exact suffix alignment. Their context is explicitly a
**trace fixture**: it proves order and codec composition, not skeletal/pose,
settings/environment effects or runtime avatar integration. The final goal
still needs a real context and full fresh/existing Person packet tests.

## Constructor and application evidence

Complete MCP records with Evidence IDs are indexed in
`evidence/native-creature-application-index.json`. New constructors and the
occupant helper reused the live analyzed session in0.085/0.085/0.088/0.104/0.079s,
with no reimport. Existing pose/SetDNA/movement records were retained from
saved results, not re-queried. Independent `native-creature-receive-gnu.txt`
retains complete constructor/update disassembly and corroborates overloads,
pre-DNA byte writes and the ordered tail. GNU selects C1 aliases at the same
addresses as the C2 constructors queried through REA.

| Constructor | Address | Established receive-visible facts |
| --- | --- | --- |
| AuAnimalia default | 0x1411ee400 | HP30=0, time38=0, pose words6c..90=0, bool40=0; float94=0 |
| DpBody default | 0x1412dc540 | HP38=0, time4010=0, config3a/64=0, pose words3c..60=0, bool6c=0 |
| DaCreature default | 0x140973ab0 | Unit first; initial DpBody and no Animalia; console680/item698 null; 5b8/5bc=4; 494/495/496=0; 492/5c0=0 |
| DaPerson default | 0x140a3f930 | Sentient first; full record retained for later inherited-state construction |

Do not extrapolate complete model/Person initialization from this subset.
SetDNA includes dynamic type dispatch, inherited HP/configuration, pose resets
and timers. Its outer application order and DhDNA catch/control flow are now
independently corroborated and implemented; see native-dna-application.md.
Actual nested model generation remains required. DpBody::Pose and Animalia::Pose records establish word
comparisons, different bool-change behavior and Animalia's final-pose fallback,
but their AnimatePose effects remain unimplemented. MovementSpeedMod's Ghidra
return type is incorrectly void; its actual float return/arithmetic are now
independently reconstructed from machine code; see native-movement.md.
OccPosForPose's 133-byte table and SetStationPose's relative jump table are now
retained and implemented, along with concrete body pose data/change rules and
fallback markers. See native-pose-application.md. Animation callbacks and station
requests explicitly still require real effects; no full runtime context is claimed.
