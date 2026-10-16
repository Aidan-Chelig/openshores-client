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

## Connected north-pole fixture verification

The controlled hypothesis above is now tested by `just test-connected-avatar`.
The harness obtains the clearance from the Rust received-DNA pipeline and seeds
the disposable server at the north-pole terrain radius plus that clearance.
After the first capture it updates the owned account to five units higher and
starts a second authenticated session. The reference server source is unchanged.

Both sessions verify received position and render-root position independently
against the seeded coordinates (0.01 native-unit tolerance for f32 wire/display
roundoff). An inspection-only gold ruler uses one world unit per tick, with its
base at the radial intersection of the actual terrain mesh. At the north-pole
vertex that mesh height agrees with the analytic fixture height. A 0.02-unit
line lift avoids z-fighting; it changes no body or terrain position. The camera
includes the ground reference in its view rather than recentering the body alone.

The inspected captures in `target/connected-avatar-1ffbaeup/` show nominal landed
feet at the ground cross, retaining the small native sole residual, and airborne
feet roughly five ticks above it. Body height is roughly six native world units,
consistent with its measured native Z extent rather than preview normalization.
Both cases pass origin checks; no shader or hierarchy errors were emitted.
This verifies this adult source-profile north-pole fixture, not all original
native environments or anatomy. Southern connected placement, appearance changes,
movement/turning and ordinary first-person-camera acceptance remain separate gates.

Mounted quick-dev/play spawns now use the same native nominal clearance instead
of the previous arbitrary +5. This only changes owned disposable server seed
data, never received coordinates inside the renderer. Other diagnostic workflows
retain their earlier fixture behavior.

## Southern connected frame verification

The negative-Z fixture now passes landed, five-unit airborne and live-turn/update
checks through `just test-connected-avatar-south`. Received model +Z must point
radially outward and the render quaternion must match the received rotation;
no additional gravity rotation is allowed. The live test verifies that bootstrap
frame before local prediction and retains mesh identities while turning.

The supplied selected-player bootstrap hard-codes zero rotation. The explicit
test-server wrapper supplies an outward orientation using its existing gravity
helper for the southern person record. This bounds the result to correct
source-profile received transforms; it does not prove unmodified selected-player
bootstrap at the south pole or original native environment parity. See the
[progress ledger](in-world-avatar-progress.md) for inspected captures and gates.
