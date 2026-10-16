# Neutral avatar origin and ground clearance

Scope: adult, normal-skin, neutral Stand composition only. This is static native
evidence and mounted Rust measurements, not an observation of the original game
running or proof of connected landed acceptance.

Target: `reference/openshores-full/OpenShores.exe`, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
`just test-native-avatar-anchor` checks the digest, raw signed dispatch entries
and symbol-backed GNU disassembly. It executes neither the target nor a DLL.

## Static observations

- `DaCreature::CGOffset(DpBody const&, ...)`, `0x140978ac0`, reads general pose
  from body offset `0x48`. Stand `0x24` selects table `0x1421c5e34` entry
  `0x24-3`, target `0x140978b10`, which writes three zero doubles. Stand does
  not request a head-location or other nonzero CG offset.
- `DaCreature::BodyPoseAdjustTransform`, `0x140980830`, reads the same general
  pose. Table `0x1421c664c`, entry `0x24-4`, selects `0x1409808e7`, the return
  epilogue. This Stand branch applies no extra pose rotation or translation.
  `RenderEnvironmentObject` calls it at `0x1409a17ca`.
- `DaCreature::GroundHeightAboveCG`, `0x14097c670`, special-cases poses `0x39`
  and `0x0f`; ordinary Stand instead tail-calls `CGHeight()` at `0x14097c6d4`.
- The active-body, no-growth branch in `DaCreature::CGHeight()` requests
  `DpBody::CGHeight(true)` at `0x14097c57b`, calls `CGOffset` at
  `0x14097c595`, then subtracts its Z component. That component is zero for
  Stand. This is distinct from `AnimatePose`'s previously verified
  `CGHeight(false)` used to position bones.

The existing retained `native-dp-pose-height.json` record is REA Evidence
`ev_c744263a605d92e3c91bd0a3854a311a5900956ad1e6fed8c260dd1e63a46477`;
its pose-height implementation is documented in
[the native model plan](native-model-application-plan.md).
The additional caller/dispatch checks above are GNU static evidence, not new
REA decompiler records. The old local REA snapshot was rejected for an analysis
profile mismatch; no cold import was started to replace it.

## Mounted measurement and implementation

`NativeDpAvatarGeometry::standing_ground_clearance(mount)` reuses the existing
pose-height provider with `include=true`, and rejects a changed/non-neutral pose
context. It supplies nominal clearance metadata, not an offset to add to the
render root. Growth, other poses and vehicle-parent contexts are not inferred.

`just test-local-avatar-preparation` takes the supplied server's actual default
DNA through the receive adapter and measures:

| Quantity | Native model units |
| --- | ---: |
| Skeleton pose height, `CGHeight(false)` | 2.6520436 |
| Nominal ground clearance, `CGHeight(true)` | 2.957287 |
| Drawn minimum Z | -3.0233366 |
| Drawn maximum Z | 3.00644 |
| Minimum Z plus nominal clearance | -0.066049576 |

The body straddles its model origin; that origin is not the feet plane. The
native nominal clearance and authored/deformed sole envelope differ by about
0.066 units for this DNA. These measurements are diagnostics, not a reason to
move every avatar by its bounding-box minimum. No renderer translation, terrain
snap or native-unit rescaling has been added.

## Placement consequence and remaining proof

For a controlled nominally landed Stand fixture, test an authoritative body
origin at terrain radius plus the native ground clearance; test an otherwise
identical off-ground fixture higher by an explicit known distance. Render both
at their received coordinates. This is the next test hypothesis, not a completed
native wire-origin/terrain-contact proof. Check actual coarse-mesh terrain height
as well as analytic terrain: the two can differ on this diagnostic globe.

Remaining work must establish that the source-server bootstrap and correction
path preserve that origin convention, verify scale against world geometry and
inspect landed/off-ground GPU output. Preserve the small native sole residual
explicitly rather than silently applying a geometry-fit correction. The existing
first-person camera's fixed 1.7-unit offset and terrain-clearance clamp are
prototype camera policy, not evidence for a body-placement offset.
