# Native avatar geometry presentation boundary

`NativeAvatarPresentation::from_dispatched` still publishes only verified
received state and leaves model-space geometry absent. It does not infer DNA,
body models, assembly success, bone pose or global placement from identity.

A body-generation producer can now attach its assembled geometry through
`with_model_space_geometry`. That validates vertex/weight counts, face indices,
UV counts and values, and float-representable finite positions. CPU weight
payloads remain retained; this preview does not skin or animate them.

The native Bevy scene creates one child mesh per supplied part with source
triangles and UVs. Coordinates are explicitly unposed model space. Shading is
diagnostic cyan/unlit/double-sided, not native material reconstruction. Missing
geometry retains the diagnostic sphere; successfully supplied empty geometry
creates no substitute sphere. Geometry replacement on an unchanged wire ID
rebuilds the mesh children, and disconnect removes them.

Client-feature ECS tests exercise triangle/UV publication, same-ID replacement,
empty-model handling and disconnect, through the existing dispatched-Person
fixture. These are headless entity/mesh tests, not a screenshot or GPU parity
test. The invalid geometry boundary is also tested. No network is involved.

`just test-native-body-preview reference/hazeron` runs the opt-in mounted test.
It constructs a Person through the existing offline packet-dispatch harness,
independently initializes explicit fixture DNA using mounted hand calculations,
and attaches 17 resources / 3,410 CPU vertices as individual unmerged parts.
It compares every published position float bit pattern, triangle index and UV
against generated geometry, without depending on ECS iteration order, and
checks that disconnect removes every mesh entity. It opens no GPU window.
This verifies the mounted-assets-to-preview boundary, not DNA reception,
production context behavior, native merge success or visual/GPU parity.

Still required for automatic in-game avatars: native model cache/merge
application, an owning production receive context, attachment of actual assembled
body geometry after successful receive, bone posing/skinning, and verified world
placement. This rendering boundary alone does not complete avatar integration.
