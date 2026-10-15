# Native model presentation investigation

Target: supplied patched `reference/openshores-full/OpenShores.exe`, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
No original executable is run. REA/Ghidra reads an immutable imported image.

Model reconstruction checklist:

- Filename/format mapping: answered by static instructions (see below).
- Mesh matrix and chunk-order coordinate stages: implemented below; caller
  scale is explicit, entity-specific scale selection remains unresolved.
- Which coordinate conversion is applied before item/entity placement?
- How are material colors, face assignments and texture coordinates selected?

Static COFF/objdump anchors in this exact binary:

- `Au3dsGLSLObject::LoadMatrixChunk(QDataStream&, int, MODEL_TYPE)`:
  `_ZN15Au3dsGLSLObject15LoadMatrixChunkER11QDataStreami10MODEL_TYPE`,
  entry `0x14139c3d0`. At `0x14139c44e` the fourth argument is compared with 3;
  that branch targets `0x14139cc30`. The other path calls `Au3dsGLSL::LoadFloat`
  repeatedly and constructs additional matrix state. These are instruction
  observations, not yet a verified model-format mapping or rendering algorithm.
- `Au3dsGLSLModelLoader` constructor: entry `0x141399730`. Constructor aliases
  share an address; disassembly by start address avoids selecting an empty alias.
- Material loaders are named `LoadColorChunk`, `LoadTextureChunk`, and
  `LoadChunk` under `Au3dsGLSLMaterial`; these are follow-up anchors, not proof
  that a particular Rust color/texture policy is faithful.

The initial full-binary Ghidra analysis timed out without matrix evidence;
the Java process was confirmed stopped and the inactive MCP target closed.
Recovery uses the installed REA CLI with its supported
`REA_GHIDRA_STARTUP_TIMEOUT_MS` raised to 30 minutes for this invocation only.
`tests/native_decompile.py` retains complete JSON in a caller-selected new file
and does not execute the target. Recovery completed successfully in 665 seconds;
the complete output is retained in `docs/evidence/native-model-matrix.json`.
The saved result and symbol-backed caller instructions establish the coordinate
stages implemented below; connected item shading remains a prototype.

The Rust raw-format implementation now lives under `src/model/`: public data
types in `mod.rs`, bounded chunk decoding in `decode.rs`, parity fingerprints
in `fingerprint.rs`, and regression fixtures in `tests.rs`. This is a structural
split only. `just check` passes (94 headless / 111 client library tests, plus
CLI tests and strict Clippy). `tests/reference_models.py` independently verifies
209 valid models / 618 meshes, geometry and material fingerprints, exported
positions/faces, and topology diagnostics after the split.

### Additional static findings

`tests/reference_native_models.py` revalidates the same target SHA-256, mapped
string bytes and exact branches, and counts symbol-resolved LoadFloat calls
using GNU objdump. These are shell/static observations, **not REA Evidence**.

- `Load(QFile&, double)` at `0x1413a6620` searches `.3dsX`, then `.3dsNT`,
  then `.3dsN`, assigning types 3, 2, 1; unmatched filenames use type 0.
  This reconfirms the earlier finding in `docs/reconstruction.md`.
- Face loader `0x1413a3e00`: types 1/2 enter the nine-float reader at
  `0x1413a4158`; comparison with 2 at `0x1413a41f8` admits the ten-float
  tail reader at `0x1413a43d0`. NT raw decoding is now implemented, preserving
  both arrays and subsequent child chunks; synthetic tests cover truncation,
  wrong variant, nonfinite tails and digest sensitivity. No real NT asset
  was present in the previously audited resource pool.
- Matrix loader's type-3 branch at `0x14139cc30` scales the existing vertices'
  coordinates at offsets `0x28/0x2c/0x30` by binary64 `1/12` from
  `0x1422cf2b0`, rounding to float, then rejoins the common twelve-float matrix
  reader at `0x14139c458`. It does **not** read a different-width matrix.
- Nonidentity matrix handling copies state to object offset `0x80`, performs
  inversion-like elimination, and projects vertex coordinates with that state
  at `0x14139cbbb`. The textured triangle loader projects them again using the
  object's first matrix at `0x14139e490/0x14139e4cc/0x14139e508`. Consequently,
  applying a raw mesh matrix once is not an established native rendering rule.
  Exact inversion, chunk-order effects, caller scale and final entity placement
  remain to be corroborated before changing connected geometry.

After NT support: `just check` passes with 95 headless and 112 client library
tests, CLI tests and strict Clippy. `just test-native-models` checks the patched
instruction anchors and compares the Rust NT decoder's geometry fingerprint
against the independent Python raw-byte decoder, including post-face smoothing
and material chunks. The 209 existing Standard/N/X model fingerprints and
preview/audit checks still match. No original code is executed by either test.

### Completed REA matrix-loader evidence

Evidence ID:
`ev_16888118f3282f39d48aaa1912788e36604e4a039c7e5602fe509dce2ad0704f`.
Ghidra 12.1.2, operation `procedure_pseudo_code`, exact patched target digest
above, default analysis profile digest
`8e3a14075ec67e0acc0e527cf433a32154bc1e12800cc3154deb7d62e385ced2`.
CLI exited 0, its complete JSON envelope was validated, and its ephemeral
analysis process finished. The original image was not executed or modified.

Pseudocode corroborates twelve LoadFloat reads for every model type, X-specific
vertex and translation scaling, the nonidentity check, a pivoted in-place
inversion of the copied matrix, and projection of existing vertices into that
local frame with float storage. These are static observations/inferences, not
runtime parity. The following textured-loader projections remain supported by
symbol-backed GNU instructions only; they were not part of this REA query.

Decompiler caveats matter: its X vertex scaling expression displays a rounded
float constant (`0.083333336`), whereas the actual instructions widen floats,
multiply using binary64 `1/12`, and narrow the result. The final Project call's
recovered parameter list is also incomplete; inspect the instruction arguments
at `0x14139cb90` rather than treating that pseudocode signature as source.
The engine accepts 12.1.2, but REA's bridge was verified against 12.1.4 and
reports that limitation. Full raw/normalized output and provider limitations
are retained with the evidence. At evidence-capture time, chunk order, singular
handling and caller scale still needed reconstruction; see the implementation
below. Helper meshes and entity orientation remain unresolved, and connected
rendering does not claim complete native-transform parity.

### Reconstructed coordinate stages

`src/model/coordinates/` now implements these stages without modifying the raw
model, its source bytes or existing geometry/material parity fingerprints.
The decoder separately records vertex/matrix chunk order. With initial identity
state (constructor `0x141788c80`, diagonal writes `0x141788da8..0x141788db6`),
vertex loading uses the current inverse when nonidentity. Matrix loading:

1. For X only, scale **existing** vertices using binary64 `1/12`, storing float.
2. Read the twelve floats into column-major binary64 state, scaling translation
   by `1/12` for X; basis axes are not scaled by that conversion.
3. For a nonidentity matrix, copy/invert it using the observed pivoted in-place
   algorithm and project existing vertices into that local frame, storing float.
4. Apply an explicit positive loader caller scale to stored vertices (float
   rounding) and matrix translation (double), then project vertices with the
   forward matrix as the textured triangle builder does.

The identity check precedes inversion; X without a matrix chunk is not blindly
rescaled. Matrix-before-vertices preserves the native order-dependent behavior.
Additional GNU instruction observations: vertex loader `0x14139b6d8` uses
object inverse state and projects at `0x14139b7ab`; positive caller scaling
in `Load(QFile&, double)` is at `0x1413a7208` (translation followed by vertex
`ScaleBy`); `AuMatrix4x4::Project` at `0x1418fd760` adds x/y, z, then translation
with zero-coordinate fast paths. These additional functions are **not** part
of the saved REA matrix-only evidence; the operation order is corroborated
with their symbol-backed instructions, not claimed as runtime observation.

Connected item meshes now consume those coordinates with loader scale 1.0,
then apply planet display scale. Normal generation and colors remain temporary.
`dump-model-coordinates MODEL_FILE [LOADER_SCALE]` provides a headless numeric
view. Model CLI handlers were split into `src/cli/models.rs`.

Verification: `just check` (100 headless / 118 client library tests, CLI tests,
strict Clippy), independent NumPy inversion/project checks across 209 supplied
models / 91,638 vertices and 96 synthetic format/matrix/order/scale combinations,
and mounted prepatch Flag/Hay/Torch numerical checks. Original raw fingerprints,
OBJ/SVG exports and audits remain unchanged. `just test-gui` passes against the
disposable loopback server, including mounted Hay/Flag mesh assignment and the
inventory/chat/movement workflow. This checks the Rust runtime, not a pixel or
original-client rendering comparison.

Deliberate limits: positive finite caller scales only; singular matrices and
nonfinite storage are explicit errors. Native zero-pivot handling instead
projects with partially inverted state, which is not reproduced here. No
singular matrix occurred in the 209-model numerical check. The NumPy oracle
uses a separate inversion method and a float-storage error bound, not bitwise
original-runtime parity. Entity axes/orientation, caller-specific size selection,
helper filtering, master-scale interpretation, normals, materials/UVs/textures
and animation are not established by this coordinate implementation.

### Native acceptance and helper removal

`src/model/roles.rs` implements object acceptance and metadata classification;
`Model::mesh_roles` traverses file order with shared screen-descriptor state.
`Mesh::role` is an isolated-object diagnostic, not the full-model traversal.
Recognized helpers remain in raw models, OBJ exports, captures and caches;
the connected item renderer excludes them from draw geometry. Metadata-only
models produce an empty mesh rather than being treated as missing assets and
replaced with primitive spheres. Ordinary `-`-prefixed objects and `SPt` names
are not blanket-filtered: the inspected code uses different rules for extents
and body scale markers.

Static rules in this exact patched executable:

- `LoadChunk` at `0x1413a5be0` appends each unsigned name byte as a QChar.
  Every accepted object needs vertices. Standard/X additionally need faces;
  N/NT permit face-free objects with a leading `!` or `@`, or exact `DataOnly`
  or `UpVector` names (`0x1413a5d8a..0x1413a5e60`).
- `TestForWeapon` at `0x141398020`: nonempty vertices, name length >1 and
  leading `!`; the second QChar is parsed alone in decimal. Failed conversion
  returns 0 and the caller does not check success; that still passes its
  unsigned-short <=26 test. All one-byte names therefore yield ID 0..9.
- `TestForStation` at `0x141398280`: vertices, name length >3 and leading `@`;
  second QChar supplies the same ID; characters 2 and 3 parse as hexadecimal
  unsigned short, with success checked and nonzero low byte required.
- `TestForScreens` at `0x141398530`: group types 21/22 (exact `Screen1` and
  `Screen2`, assigned at `0x14139a719..0x14139a753`) each need at least four
  distinct position keys in the native QMap. Duplicate positions replace keys;
  signed zeros compare equal. An undersized relevant group makes this screen
  test fail immediately. The caller initializes the two descriptors once at
  `0x1413a6bfa/0x1413a6c06`, before its object loop at `0x1413a6cdb`, and reuses
  them. This order-dependent state is reproduced, including later objects with
  no screen groups after both descriptors have been established. A failed test
  does not clear previously established descriptors.
- `TestForUpVector` at `0x14139abe0`: nonempty vertices, length >3 and a
  case-sensitive `UpVector` substring. Tested after station, weapon and screens.
- Caller `Load(QFile&, double)` tests those roles at
  `0x1413a6d2d/0x1413a6f12/0x1413a797e/0x1413a7a6e`, then erases recognized
  objects through `0x1413a6ef6` or `0x1413a7aae`. This is the basis for excluding
  helpers from drawing, not their names alone.

Custom vertex groups are now decoded boundedly: mesh child `0x4112` contains
`0x4113` (null-terminated raw name) and `0x4114` (u16 count, then position xyz
and weight as four floats per record). Native `LoadVertexGroupChunk` at
`0x141399c00` confirms this layout, including its four-LoadFloat one-record path
at `0x14139a090`. Names, record order, weights, duplicates and signed-zero bits
are preserved. Duplicate known fields, truncated/nonfinite records and budget
overruns are explicit errors; unknown children remain opaque. This does not
implement native skin/bone/group animation or helper interaction poses.

REA corroboration for the weapon predicate: complete evidence retained in
`docs/evidence/native-model-weapon-helper.json`, ID
`ev_59c92ca44904e255642b6f70dc562965ec034b77e6c7a30004d7e61ee6208154`.
Ghidra 12.1.2, matching target and analysis-profile digests above, static
`procedure_pseudo_code`; CLI exited 0 in 648 seconds and closed its ephemeral
process. Decompiled Qt constructor/conversion argument lists are incomplete:
the QChar source and explicit decimal base come from instructions at
`0x141398095..0x1413980ad`. Station/screen/up-vector and caller-removal details
above are GNU disassembly observations, not additional REA decompilation or
original-runtime observations.

Header conversion behavior is cross-checked with primary Qt 5.15 sources:
[QString numeric conversion](https://raw.githubusercontent.com/qt/qtbase/5.15/src/corelib/text/qstring.cpp),
[QLocale C-locale conversion](https://raw.githubusercontent.com/qt/qtbase/5.15/src/corelib/text/qlocale.cpp),
[unsigned conversion rejection](https://raw.githubusercontent.com/qt/qtbase/5.15/src/corelib/text/qlocale_tools.cpp), and
[QChar whitespace rules](https://raw.githubusercontent.com/qt/qtbase/5.15/src/corelib/text/qchar.h).
The two-character flags parser permits leading plus and surrounding Latin-1
whitespace, including NEL/NBSP; it rejects negative, malformed and zero flags.
This is source corroboration of imported Qt semantics, not execution of the
supplied Windows Qt DLL or proof of every Qt build.

Verification: 209-model independent checks match 70 raw-group fingerprints and
all 618 helper classifications (572 geometry, 37 station, 7 weapon, 1 screens,
1 rejected). Existing geometry/material fingerprints and coordinate checks are
unchanged. Synthetic checks cover names/case, malformed and empty groups,
position-key deduplication, shared screen state and helper precedence.
`just check` passes (106 headless / 125 client library tests plus CLI tests and
strict Clippy); local GUI and user-installation mount/cache checks pass with
all 4,882 assets retained and the same 2,224 conversions / 2 opaque models.
No original executable or DLL is executed, and no public game service is used.

## Native body deformation: unweighted branch

`NativeBodyObjectScale` starts the actual CPU skin-deformation path rather than
adding more skeletal animation or treating raw assets as finished avatars.
`Au3dsGLSLObject::ApplyScaleFactor` has a hidden AuPoint result in RCX, object
in RDX and input point in R8; the decompiler's one-argument signature is wrong.
The proven no-group/no-matching-group branch reads double140, float offsets
1c0/1c4/1c8 and float connection coordinates1f4/1f8/1fc. Float metadata widens
before independent double products. It computes offset-origin*scale plus
point*scale, NOT (point-origin)*scale+offset. Zero scale skips input/origin
arithmetic completely, including NaNs; NaN scale takes the nonzero path.

`Mesh::body_unweighted_positions` composes existing native coordinate loading
with this operation ONLY when native object loading retains no deformation
groups (including meshes containing only extracted control points). A mesh with
retained groups returns Unsupported. This is a temporary support boundary, not a
claim all grouped vertices require deformation or a successful uniform-scale
fallback. The low-level unweighted helper requires callers to establish that
the point has no matching groups. Renderer publication, weighted deformation,
body assembly, bone assignment and production Person context remain unfinished;
this does not make avatars visible in-game.

Complete static Evidence
`ev_413b8d0889ff3bece7696843079d27ff60f635084248ddc2dc5bb4eba8c1bbb0`
is retained in `docs/evidence/native-body-apply-scale.json`, with full GNU in
`native-body-apply-scale-gnu.txt`. `just test-native-body-scale` verifies the
original SHA256, hidden-result ABI, float metadata, no-group gate and exact
zero/nonzero branch arithmetic, then tests cancellation-sensitive products,
zero/NaN/nonfinite inputs, coordinate-stage ordering, input preservation and
explicit grouped-mesh rejection. These are static/Rust checks, not native
execution, weighted-mesh parity or rendering verification.

## Native body vertex-group membership

`NativeBodyVertexGroup::from_group` now reconstructs native classification,
point extraction and the position-key weight map from raw ordered group fields.
The default type is1. ASCII tokens are tested case-sensitively in native order:
ScalePt/ConnectPt/MidPt and exact point names precede Torso/Hand/Foot/Neck/Tail/
Head flags and special helper names. Consequently TopScalePt matches the early
ScalePt substring; the later exact subtype test is not substituted for it.
Native WeaponUDPt/LRPt types19/20 and original missile-name spellings are kept.

One serialized record NEVER inserts membership. It assigns a point only for
the verified type3/type10 or literal-mask fast path. Multi-record groups insert
position/weight pairs except the final record, which becomes a point for types
2..6/10/11 or any anatomical flag. Classification at the time the Members field
is read is authoritative; a later Name cannot retroactively remove a weight
or recover a pivot. Duplicate positions replace the weight but retain the first
stored key bits. Signed zeros compare equal; entries are sorted numerically by
XYZ, and weights widen from f32 to f64. Zero weight still denotes membership.
ApplyScaleFactor narrows queried coordinates to f32 before lookup. NaN keys
return Unsupported instead of guessing unordered Qt map behavior; malformed
field order/nonfinite raw records return InvalidData under existing Rust policy.

Complete retained records are `docs/evidence/native-body-group-members.json`
(`ev_ee5907c272b7213258a4647ea21954cbfc8217d4be3d3d67e1a1cebe0c8bcf08`),
`native-body-group-classification.json`
(`ev_b424b56c7897d03f5f9d1a3ab97f664f65fe7a0dbecde663aafd5faca9ee4c3e`)
and `native-body-group-object-loader.json`
(`ev_878a04dbefe1ad0e5e77303804582e10359a2509bbab1dc39bd513e75ec46285`).
Full member-loader GNU is in `native-body-group-members-gnu.txt`. All REA records
were reused, not requeried or reimported. `just test-native-body-groups` checks
SHA-bound defaults, name precedence, pivot mask/flags, map replacement and lookup
narrowing, then runs seven Rust fixtures covering those rules and error cases.

The read-only mounted `native_body_mounted_groups` test (included in
`just test-native-dp-mounted reference/hazeron`) parses all64 native-selected
hand resources plus PA:168 groups,10,622 position keys,130 nonzero pivots in the
supplied installation. These are actual raw carrier checks, not original-client
execution or independent weighted deformation parity. Native object loading
extracts some point-only groups into object fields rather than retaining them
in its deformation list; the group-based scaling rules, complete skin assembly,
bone weights and renderer publication
remain required. No avatar body is rendered by this membership implementation.

## Native object control-point extraction

`NativeBodyObjectGroups` now applies LoadMeshChunk's ordered group selection.
Types3/10/11/12/13/14/19/20 write their respective object control points and
are removed from the deformation list. Type4 (MidPt) both writes its control
point and remains in that list. Other types remain in source order; type1
additionally sets the sticky object3a9 flag. Repeated writes replace coordinates,
including zero coordinates. Missing writes remain absent rather than inventing
constructor defaults. Only the established xyz portion of each copied native
vertex is represented; normals, UVs and bone fields are not inferred.

The mesh unweighted path uses an extracted ConnectPt as its origin, without
mutating the caller's definition. Point-only meshes can now use the verified
no-retained-group branch. The mounted65-resource fixture found103 retained
groups and65 connection writes, but ZERO point-only meshes: those actual hand
meshes still correctly return Unsupported. This is not rendered-avatar progress
or evidence of weighted deformation parity.

`just test-native-body-object-groups` checks the original executable digest,
all17 native switch destinations, separate type3 handling, nine destination
offsets, sticky type1 flag and MidPt copy-and-retain path, then runs Rust tests
for ordered writes, zero replacement, partial errors and mesh composition.
The complete previously retained REA record is
`docs/evidence/native-body-group-object-loader.json`; independent full GNU is
`docs/evidence/native-body-object-groups-gnu.txt`. No binary execution or new
analysis import was needed.

## Native per-vertex deformation selection

`NativeBodyObjectGroups::select_vertex` now implements ApplyScaleFactor's
ordered membership/pivot gathering. It narrows the lookup coordinates to f32;
membership, not the numeric weight, selects this stage. ScalePt, MidPt, Top and
Bottom cases precede anatomical flags. Each other group uses only its first
set flag in Torso/Hand/Foot/Neck/Tail/Head order. Later matching groups replace
their category's pivot; Wing and HandWing flags stay set across subsequent
groups. Wing substring matching is case-sensitive. Top/Bottom use explicit
initialized object scale-point overrides unless every component equals zero;
signed zeros count as zero, while NaNs select the override. The API requires
these values from its caller rather than inventing constructor defaults.

`just test-native-body-selection` binds static checks to the existing complete
ApplyScaleFactor evidence (`ev_413b8d0889ff3bece7696843079d27ff60f635084248ddc2dc5bb4eba8c1bbb0`)
and original PE digest, then exercises zero-weight membership, narrowing,
ordered replacement, flag precedence, sticky/case-sensitive wing flags,
overrides and absent/NaN keys. No new REA import or query was performed.
This collects inputs for the still-required weighted arithmetic, not a uniform
fallback, complete skin assembly or renderer publication.

## Anatomical CPU deformation: first connected branches

`NativeBodyDeformation::apply_vertex` implements Tail, Head, Neck and Foot
branches from the retained ApplyScaleFactor record and GNU. Native precedence
is Tail > Head > Neck > Hand > Foot > Torso; an unsupported Hand cannot fall
through to Foot. Head scales all axes with object180; Tail/Neck scale XZ with
180 then Y with188; Foot scales all axes with180 then Y with188. Both vertex
and connection undergo these operations about raw group pivots. Uniform140
scaling happens first, except zero skips input/connection arithmetic entirely.
Even unchanged axes preserve native subtract/add cancellation within taken
branches. The shared anatomical exit optionally applies178 to the connection
alone when Top AND Wing matched, then returns (offset - connection) + vertex.

`Mesh::body_deformed_positions` now connects existing coordinate loading,
ordered object metadata extraction, per-vertex membership selection and these
arithmetic branches. Callers must supply actual initialized definition fields
and top/bottom defaults. Extracted control points override those defaults.
Unsupported vertices fail the whole mesh result instead of publishing a
partially deformed or uniformly substituted mesh. Seven Rust fixtures cover
the four branches, precedence, connection-only correction, zero/NaN inputs,
cancellation and full mesh composition; `just test-native-body-deformation`
also verifies digest-bound static branch/final-correction anchors against
the existing evidence. No reimport, query or reference-client execution occurs.

Hand, top/bottom-only and other matched branches remain Unsupported.
Model definition generation, complete part assembly, bone weights and live
renderer publication remain required: these APIs do not yet render avatars.

The same connected mesh path now supports Torso: XZ use object170, Y uses178,
independently of limb180/188. Both vertex and connection receive the torso
transforms. After a non-unit178 stage the native exit goes through5cd, so
Top+Wing membership can apply178 to the connection a second time, around the
Top pivot. Tests explicitly distinguish this from a single height transform,
poison unused limb fields with NaNs, check native group priority and exercise
the Torso branch through raw mesh classification/loading. The static verifier
checks the entire torso block14139724d..141397371 against the original PE,
including absence of limb field reads, XZ/Y multiplication and the shared exit.
These checks reuse the same complete ApplyScaleFactor evidence; no new import,
reference-client execution or avatar-rendering verification is claimed.
