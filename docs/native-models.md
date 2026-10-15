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

HandWing remains Unsupported; later sections extend the other matched branches.
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

Non-wing Hand now uses its native uniform180 then Y188 operations on both
vertex and connection, followed by the shared connection correction. Native
Hand priority over Foot/Torso is retained. A separate Wing match does not make
it HandWing; actual HandWing membership still fails Unsupported. Tests cover
this distinction, native priority and the raw mesh path. Static checks pin
all six Hand uniform multiplications and its188 load/continuation to the same
digest-bound retained routine; no new analysis import was performed.

The mounted65-resource fixture now additionally runs the connected deformation
API with explicit non-unit test parameters (NOT generated Person definitions).
All65 sampled hand meshes succeed, and11,699 output points differ from the
unweighted branch. The test checks finite, count-preserving output and requires
some genuine grouped deformation. This demonstrates mounted-asset CPU pipeline
coverage, not native runtime parity, complete avatar generation or rendering.
Wing and top/bottom-only branches, live definition generation, assembly and
renderer publication are still required.

Top-only/non-anatomical deformation is now connected too, including Top taking
priority over Bottom when both match. MidPt membership averages170+180 and
uses the scaled midpoint; object2c4 is used unless all-zero, then228. Explicit
midpoint defaults are required and serialized writes override them. Without
MidPt, the native COMISD/JBE comparison selects170 when180 is below/equal OR
unordered, or when Wing is present; otherwise it uses the average180+170.
That latter branch still performs arithmetic when the average equals1, unlike
the midpoint branch's unity shortcut.178 then transforms the vertex and the
connection independently around Top, even when Wing is absent.

Four new fixtures cover scale selection, midpoint fallback/override, exact-one
cancellation and the unordered-comparison distinction that pseudocode alone
obscures. Static checks pin original branch destinations and midpoint addition/
half multiplication in the retained routine. This extends CPU deformation,
not complete avatar generation or rendering. Bottom-only and HandWing paths
remain Unsupported, and live model definitions/assembly/publication are pending.

Bottom-only and default non-anatomical membership now follow their native
branches. Selection preserves initialized Top scale coordinates separately from
Top membership. Bottom first shifts the scaled midpoint and vertex by178 when
Wing is absent, chooses180 or the170+180 average with original COMISD/JBE
unordered behavior, then applies188 about Bottom with midpoint compensation.
The connection takes178 only for non-Wing vertices without matching Top.
Default/ScalePt-only matches skip Bottom arithmetic but retain this distinct
connection exit; they are not silently treated as absent membership.

Fixtures cover Bottom through the real raw-mesh pipeline, midpoint motion,
Wing bypass, unordered scale choice, zero-scale nonfinite inputs and unmatched
Top scale retention. The static verifier pins the Bottom block's scale reads,
branch destinations and shared exits using the existing digest-bound evidence.
Only HandWing remains Unsupported in the CPU deformation dispatcher. This is
not full model generation: live definitions, part assembly, bone weights and
renderer publication are still pending, and avatars are not yet visible.

HandWing now has its own connected arithmetic module too. At native Hand
priority it moves the scaled midpoint and vertex by178, scales vertex and
connection XZ by180 around Hand, applies188 around Bottom with midpoint
compensation, then applies190 around Hand with scaled HandWingPt compensation.
The final correction preserves native operand order, including190 on the left
of the Wing-point Y delta. Top membership can additionally transform the
connection at the shared exit. Selection retains initialized Bottom scale
coordinates even without Bottom membership; extracted HandWingPt overrides
the caller's explicit object defaults. Zero140 skips nonfinite input/control
point products but not subsequent branch arithmetic.

Fixtures cover control-point extraction through the raw-mesh path, nonzero
Wing-point compensation, native priority, initialized Bottom without membership,
zero scale and NaN190 behavior. Static checks pin180/188/190 reads, arithmetic
and the shared exit in the existing complete digest-bound evidence. All normal
CPU deformation branches are now represented; unsupported key ordering or
malformed inputs still fail explicitly. This is not original-client execution
or a claim of native floating-point payload parity. Live model definitions,
complete skin assembly, bone weights and rendering remain unfinished.

## Model definition to object geometry

`NativeBodyModelGeometry` represents only the established geometry portion of
Au3dsGLSLModelDefinition, not an invented replacement for the complete Qt type.
It preserves the constructor's first six doubles at00..28, AuPoint at40..50,
and five following doubles at58..78. `apply_to_deformation` follows the actual
cache forwarding and generator writes:

| Definition | Deformation object |
| --- | --- |
| 08 | 140 (uniform scale) |
| 40/48/50 | 1c0/1c4/1c8 (f64 narrowed to f32) |
| 58 | 178 |
| 60 | 170 |
| 68 | 188 |
| 70 | 180 |
| 78 | 190 |

Existing connection/midpoint/wing control points remain unchanged. Translation
preserves native float narrowing, including signed zero and overflow; unused
definition scalars are not interpreted as display scale. Other generator
writes (object148/group48, bone weights, colors and device objects) remain
outside this geometry projection and are not reported as implemented.

Complete evidence is retained in `native-body-definition-constructor.json`
(`ev_32db6032543e537730818b45049fa7e949c76e124752809a9e1221103abe8b1e`),
`native-body-cache-generation.json`
(`ev_980dd5bc2a83b8aa63ce6dbf99ce4dc7bfc9da7a34e4d275eb90eb0ae9364f21`),
and `native-body-instance-generation.json`
(`ev_1c0dbd30bfa2b18a76c92dcad5cf14e063d5a2853011077841967eeee448341d`).
The instance record was reused; the two missing procedures used the already
analyzed warm session in0.112s and0.292s, with no new import. `just
test-native-body-definition` binds all records to the original PE digest,
checks constructor layout and actual wrapper loads/stores with GNU, and tests
mapping, metadata preservation, float narrowing and mesh composition. The
mounted hand fixture now applies this projection before deformation using
explicit fixture parameters, not live Person definitions. Production part
CreateSkin definitions, full assembly/bones and avatar rendering remain pending.

## Initialized arm/hand skin geometry requests

`NativeDpPartState::arm_skin_geometry` now reconstructs the geometry requests
from DpArmAndHand::CreateSkin: actual Bones out-reference order, literal
first-bone side mask, owner prefix/PA selection, part-DNA arm/hand/wing variants,
constructor geometry scalars and stored connection. Ordinary arms request Arm
then Hand; wing selectors2..7 request Wing only. The Hand translation uses
(body40b4 + connectionY) + body4088 in native order; its definition68 is replaced
with the doubled float Hand factor, not definition78. Uniform scale divides
owner SizeFactor by0.84 in f32 before widening; limb factors retain their actual
float/double stages and separate double multiplications.

`NativeDpInitialization::arm_skin_geometry_requests` requires completed part
calculations and obtains actual owner DNA/dimensions and the selected part.
The caller must supply real body4120..4138 fields. Wrong part kinds fail
Unsupported; incomplete initialization fails InvalidData. This is a geometry
projection, not complete CreateSkin: material/color parameters, cache success/
failure behavior, bounds, model publication and bone weights are not fabricated.

Complete static evidence is `native-dp-arm-create-skin.json`
(`ev_a185d7368b4d96d25c37384b278ad798ecbd1bd756ea6e0deeec1c60516678f8`),
with full GNU in `native-dp-arm-create-skin-gnu.txt`; the reused prefix record is
`native-dp-body-model-prefix.json`
(`ev_587e4a153a7a8d64ebffce5b9e797df65ba2d41cc4237b587891da95c62757ff`).
The new query took0.197s on the existing warm analyzed session; no import.
`just test-native-dp-arm-skin` verifies the original PE digest, actual scalar
constant bytes, call/translation/Hand68 anchors, prefix bytes and bone-side
mask. Rust fixtures cover both sides, all selectors, all three pairs plus
native index fallback, PA modes, source precision and initialization failures.

The mounted fixture initializes an actual one-arm-pair body using its mounted
HandLength provider, selects four Arm/Hand resources and runs their940 vertices
through definition projection and CPU deformation. Its metadata defaults are
explicit fixtures, not reconstructed object-constructor defaults or a claim of
native/rendering parity. Leg/torso/head/tail skin generation, full model state,
bone weights and renderer publication still remain under the original goal.

## Initialized leg/foot skin geometry requests

`leg_skin_geometry` and the completed-owner wrapper now produce ordered Leg
then Foot requests from actual part DNA, owner prefix/dimensions and supplied
body4120..4138. Leg variants use DNA12 bits4..5; Foot variants use DNA10 high
bits, except PA uses the actual HasAnkleBend predicate. The first Bones output
and literal0xfbefbfffffffffff mask choose the side. Invalid leg pair indices
leave native outputs unwritten and remain Unsupported, unlike arm fallback.

Geometry factors use the verified DNA15/16 high bits for58/60 and DNA2/3 high
bits for68/70, retaining native float stages and subsequent double division/
multiplication. The Foot replaces68 with the doubled float foot factor and
translationY with (body409c + connectionY) + body4080. Disassembly supplies1.0
in constructor XMM1 and forwards4120 with MOVSD: the decompiler's literal0 and
integer cast are not copied into Rust. Owner uniform scale still divides by
0.84 in f32 before widening. Other definition values and bone order survive.

Complete evidence `native-dp-leg-create-skin.json`
(`ev_8d30ccd22ac7761832a6d7e056724355aa0c81029ef3b3ecaa1600ff648c27de`)
and full GNU `native-dp-leg-create-skin-gnu.txt` are retained. The query reused
the warm analyzed session in0.252s with no import. `just test-native-dp-leg-skin`
checks digest-bound call/resource/precision/attachment/ABI anchors and fixtures
for both sides, all pairs/variants/PA foot selectors, field mapping and errors.
This is the geometry projection only: native CreateSkin also invokes FootLength
for cache side effects, materials, bounds and actual model creation/publication;
those effects are NOT reported as implemented by the request builder.

The mounted initialized-body fixture now includes one arm pair and one leg
pair: eight selected Arm/Hand/Leg/Foot resources and1,758 CPU vertices pass
through definitions and deformation. Object metadata defaults remain explicit
fixture inputs, not reconstructed production defaults. Torso/head/tail requests,
full skin assembly, bone weights and actual avatar rendering remain unfinished.

## Initialized tail skin geometry request

`tail_skin_geometry` and the completed-owner wrapper now produce the native
optional Tail request. DNA14 bits4..5 zero mean no geometry; bits6..7 choose
Tip or Tail1/3/5, and the suffix is bits4..5. Unlike limbs, the resource prefix
comes from part DNA16 low bits and never PA/body4068. All three bone references
are17. The definition retains the initialized tail attachment and owner4120..
4138; uniform scale uses the owner's SizeFactor. Height divides the DNA19
float factor by1.5 in f32 before widening. Width uses ordered float products
of DNA20 low bits and that length, adds length/3 in float, divides by0.875 in
float, then widens. These are not the limbs' double-division formulas.

Complete evidence is `native-dp-tail-create-skin.json`
(`ev_89a93f737697907ef0718e90add2c5e915053e46299e99e7c889c82774bc4171`)
and full GNU `native-dp-tail-create-skin-gnu.txt`. The query used the existing
warm session in0.139s with no import. `just test-native-dp-tail-skin` checks the
original digest, disable/segment gates, resource rules, bone17 arguments,
source float constants, division/widening instructions and constructor ABI.
Fixtures exhaust prefix/type/segment combinations and length/width factors,
distinguish float from double division, and check wrong-part/incomplete-owner
errors. The decompiler's apparent constructor literal0 is corrected from the
actual XMM1 load of1.0, as for Leg.

The initialized mounted-part fixture now selects nine Arm/Hand/Leg/Foot/Tail
resources and deforms2,016 vertices. Its object metadata defaults remain
explicit test inputs, not reconstructed production defaults or runtime parity.
This request builder does not execute full CreateSkin cache/material/device
effects. Torso and head/neck generation, production object defaults/weights,
full assembly and avatar rendering are still required.

## Torso segment selection and skin geometry plan

`native_dp_torso_segment` now ports the bounded wrapper and part.0 selector.
It uses actual WaistVertex/ArmVertices, adjusted wing arms, available leg
positions and tail endpoints to index the native6x9 table. Original PE bytes
confirm consecutive u32 values1..54; out-of-range segment indices return0.
Fixtures cover literal simple-body selections, wrapper bounds and packed
arm/leg/wing/tail combinations without replacing selection with limb counts.

`torso_skin_plan` and the completed-owner wrapper preserve native two-stage
generation: a combined-model key first, then ordered fallback segment/crest
pieces if combined-model creation fails. Segment codes choose the exact native
resource stems, including literal ModelMissing names. PA suppresses crests;
otherwise crest selectors enter both the combined key and optional per-segment
resources. Fallback bone triples preserve previous/current/next indices, and
translation decrements by the owner's stored40a8 per accepted segment. The
combined definition retains native geometry scalars, including40a8 as its
first double (not guessed1.0); float length/width stages and merge tolerance
(width*0.05)*uniform remain explicit. Zero part segments produce no plan.

Retained complete records are `native-dp-torso-create-skin.json`
(`ev_bd95f62b443a571fce7c7f6b5a209ab557397d1a065fbebe584c2c6c7b946bd5`),
`native-dp-torso-segment.json`
(`ev_e8d3236422368ba6e4d43e57c78845c441506305879017a53d38a1b41f53815e`)
and `native-dp-torso-segment-part0.json`
(`ev_2810175bdb9ccaf85137c4ba62b767b9004a1f16aa681ff8ccf530dad0636441`).
Full selector GNU is `native-dp-torso-segment-gnu.txt`. The skin record was
reused; both missing selectors used the warm session in0.144s/0.163s, no import.
`just test-native-dp-torso-skin` verifies original table/constant bytes, input/
definition/combined/crest/fallback anchors and Rust ordering/PA/error fixtures.

The mounted initialized-body fixture now exercises11 Arm/Hand/Leg/Foot/Tail/
Torso fallback resources and2,463 CPU vertices. It deliberately tests the
fallback pieces, not a successful combined-cache path. Object defaults remain
explicit fixtures. The plan does NOT implement cache lookup, failed-load
semantics, MergeTorsoWithInstance, CreatePatchedModel, bounds/materials or
publication. Head/neck requests, production defaults and weights, full model
assembly and avatar rendering remain unfinished under the original goal.

## Constructor-established deformation defaults

`NativeBodyDeformation::native_initialized` now reconstructs the actual
Au3dsGLSLObject constructor's deformation subset:140/170/178/180/188/190 are
double1.0, and translation, connection, midpoint, midpoint-scale, HandWingPt,
TopScalePt and BottomScalePt XYZ fields are positive float zero. This is not
blanket zero initialization of the native object or a complete GL model type.
Raw `NativeBodyObjectGroups` still distinguishes absent writes from zero
writes; constructor defaults belong to the object state, not its input stream.

`Mesh::body_defined_positions` composes the verified constructor geometry,
definition projection, serialized group metadata and CPU deformation. The
mounted initialized-part test now uses this API, removing its handwritten
metadata defaults. Supplied body4120..4138 test values remain explicit, and
the torso fixture still exercises fallback pieces rather than merging them.

Complete constructor evidence is `native-body-object-constructor.json`
(`ev_df29903cd2d7019d4ccdeb46d7cafaabafcc6a3695cb67e32a606eef86b5620c`),
with full GNU in `native-body-object-constructor-gnu.txt`. The actual52-byte
vertex assignment is retained independently in `native-body-vertex-assignment-gnu.txt`.
The query reused the warm session in0.127s without an import.
`just test-native-body-defaults` pins the original digest, both unity doubles,
scale writes, seven XYZ zero fields and assignment copy instructions. Rust
fixtures check positive-zero bits, constructor/definition/control-point
composition, unchanged input and explicit malformed-group errors.

These establish geometry defaults only, not device/material/bone defaults or
original-client runtime parity. Head/neck generation, torso merging, complete
weights/model assembly and actual avatar rendering remain unfinished.
