# Native avatar textures: implementation ledger

Goal and completion gates: [avatar-textures-goal.md](avatar-textures-goal.md).
The interactive merged head and torso now use a mounted atlas, verified UVs and
native top/bottom region composition. See the GPU milestone below for scope and
verification. This does not complete automatic in-game avatars.

## Initial inventory and established resource selection

The read-only `AssetMount` supports both Qt model resources and loose
installation assets. Its cache retains byte-exact resources and diagnostic OBJ
conversions; it is not a texture decoder. No dedicated CPU image decoder exists
in this crate at the start of this work. Bevy's PNG decoding was initially enabled
by `client`, not `viewer`; the GPU milestone enables it for `viewer` as well.
The model decoder retains raw material texture names and face membership, but
that does not establish the body shader's skin texture bindings.

`DpBody::SelectTextures` at 1412c1310 reads body byte28 (DNA byte16), masks with7,
then tests masks84,21,42 in that order. Result:

| DNA byte16 low3 | Skin atlas | Clothes atlas |
| --- | --- | --- |
| 2,7 | tex/Skin.png | tex/SkinClothes.png |
| 0,5 | tex/Exo.png | tex/ExoClothes.png |
| 1,6 | tex/Scales.png | tex/ScalesClothes.png |
| 3,4 | tex/FurFeathers.png | tex/FurFeathersClothes.png |

Evidence: `evidence/native-dp-select-textures.json`,
REA ev_462ba95b20607c996f3d4ceb7c98d7ae3d8e143642fe8b1881cd877d399ed94a;
independent full-symbol disassembly in `native-dp-select-textures-gnu.txt`.
Original PE strings at1422bdb58/97/9b/a2 establish Skin/Exo/Scales/FurFeathers.
All these atlas calls pass min/mag2601 (GL_LINEAR), false,0 to bitmap creation;
orientation, wrapping, upload format/color space still require its implementation.
The same native function also loads Wings, vacuum/power armor, and helmet
textures; those paths are not implemented by the normal-atlas selector.

`NativeDpSkinAtlas` now reconstructs this normal atlas selection. Its
`read_skin` goes through the existing mount's bounded loose-asset lookup, not a
parallel path loader. It returns encoded bytes, not a decoded image or composed
skin material. Exhaustive tests cover all256 values of DNA byte16. The mounted
test confirms PNG signatures for all four skin and clothes pairs. The existing
head/torso fixture has DNA byte16=0 and therefore selects Exo, not Skin.

## UV/material upload investigation

New queries reuse the still-live `tests/native_session.py` session8086 and retain
complete results in `target/native-model-analysis-4`; there was no new import.
Ambiguous `Au3dsGLSLObject::LoadModel` was resolved with the exact TriBody
mangled overload, not a repeated identical query.

Retained evidence under `docs/evidence`:

- `native-body-create-device-objects.json`: array-to-object body VAO creation.
- `native-body-create-device-object.json`: calls the TriBody LoadModel overload,
  then `AuGLVAO::CreateForBody`.
- `native-body-load-body-model.json`: body triangle builder, including per-face
  material lookup, UV selection, and per-corner texture controls. REA
  ev_0ad9fb10afe5f344430f9e7a2f8c50c4508527e505d2d3c8e1f4cd1150b664c0.
- `native-body-get-texture-weight.json`: returns double0 for a vertex present in
  any retained kind7 group, otherwise double1. This is a skin-control input, not
  simply a UV coordinate. Existing `glsl-object-texture-offset.json` in the
  retained session shows fields20/24 copied into the six corner UV outputs.

The normal body path has a mapping-array presence gate before reading stored
vertex UV payloads. Model type3 follows a different indexed mapping path. Do not
replace these with an unconditional raw mapping-array copy. The triangle
builder also consumes material colors, object appearance controls, and kind7
membership. Decompiler float-return/argument artifacts need assembly checks
before implementing those arithmetic paths.

Next: corroborate UV/skin-control upload and actual texture binding/composition;
decode through the existing mount; introduce a separate textured render adapter
with per-corner seams and material partitions while retaining the untextured
adapter. Generate smoothing normals before render splits, so UV seams do not
accidentally become hard edges. Then wire the verified pipeline into the two
interactive previews and perform visual placement checks plus GPU smoke tests.

No texture pipeline, visual inspection, full material parity, or completion of
the texture goal is claimed by this resource-selection milestone.

## CPU surface-upload milestone

`NativeBodyMergeGeometry::textured_baked_preview_geometry` now projects the
already baked geometry once and publishes normal N/NT UVs from stored vertex
payload20/24, gated by the native mapping-array presence. It rejects absent
mapping, unsupported variants, unbaked geometry, malformed indices and nonfinite
UVs explicitly. It retains raw weights and does not replace the untextured
adapter. Per-vertex top/bottom values reproduce GetTextureWeight's kind7
membership test, including zero-weight membership. Every face's matching
material names remain retained in map order; partitioning includes an explicit
unassigned set. The adapter does not repair native head's off-by-one membership
or invent material colors. Synthetic tests cover head/torso merge UV remapping,
seams, material partitions and malformed/unsupported inputs.

The mounted CLI fixture now supplies verified SetupSkinMaterial atlas offsets
to the geometry plans instead of the earlier explicit all-ones placeholders.
Positions/counts remain unchanged. Mounted CPU checks pass with902 head UVs,
0 bottom vertices,2 material partitions;639 torso UVs,323 bottom vertices,
2 material partitions. These are CPU upload inputs, not GPU texture rendering.

### Appearance, shader and image evidence

The new `inspect-mounted-har INSTALLATION :glsl.har [ENTRY]` diagnostic reads
the supplied pre-patch sequential HAR through the existing mount/archive
decoders. `Body.f.glsl` and `Body.v.glsl` are present. Their contents are not
copied into this repository. Inspection establishes that this asset shader
uses unchanged UVs, blends bottom/top skin samples with the interpolated
top-face value, then optionally combines clothes by wear/alpha. It discards
fragments with alpha below0.15. Model material colors are not an independent
skin atlas selector in that shader. Its lighting is separate and remains
deferred. This is the supplied pre-patch shader evidence, not an observation
of the patched executable running.

`native-dp-setup-skin-material.json` plus independent full GNU disassembly
establish normal skin offsets: top selector DNA19&7; bottom DNA3&7. Each
selects U=(selector&3)/4 and V=0 for selectors0..3,0.75 for4..7.
The native doubles at4120/28/30/38 are narrowed to float at4070..7c for GPU
uniforms. `native_dp_skin_offsets` implements this subset; exhaustive tests
cover all256x256 input bytes. This does not implement all SetupSkinMaterial
QColor, shininess or armor fields. Fixture selectors are both zero.

`native-glsl-body-build.json` binds Body.v/Body.f and resolves skin/clothes
offset uniforms. `native-vao-create-for-body.json` publishes per-corner UVs and
top-face controls. The retained RenderSkin query in the same analysis session
selects the same atlas families and binds skin without clothes for owner4068=0.

`native-gl-texture-create-bitmap.json` and its independent GNU disassembly
establish bottom-to-top image row traversal and RGBA byte order. The bitmap
effects wrapper (`native-effects-create-bitmap-image.json`) delegates the
upload and can choose sRGB under the patched linear-lighting mode. Do not infer
that the original renderer always used sRGB. Preview color-space and lighting
choices must be explicit. Wrapping defaults and actual GPU decode/upload still
need implementation/verification.

`python tests/native_body_textures.py` pins the original PE digest, family
strings, offset constants1024/4096/0.75, selector reads, kind7 control and row
order anchors. `just test-native-textures` combines static checks, library tests,
and the real mounted head/torso CPU-surface fixture. No target execution occurs.

Next implementation: decode atlas PNGs, apply the verified row orientation,
publish per-corner UV/top-face/material partitions while computing smoothing
before render splits, and implement the two-sample skin blend in a preview
material. Keep owner4068=0 fixture clothing disabled, expose an untextured
comparison mode, then run GPU and visual placement checks for both parts.

## GPU texture milestone

`viewer/native_skin.rs` reuses Bevy's existing PNG decoder. Before decoding it
requires a complete PNG envelope, bounds encoded size to32MiB and dimensions
to4096x4096, and rejects animated PNGs. Decoded RGBA rows are reversed to match
the verified native upload. Missing mount resources and unsupported/corrupt
images return errors naming the atlas; there is no substitute texture.

The new, independently written embedded WGSL contains only Rust-client shader
code, not copied game assets/shaders. It implements two atlas samples plus
interpolated top/bottom blending, with alpha cutoff0.15. The explicit fixture
has owner4068=0, so clothes are disabled as in the retained native RenderSkin
branch. The fixture DNA remains byte1=1,12=4,14=0x51,20=0x60, all others0;
DNA16 selects Exo and both skin palette selectors select tile0. Skin-image
bytes are read at launch from the installation, never embedded or cached there.

Modern preview choices: sRGB atlas decoding, Bevy PBR (or the L-toggle unlit
mode), linear min/mag filtering, repeat wrapping, double-sided surfaces, no
native specular/glow emulation, and no shadow casting from these debug parts.
Linear filtering and RGBA row orientation are native-evidence-backed; repeat
wrapping and modern lighting/color-space policy are explicit preview choices.
Native image upload can use sRGB under its linear-lighting setting; no claim is
made that every original mode did. Full clothing, armor, shader lighting and
received appearance remain outside this normal fixture preview milestone.

Render-only mesh splits retain each triangle corner's UV, skin control and
normal, then partition by the retained material-membership sets. All partitions
use the body-selected atlas, as the normal body shader does. Source positions,
indices, weights and material metadata are unchanged. Normals are calculated on
the complete surface before splitting, so UV/material seams do not become hard
edges. Unit tests verify both smooth and hard thresholds, distinct UVs at exact
coincident positions, per-partition corner/control data, and actual ECS image,
material-uniform and lighting attachment. The untextured adapter is retained.

### Reproduction and visual check

- `just preview-avatar` / `just preview-avatar torso`: textured head/torso.
- `just preview-avatar-untextured [head|torso]`: untextured comparison.
- Both recipes retain installation and angle arguments, defaulting to
  `reference/hazeron` and60 degrees. 0 means flat normals.
- `just test-native-textures`: static anchors, CPU tests, all four mounted atlas
  decodes, and both mounted merged surfaces.
- `just test-avatar-previews`: real GPU windows, captures and nonblank checks.

Clean GPU captures were inspected on the AMD Radeon AI PRO R9700 / RADV Vulkan
adapter: `target/native-head-textured-final.png` and
`target/native-torso-textured-final.png`. The head shows the selected gold Exo
skin pattern, mapped eye details and pale jaw geometry. The torso displays the
same selected skin region over its merged segments, including the pale lower
surface. These are existing unposed model-space parts: the thin/overlapping
torso shape and unposed head attachments are not texture fixes or full assembly
claims. No arbitrary image projection or geometry repair was introduced.

Initial60-frame captures were blank because cold PBR/driver pipeline compilation
had not finished, even for a temporary StandardMaterial diagnostic cube. That
diagnostic cube and logs were removed. Evidence capture now waits600 frames and
requires900+ smoke frames; the repeatable GPU test checks the saved pixels for
substantial non-background coverage and color variation, not just a clean exit.
Cold compile time is not a reason to redo native static analysis. Captures are
local ignored files containing user-provided assets, not redistributable fixtures.

### Completion audit (2026-10-15)

- Source/selection evidence and CPU pipeline: retained native records above;
  `native_body/textures`, `body_surface_preview` and render-split tests cover
  palette selection, merge remapping, material sets, skin controls and seams.
- Image/missing-resource behavior: malformed/truncated/oversized/animated inputs
  are rejected; the missing-atlas test confirms an error rather than fallback.
  All four mounted atlases decode successfully through the existing mount.
- GPU publication and smoothing: ECS asset/uniform tests plus actual renderer
  captures; smooth normals remain continuous across UV/material splits, and
  89-degree/91-degree tests distinguish hard/smooth sides of a90-degree edge.
- `just test-native-textures` passed:704 library tests, the mounted four-atlas
  decoder test, both mounted head/torso surface fixtures, and static PE anchors.
- Full client-feature regression:717 library tests,10 CLI tests and1 slice-bin
  test passed;6 opt-in fixtures were left ignored in that default suite.
- `cargo clippy --offline --features client --lib --bins --tests -- -D warnings`,
  formatting and diff checks passed.
- `just test-avatar-previews` passed twice, including the final no-clobber
  capture writer. Final inspected images are in
  `target/avatar-textures-_oka5mvt/{head,torso}.png`:166,816 head foreground
  pixels/26,933 colors and32,929 torso foreground pixels/11,521 colors.
  Both windows were reopened offline; no shader errors were reported.
- Sorted installation filename/size/mtime/ctime metadata hashes matched before
  and after verification:
  `ac9a2a3cacad40edc5a0ccb151fab37e684b0e124889add5caed477e78bd562b`.
  Only read-only asset APIs are used. Screenshots are exclusive-create files
  outside the installation; existing files and broken symlinks are not overwritten.
- Milestones are committed separately: atlas selection (`c5b1e90`), CPU surface
  inputs (`b2537cc`), and the final GPU preview implementation.

The requested existing-head/torso texture milestone is satisfied. This audit
does not claim full native shading, arbitrary armor/clothing support, animation,
received player appearance, or finished in-game avatars.
