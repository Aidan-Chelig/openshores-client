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
- `0x004`: read signed short hit points. DpBody clamps only the lower bound
  to -30; Animalia uses its SetHitPoints helper.
- Sample old console type (default `0x3d`) before pose reception.
- `0x008`: receive pose scalars in a fixed loop followed by a short. Pose
  application depends on body, station-pose handling, movement speed and state
  `0x690`; successful pose change or differing short can restart a timer and
  replace `0x5c0`/`0x492`. Exact loop shape and widths need assembly review.
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
must be checked against assembly before reproducing its fallback conditions.

## Remaining implementation dependencies

Unit and both DNA wire readers are implemented; remaining work includes
pose loop/side effects, console virtual bodies,
constructor defaults, precise scalar overloads and stream floating widths.
The existing source-server Person codec does not resolve these native layers.
Next inspect Unit, then reconstruct nested readers and concrete state in small
module folders; do not silently accept unresolved suffix bytes.

Animalia SetHitPoints is now independently reviewed in native-avatar-helpers.md:
it clamps the signed short at -30 and stores it, like the DpBody lower bound.
