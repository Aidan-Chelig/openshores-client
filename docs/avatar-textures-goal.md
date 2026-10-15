# Next goal: textured native avatar previews

Implemented for the existing merged previews. See the requirement-by-requirement
[completion audit and limitations](native-avatar-textures.md#completion-audit-2026-10-15).

Render the existing merged head and torso previews with the appropriate textures
from a user-provided Hazeron installation, preserving verified UV and material
assignments through loading, deformation, merging, and GPU upload. Keep the
angle-based preview smoothing available. This is the next milestone toward the
Rust OpenShores port, not a replacement for full in-game avatar integration.

## Work sequence

1. Inventory existing Rust texture decoding, mounted resources, model material
   records, and retained native evidence. Identify the actual texture-selection
   and UV-generation consumers for these body parts. Reuse retained analysis;
   only investigate missing dependencies, using one reusable analysis session.
2. Establish the head and torso texture paths separately: resource selection,
   material-to-face assignment, generated or loaded UVs, merge remapping, and
   any required skin-color/compositing inputs. Record what is verified versus
   still unsupported. Do not assume raw per-vertex UVs alone texture merged
   geometry correctly or invent an atlas/material lookup.
3. Implement the required CPU texture/UV/material pipeline with offline tests.
   Preserve per-corner seams by splitting render vertices when UVs, materials,
   or normals differ. Keep render-only splits separate from native geometry and
   weight payloads. Preserve the existing untextured adapter as a diagnostic.
4. Load decoded textures through the read-only asset mount and attach the
   resulting materials to the preview meshes. Establish UV orientation, color
   space, alpha handling, and sampler behavior from evidence where needed;
   label any diagnostic rendering choices explicitly. Preserve auto smoothing.
5. Expose textured head/torso through `just preview-avatar`, retaining an explicit
   untextured option for comparisons. Verify both on the mounted installation
   and in GPU window smoke tests; document commands and remaining limitations.

## Completion criteria

- Both existing merged previews visibly display the appropriate mounted body
  textures with verified face assignments and UVs, rather than arbitrary images
  projected onto the models. Record any fixture-only appearance inputs.
- Tests cover UV/index remapping across merges, seam vertex splitting, material
  partitioning, texture lookup/decoding, and missing or unsupported resources.
- Texture failures are reported explicitly; no silent fabricated appearance or
  claim of native parity for unsupported composition paths.
- Angle-based normals still work across UV seams and retain sharp edges.
- Reopening either preview works offline without modifying the installation.
- Targeted tests, lint, and actual GPU window smoke tests pass. Record a visual
  inspection of texture placement; a window smoke test alone is insufficient.
- Document results and commit meaningful implementation milestones.

## Boundaries

Only use user-supplied assets. Never contact public servers, execute reference
binaries/DLLs, modify the reference installation, or redistribute its assets.
Any extracted/converted cache belongs outside the installation. Reuse existing
decoders and asset-mount facilities instead of creating a second asset pipeline.

Modern diagnostic lighting and auto smoothing are acceptable; faithful native
shading is deferred. Posing, animation, complete avatar assembly, production
receive-context ownership, and world placement remain separate follow-on work.
Do not describe textured preview parts as finished in-game avatars.
