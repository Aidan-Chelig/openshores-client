# Native body merging: implementation boundary

Static REA results from the existing warm session are retained in
`evidence/native-body-merge-head.json` (1413a7ea0),
`evidence/native-body-merge-torso.json` (1413aa3e0), and
`evidence/native-body-add-if-unique.json` (14139c1c0).
The three queries took 0.581s, 0.423s and 0.090s respectively; no executable
reimport, target execution, or public network request was needed.

Focused inspection establishes that both merges seed a missing destination
through CopyInstance, rather than concatenating arbitrary rendered triangles.
Head returns immediately after that seed. Torso has a distinct bool branch.
Both subsequently handle native object/group/face/material storage. Full merge
reconstruction remains incomplete: neither routine may be replaced with simple
mesh concatenation or a universal position-weld rule.

The inspected head connection branch uses inclusive per-axis absolute
differences, while the inspected torso connection branch uses strict Euclidean
distance. Head also calls AddIfUnique with its final bool true. These are
different operations and must remain distinct.

`native_body_add_if_unique` implements that complete small callee: first matching
position by strict Euclidean distance, double arithmetic on stored floats,
conditional float equality of opaque fields20/24, and snap-and-append when the
fields differ or comparison is disabled. It preserves all ten payload words;
the fields are not guessed to be UVs, normals, or semantic skin data. Tests cover
the strict boundary, first-match priority, snapping, payload preservation,
signed zero and NaN field comparisons. It does not implement the larger merge.

Next required reconstruction is ordered connection-group remapping, original
face/material/group updates and CopyInstance state. The retained complete
records can be paged locally; do not request the same functions again.

## One-time destination preparation

`NativeBodyMergeBake::bake_once` now implements the shared first-object
preparation observed at head lines128..271 and torso lines159 onward of those
retained pseudocode records. If field369 is clear, apply the actual object
deformation to stored f32 vertex coordinates (not triangle-builder-projected
coordinates), narrow the result back to f32, rebuild only kind9 BottomConnect
position maps through the same operation, then set field369. Other vertex
payload words and group metadata remain unchanged. Calling again is a no-op.

The caller supplies actual object deformation and Top/Bottom scale points;
this type does not invent constructor/model state or stand in for a complete
native object. Map rebuilding follows numeric original-key iteration order,
retains first stored keys on collision and assigns the last weight. Earlier
writes survive a later Unsupported nonfinite-coordinate error and field369
stays clear. This explicit finite boundary does not claim native nonfinite
runtime behavior. Tests cover idempotence, payload preservation, only-bottom
map mutation, ordered collision handling and partial-error state.

CopyInstance, source-object preparation, welding/remapping and instance
publication remain unimplemented. Do not call this stage a complete merge.

## Head source-vertex mapping

`NativeBodyMergeBake::append_head_vertices` now reconstructs the next head
vertex loop (retained pseudocode lines275..502), with complete independent GNU
disassembly retained in `evidence/native-body-merge-head-gnu.txt`. The caller
must provide an already baked destination and source deformation after native
connection-point inheritance. This API does not perform that inheritance.

For each source vertex, only the first kind8 TopConnect group determines
connection membership, before source xyz mutation. ApplyScaleFactor writes
the narrowed stored coordinate. Nonconnections call AddIfUnique on a copy;
connections instead scan destination vertices with inclusive per-axis tolerance,
snap xyz, and always append. Destination kind9 group existence (not candidate
membership in its map) makes the first match terminate the scan. Without such
a group, snapping can continue from the updated position through later matches.
The stage retains ordered source-to-destination indices and the native two
first-insertion-wins connection remap tables. Source group keys and payloads
are not rewritten by this loop. Tests distinguish these rules from generic
position welding. Nonfinite tolerance, unfinished destination baking and index
capacity beyond the native signed-int range are explicit unsupported cases.

Still required: complete instance CopyInstance/state inheritance, head face,
material/group update stages, torso's different vertex/remap path, and final
cache/model publication. No complete merge or runtime parity is claimed.

## Stored faces and head appended-face fields

The warm session additionally retained `native-body-load-face-list.json`
(1413a3e00, 0.134s) and `native-body-copy-instance.json` (1417ad9f0,
0.237s). CopyInstance is retained for subsequent state-lifecycle work; it is
not implemented by copying only a Rust vertex vector.

`NativeBodyLoadedFace::from_serialized` models assigned native face fields
separately from the serialized `Face`. The first nine N/NT floats widen into
native50..90, NT's next nine widen into08..48, and its final float sets98;
otherwise arrays/default98 are zero/one. The trailing9c..a4 words initialize
to zero. Mismatched raw extension/variant combinations are errors.

`NativeHeadVertexMapping::head_appended_faces` implements the appended-face
field construction at retained head lines530..676: mapped source indices
narrow to u16, flags06 and08..48 become zero, fields50..90 are copied,98=1,
and9c..a4=0. Degenerate mapped triangles are retained, not culled. Tests cover
raw-to-loaded N/NT/default storage, metadata resets, missing mappings, native
u16 narrowing, and actual connection-vertex maps feeding the face stage.

This stage does not perform the conditional connection-normal hash table,
face-material mapping, smoothing updates or later face/group removal/rewrite.
These remain required parts of full head merging; the returned faces alone
must not be treated as completed instance publication.
