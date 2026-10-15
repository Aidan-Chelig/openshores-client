# Native avatar geometry presentation boundary

Next milestone: [textured native avatar previews](avatar-textures-goal.md).

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

The same task now also runs a merged-head/torso case. Five head assets and three
normal torso segments run through the reconstructed geometry merge stages,
then explicitly project their baked stored vertices through their ORIGINAL
seed mesh matrices exactly once at loader-scale1. The resulting two meshes
(2,224 triangles) are attached to the dispatched-Person presentation fixture.
Every uploaded position bit pattern and triangle index is checked by the same
headless ECS helper; disconnect removes both meshes. The fixture supplies DNA
and owner fields independently, so this is not automatic receive-driven body
generation or complete model/cache lifecycle evidence.

`untextured_baked_preview_geometry` refuses unbaked geometry, retains native
weights and loaded face fields, and intentionally provides no UVs. This is an
unposed/untextured diagnostic rendering adapter, not faithful texturing or skin
animation. Its translated-matrix regression checks projection is neither
omitted nor applied as another local inversion. Native global/world axes are
still not inferred.

Still required for automatic in-game avatars: native model cache/merge
application, an owning production receive context, attachment of actual assembled
body geometry after successful receive, bone posing/skinning, and verified world
placement. This rendering boundary alone does not complete avatar integration.
# Interactive mounted previews

Run `just preview-avatar` for the merged head or `just preview-avatar torso`
for the merged torso. An optional second argument selects the user installation
(default `reference/hazeron`), and a third selects the smoothing angle in degrees
(default 60). For example, `just preview-avatar head reference/hazeron 30`.
Use 0 for flat shading. The CLI also accepts `--smooth-angle 0..180`.
These open the offline Rust geometry inspector;
no server connection or reference executable is involved, and the installation
is read-only. Drag to orbit, wheel to zoom, L toggles lighting, W toggles wire
display, R resets the camera, and Escape closes the window.

These use the same explicit fixture DNA and head/torso geometry stages as the
mounted preview tests. They are separate, untextured, unposed model-space parts,
not a complete avatar, received player appearance, or native instance/skinning
parity. The inspector centers and scales geometry for viewing only.

Auto smoothing generates area-weighted corner normals across shared edges whose
face angle is strictly below the threshold. Exact coincident positions identify
seam edges without changing the source vertices. Sharp, boundary, non-manifold,
and degenerate edges remain split; disconnected fans touching only at a vertex
are not smoothed together. These are diagnostic render normals, not an
interpretation or replacement of native smoothing groups/normal records. Terrain
and the connected client keep their existing normal generation.
