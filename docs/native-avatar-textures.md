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
