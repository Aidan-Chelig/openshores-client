# Assembled avatar preview: acceptance evidence

Scope: the original [complete assembled avatar goal](assembled-avatar-goal.md),
plus the user's explicit choice of modern render-only seam repair. This is an
offline fixture preview, not multiplayer integration or original-runtime pixel
parity. The [implementation ledger](native-avatar-assembly.md) retains the
reconstruction history, evidence IDs and deliberate native/modern boundary.

## Requirement-by-requirement audit

| Required result | Authoritative implementation and verification |
| --- | --- |
| Complete head/neck/torso/arms/hands/legs/feet/tail assembly, no hand placement | `NativeDpAvatarGeometry::generate` retains every allocated part and actual source mesh. `native_avatar_generation` verifies both anatomies, indices, attributes and referenced-vertex skinning. `native_avatar_attachments` checks head/neck/tail/proximal contact, all 12 distal sleeves, the base shoulder sleeve, unchanged source attributes and post-skinning edge defects. The repaired torso's other children must contact remaining original torso triangles, not a newly batched limb. All parts remain represented by source provenance. Multi-view visual inspection confirms no detached inspection pieces. |
| Native neutral standing, not unposed local meshes | `dp/neutral_pose.rs` composes native height, selective reset, both leg passes, tail, torso, heads and both arm passes. `native_neutral_pose.py` binds pose0x24 to Stand and native call/dispatch order. Four `native_body_mounted_standing` tests include composed expected matrices, measured angles, current attachment pivots and repeated posing without accumulation. No fabricated vertical offset or fallback weights. |
| Second supported anatomy with different proportions/counts | Base: 7 allocated parts / 12 original surfaces. Many: 12 parts / 21 original surfaces, two heads, one shared Neck22, two arm pairs and two leg pairs, different scale/thigh fields. DNA bytes are explicit in CLI and mounted tests. The same generator and render pipeline handle both; proximal repair is selected by posed geometry contact, never a fixture name. |
| Correct native texture inputs and angle smoothing | `native_body_textures.py` verifies atlas selection, DNA offsets, kind7 control and bottom-up RGBA anchors. Body-surface unit tests retain payload UVs, zero-weight membership, merge remaps and material partitions. `crease_normals::tests::duplicate_seams_join_but_threshold_and_nonmanifold_stay_hard` checks duplicate-position/UV seam smoothing and strict crease thresholds. Mounted tests compare every native and proximal-repaired corner with model-space crease normals followed by the actual raw-weight bone loop. Original UVs and controls are unchanged on all repair endpoints. GPU images were inspected for placement/orientation, top/bottom composition and obvious seam/part regressions. |
| CPU coverage for hierarchy, transforms, weights, normals, sequencing and unsupported inputs | Default/viewer/client library suites include native definition/deformation/merge, limb/torso/head/tail pose, shader-loop W/raw influences and normal transforms. Mounted generation and standing tests cover both complete fixtures. The unmerged adapter tests projection-before-deformation; the baked adapter avoids applying deformation twice. Armor/helmet, crest, separate PA eyes and unsupported winged/legless neutral cases return explicit errors. Modern sleeves reject invalid boundaries, new degenerate triangles and new edge defects. |
| Existing workflows and real multi-view GPU output | `python tests/avatar_previews.py reference/hazeron --body` launches 11 separate owned Rust preview processes: head, torso, both bodies in four directions, base untextured. Each warms shaders, saves a new PNG and exits; shader-error and nonblank tests pass. Every image in ignored `target/avatar-textures-vwjerdpe/` was opened for inspection. This is actual rendering evidence, not a process-exit-only claim. |
| Repeatable offline workflow, tests/lint/format, documentation and commits | `just preview-avatar body`, fixture/view arguments and `preview-avatar-untextured body` expand to the checked offline CLI. `--native-seams` selects the untouched native path. `test-native-avatar`, `test-native-avatar-attachments`, `test-native-avatar-torso`, `test-native-standing` and `test-avatar-body-previews` expose targeted reruns. Final gate commands and their results are recorded below; implementation and acceptance documents are committed. |

## Reproduction and gates

Final results: 724 default library tests, 743 viewer library tests, 756 client
library tests and 11 viewer CLI tests pass; their asset-dependent ignored cases
are tested separately. All 11 selected mounted CPU tests pass with ignored tests
enabled. All eight listed static verifiers, warnings-denied Clippy, formatting,
and diff whitespace checks pass. All 11 GPU captures pass and were inspected.
An additional current-binary front-view launch also saved and exited successfully;
`target/avatar-textures-vwjerdpe/body-base-front-final.png` was inspected and
confirms the final shoulder sleeve and complete base fixture are rendered.

Run inside `nix develop .#server-test`, or prefix each command with
`nix develop .#server-test --command`. No Ghidra imports or original binaries are
needed for these checks; static verifiers reuse retained evidence and SHA-bound
local disassembly.

```sh
cargo fmt --all -- --check
cargo test --offline --lib
cargo test --offline --features viewer --lib --bin openshores-client
cargo test --offline --features client --lib
cargo clippy --offline --features viewer --lib --bin openshores-client \
  --test native_avatar_attachments --test native_avatar_torso \
  --test native_avatar_generation -- -D warnings
env OPENSHORES_HAZERON_INSTALLATION=reference/hazeron cargo test --offline \
  --test native_avatar_generation --test native_avatar_attachments \
  --test native_avatar_torso --test native_body_mounted_standing -- --include-ignored
python tests/native_neutral_pose.py
python tests/native_body_definition.py
python tests/native_body_textures.py
python tests/native_body_triangle_upload.py
python tests/native_dp_torso_skin.py
python tests/native_leg_standing.py
python tests/native_arm_standing.py
python tests/native_dp_head_pose.py
python tests/avatar_previews.py reference/hazeron --body
```

## Deliberate limits, not substituted goals

- Original assets, native generated geometry, raw weights and posed bones are
  retained. Modern sleeve vertices copy existing positions and payloads; only
  derived faces and their smoothed corner normals differ. Sleeve UV interpolation
  uses existing endpoint UVs, not a newly recovered native unwrap. `--native-seams`
  exposes the original gaps for comparison.
- The narrow torso, small authored openings and native first-in-tolerance weld
  artifacts remain visible and documented. Inspection and contact/provenance
  tests distinguish these from missing or detached assembled parts. Watertight
  replacement anatomy was not an objective and has not been substituted for the
  native shape.
- Atlas inputs/composition and native transforms are verified by static evidence,
  CPU tests and Rust GPU output. No original executable/DLL was run; original
  runtime visual parity and faithful shading are not claimed. Lighting and
  angle-based smoothing are the requested modern diagnostic choices.
- Only user-provided, read-only mounted Hazeron assets are used. Captures remain
  ignored local files outside the installation and are not redistributed. No
  public game server, installation writes or reference-runtime execution are
  part of the workflow.
- Full production cache ownership/publication, received appearance, in-world
  placement, animation and clothing/armor remain the original explicit follow-on
  scope. Finishing this preview does not claim the Rust port is finished.
