# Native rotation getter input stage

Static GNU disassembly of the patched target/digest pinned in
[native transforms](native-entity-transforms.md):
`AuMatrix4x4::GetRotation` at `0x1418fa0d0..0x1418fa2b8` constructs two
projected direction vectors and passes them to `AuPoint::CalculateRotation`
at `0x1418fa26e` (target `0x14164c1b0`). That final angle calculation is not
ported here. No original-client execution or scalar-angle parity is claimed.

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
or modify position/orientation state. The caller still needs the native final
`CalculateRotation` algorithm and SetPosAndOrient integration.

`dump-atom-rotation-vectors PROFILE HEX_RECORD` accepts the same bounded packed
record as `dump-atom-matrix`, constructs its fresh matrix, and exposes these
inputs offline. `just test-native-transforms` pins getter/call/constant anchors,
compares 768 input-stage fixtures on independently verified affine matrices,
and checks large-coordinate cancellation. Unit tests additionally cover
nonaffine denominators, exact zero/one handling and finite safety. These are
static and synthetic mathematical checks, not an original runtime oracle.

The next focused REA/Ghidra query is
`_ZN7AuPoint17CalculateRotationERKS_S1_` at `0x14164c1b0`, with complete output
selected as `target/native-calculate-rotation.json`. It is pending, not
validated evidence. This is the actual two-vector callee used by GetRotation;
the one-vector overload is a different entry and is not interchangeable.
