# Native avatar textures: implementation ledger

Goal and completion gates: [avatar-textures-goal.md](avatar-textures-goal.md).
Textured rendering is **not complete**. Existing previews remain untextured.

## Inventory and established resource selection

The read-only `AssetMount` already supports both Qt model resources and loose
installation assets. Its cache retains byte-exact resources and diagnostic OBJ
conversions; it is not a texture decoder. No dedicated CPU image decoder exists
in this crate. Bevy's PNG decoding is currently enabled by `client`, not `viewer`.
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
