# Native rotation getter and angle calculation

Static GNU disassembly of the patched target/digest pinned in
[native transforms](native-entity-transforms.md):
`AuMatrix4x4::GetRotation` at `0x1418fa0d0..0x1418fa2b8` constructs two
projected direction vectors and passes them to `AuPoint::CalculateRotation`
at `0x1418fa26e` (target `0x14164c1b0`). The complete pipeline is now ported
for the documented finite/safety domain below. This is static reconstruction,
not observed original-client execution or an original runtime parity oracle.

For column-major matrix M, the observed leading stage is:

```text
translation = [M[3][0], M[3][1], M[3][2]]
origin_w = ((M[0][3] + M[1][3]) + M[2][3]) + M[3][3]
origin = normalize(translation, origin_w)
for axis in [0, 1]:
  numerator = (0 + M[axis].xyz) + translation
  projected = normalize(numerator, M[axis][3] + M[3][3])
  direction[axis] = projected - origin
```

`normalize(point, w)` divides only when w is neither exactly 0 nor exactly 1.
The constant double 1.0 is at `0x141e6ba40`. The origin denominator is the
observed **sum of all four homogeneous row entries**, not a substituted
generic origin-point projection. Exact zero/one branches and operation order
matter. Under fresh affine transforms all denominators are 1, but adding then
subtracting large translations can round components away. A unit basis at
translation `[1e17; 3]` produces two zero vectors in this input stage;
returning raw matrix columns would not reproduce native arithmetic.

`native_rotation_vectors` under `src/atom_position/rotation/` ports only this
stage for finite matrices. Nonfinite inputs, overflow and nonfinite projected
results are explicit Rust safety rejections, not recovered native policy.
It does not normalize direction lengths, extract angles, invent a quaternion,
or modify position/orientation state. `native_rotation` now feeds these inputs
into the separately reconstructed `native_calculate_rotation` algorithm.

`dump-atom-rotation-vectors PROFILE HEX_RECORD` accepts the same bounded packed
record as `dump-atom-matrix`, constructs its fresh matrix, and exposes these
inputs offline. `just test-native-transforms` pins getter/call/constant anchors,
compares 768 input-stage fixtures on independently verified affine matrices,
and checks large-coordinate cancellation. Unit tests additionally cover
nonaffine denominators, exact zero/one handling and finite safety. These are
static and synthetic mathematical checks, not an original runtime oracle.

## Angle calculation

The focused REA/Ghidra query for `_ZN7AuPoint17CalculateRotationERKS_S1_`
completed in 648689 ms with Ghidra 12.1.2. Complete validated output is retained
in [native-calculate-rotation.json](evidence/native-calculate-rotation.json),
Evidence `ev_fc5cb165e10961e3a46c5c77c563b13e8cf9ab94494176286dfa82097971ffa3`.
This is the actual two-vector callee, not its one-vector overload. Recovered
register/phi variable names are not original source; special zero-primary
operand bindings and arithmetic grouping were checked against disassembly.

The algorithm normalizes supplied vectors individually, without Gram-Schmidt.
An exact zero secondary uses +Y except that normalized primary +Y selects -X
and primary -Y selects +X. A zero primary remains zero. The cross product
provides the up vector. Native exact-zero branches choose X/Y atan2 formulas;
the straight-down case selects Y=pi rather than a generic Euler convention.
The expanded inverse-X/Y rotation of the normalized primary gives Z.
Exact absolute sine/cosine values of one trigger the native zero-coefficient
shortcuts, including signed-zero products. X/Y are snapped only **after** Z
is calculated; each angle with absolute value <=1e-5 becomes positive zero.
The threshold's original double bytes are at `0x14232d158`.

Implementation is split into normalization/defaults, native trig arithmetic,
angle control flow and tests under `src/atom_position/rotation/calculation/`.
Nonfinite inputs and overflowing/underflowing nonzero norms are explicit Rust
safety rejections, not native rejection policy. Arbitrary underflowed norms
can reach native branches whose implied products differ from ordinary cross
products; those unsafe degenerate cases are not silently generalized here.

`dump-atom-rotation PROFILE HEX_RECORD` exposes the complete fresh-matrix
pipeline. `calculate-native-rotation HEX_6_BIG_ENDIAN_F64S` is a bounded raw
two-vector mathematical diagnostic, **not** a native scene wire record.
`just test-native-transforms` also runs 311 independent canonical/random/
near-threshold mathematical cases, explicit scalar branch choices, norm
safety and malformed command fixtures. The reduced mathematical oracle
allows equivalent +/-pi aliases; important native scalar choices are pinned
separately. These tests do not execute the patched client.

`set_fresh_position_and_orientation_by_ether_at` now uses this getter pipeline
to populate the separate native orientation scalars, then restarts its timer.
Position-only setters retain those scalars; a received transform is not assumed
to be its extracted orientation. Unknown initial scalars remain None, not
guessed constructor defaults. Caller-owned TickTock, hierarchy index changes
and later cache invalidation remain outstanding. No playable client protocol
switch or native hierarchy completion is claimed.
