# Production native model application: remaining evidence gates

The diagnostic presentation bridge does not complete the avatar goal. The next
work is actual model generation/application so that the production receive
context can implement its required operations rather than use fixture callbacks.
Keep supplied installations/binaries read-only and never contact public servers.

## Selected unresolved batch

Use one connected REA target/provider for the patched OpenShores.exe SHA256
285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80.
The initial new queries are:

- DpBody::SetDNA(DhDNA const&, float),1412f1fb0: actual model-part construction,
  DNA-to-model mapping, resource selection, transforms and invalidation writes.
- DpBody(char,bool,DhDNA const&),1412e24a0: ordering of initial model construction
  relative to SetDNA and the constructor defaults already retained separately.
- AuAnimalia::SetDNA(AuDNA const&, CHROMATID),1411ed650: chromosome-driven part
  allocation/reuse, geometry/pose dependencies and cleanup/invalidation effects.
- AuChromatid::Volume(),141216980: actual chromosome virtual128 accumulation,
  not rendered mesh bounding-box volume.

These are queries to review, not declarations that model generation is complete.
The connected-tool attempt in `target/native-model-analysis-3` was cancelled
after the client's 300-second deadline; its bundle is empty and is not evidence
for any function body. The replacement single raw-stdio session retains complete
responses under `target/native-model-analysis-4` as results arrive. Its queued
batch includes these four procedures and all eight distinct chromosome Volume
implementations below. Continue the same provider
for the constructor, animation, part-generation and volume dependencies that
the returned bodies actually call. Do not redo earlier successful reader,
constructor, pose or scalar queries.

## Independent volume dispatch findings

Original PE bytes at the 28 factory-confirmed chromosome vptrs from
native-chromosomes.md establish virtual128 Volume dispatch. Twenty types use
AuChromosome::Volume at141900f50; its instructions return positive float zero.
`just test-native-chromosome-volumes` pins all28 slot bindings and chromatid
f32 accumulation to the original PE digest; it is not a formula/model test.
The distinct volume implementations are:

| Type | Class | Address |
| --- | --- | --- |
| 8 | Head | 141205b90 |
| 9 | Torso | 141205d30 |
| 10 | Arm | 141205e70 |
| 11 | Leg | 141206110 |
| 17 | Ear | 1412065e0 |
| 19 | Nose | 1412066d0 |
| 20 | Mouth | 1412067f0 |
| 27 | Mammary | 141206910 |

`native_dna/volume.rs` and its face/limbs modules now implement all eight
overrides from independent GNU disassembly retained in
`docs/evidence/native-chromosome-volume-gnu.txt`,
`docs/evidence/native-face-volume-gnu.txt`, and
`docs/evidence/native-limb-volume-gnu.txt`.
Head reads stored shorts1e/20/22 with double scaling then float multiplication;
Ear uses short12 and preserves division after two float products. Mammary reads
short12, early-outs for zero or field0c==1, and otherwise uses ordered
double/float scaling and cubing. The twenty shared base methods return positive
zero. Torso preserves the ordered ellipse/length contributions; Arm/Leg preserve
raw-short equality branches and their actual cone subtraction (the larger
radius's area is used for both cone terms). Nose preserves its distinct mixed
double/float path; Mouth narrows only its second summand before the double add.
Ordered chromatid summing now handles all factory-decoded chromosome kinds,
with errors for malformed Rust field storage or non-factory type identifiers.
This does not choose the model's chromatid or establish wing capabilities.
Tests exercise wire-decoded records, unsigned-short extremes, exact float bits,
Mammary suppression, ordered sums and malformed storage. The static verifier
also pins fifteen consumed constant bit patterns to the original PE.
Preserve exact conversions and sum order, not simplified geometric formulas.

## Required implementation and verification

### Model-part construction evidence

`docs/evidence/native-model-part-dispatch-gnu.txt` independently establishes
AuDNA::MakeParts at141228890: select the requested chromatid, seed one shared
byte bone index to zero, iterate chromosomes in order, call virtuald0 with that
chromatid and mutable index, and append only nonnull results. The source list is
not reordered or filtered to volume-bearing chromosomes. `just
test-native-model-parts` checks every factory vptr slot against the original PE:

| Kind | Part generator | Address | BoneLoc slot48 |
| --- | --- | --- | --- |
| 8 | Head | 1412261a0 | 1412193e0 |
| 9 | Torso | 141225fe0 | 141205c20 |
| 10 | Arm | 141226370 | 141215a30 |
| 11 | Leg | 141224730 | 141215df0 |
| 12 | Fin | 141225e00 | 141206450 |
| 13 | Tentacle | 141225a50 | 141206480 |
| 14 | Wing | 141225c20 | 1412064d0 |

All other kinds dispatch to the complete null-returning base MakePart at
141205af0. This differs from Volume: Ear/Nose/Mouth/Mammary contribute volume
but no standalone part; Wing/Fin/Tentacle create parts despite zero Volume.
Do not derive the part list from nonzero volume or invent separate facial parts.

Wing MakePart clones its chromosome, captures the pre-increment bone-index byte,
then invokes GenBones; it does not load a mesh at this stage. Torso has the same
clone/GenBones structure. Their complete geometry, pose and rendering consumers
still need reconstruction. Wing::CanFly at1411fcd50 delegates through its cloned
chromosome at+f8 to AuChromoWing::CanFly at14121e550, whose full body returns
true. This establishes capability for an actual generated Wing, not that any
unresolved part allocation/generation may be replaced by a fake Wing marker.

The same live analysis session also queues MakeParts, AuDNA equality, the
Animalia DNA constructor, the seven MakePart overrides, Wing CanFly, and the
four nontrivial BoneLoc methods. Do not resubmit these while queued or completed.

### DpBody allocation and retained initialization evidence

`native_body::native_allocate_dp_parts` implements DpBody::Init's allocation
phase: Torso, optional Tail (DNA byte14 mask30), one/two HeadAndNeck parts
(byte5 bit1), ordered left/right arm pairs (byte1 low2), then left/right leg
pairs (byte12 bits2..3). Every part owns its DNA copy and actual head/pair index.
When multiple arm pairs and byte2 mask6 are set, all but the last pair clear
their own byte2 low3 bits; the body's original DNA is not changed.
The 65,536 count/wing-byte combinations and every optional count byte are tested.

This phase is not a complete body constructor. Init continues through bone
matrix reset, body OneTimeCalcs and part virtual10 OneTimeCalcs. Those methods
are now retained in `target/native-model-analysis-4`, along with SetDNA and
AnimatePose. DpBody::Init Evidence is
`ev_5ac6fc428ff8926b29782cbbd39115ee7f7c4e15713693bb2e110319920e20c6`,
corroborated by complete GNU assembly in
`docs/evidence/native-dp-initialization-gnu.txt`.
The five part calculation overrides are Torso (actual no-op), Tail,
HeadAndNeck, ArmAndHand (both sides), and LegAndFoot (both sides).
No no-op is substituted for the four nonempty calculations.

The next phase is implemented as `NativeDpDimensions::from_dna`: all fifteen
f32 stores at4080..40b8 from body OneTimeCalcs, backed by the literal32-entry
SizeFactor table and actual DhDNA ArmVertices/WaistVertex helpers. Assembly
confirms that the two1.3 multiplications widen to double then narrow, unlike a
straight all-f32 interpretation of the decompiler expressions. The code keeps
that sequence, the width-table branches, and the multiplication order.
`just test-native-dp-dimensions` binds the Rust table and scalar constants to
the original PE; Rust tests cover exact-bit15-field fixtures and packed flags.
Evidence is retained in `native-dp-dimensions-gnu.txt` and
`native-dp-layout-gnu.txt` under docs/evidence. REA records for SizeFactor,
ArmVertices, WaistVertex and HandLength are already retained in session4.

Part OneTimeCalcs is still required. In particular ArmAndHand calls the actual
DpBody::HandLength(false): that method uses the geometry cache, model-DNA key,
body-model prefix, mounted HandRight*.3dsN geometry extents and vertex/group
information. A guessed hand length or empty geometry cannot complete this
dependency. GetBodyModelPrefix and DhDNA::CreateModelDNA have been requested in
the same warm session; inspect their retained files instead of querying again.

`NativeDpModelDna` now implements the zeroed48-u32 Au3dsGLSLDNA constructor,
all47 DhDNA::CreateModelDNA selector writes, retention of the finalbc field,
and the actual unsigned48-field comparison priority. Native key ordering is
not memory-order lexicography. `just test-native-dp-model-keys` independently
derives every bit selector and comparator offset from GNU instructions and
compares them with the Rust tables. REA constructor/selector/comparator Evidence:
`ev_377c1f0859d7d7580e718cbe349a3caadec66d7262773dbe8c2e972503c07d54`,
`ev_9a998979e39c535e1a51c68be052b7cb0b42e9599ef03f70f3fdbd3813cb7030`,
`ev_ffbfeb0d0a286808ceef6514d6f345945412396929bcd8079bad5c8daa4c9802`.
Complete GNU corroboration is in `docs/evidence/native-dp-model-key-gnu.txt`.

`NativeDpHandResource` selects the actual resource and cache key using supplied
body4068 and HandLength's boolean. Full32-bit4068 values41/50/99/9b select
`:data/PAHandRight.3dsN`; all others use `:data/{DNA16&7}HandRight{DNA11>>5}.3dsN`.
The PA false-mode override sets keybc=1 (stack17c minus keybasec0), not keyb4;
the boolean does not select a different filename. Its read method resolves only
that exact Qt resource through the user's AssetMount and bounded raw N decoder.
Missing resources remain errors, with no basename or primitive substitute.
GetBodyModelPrefix Evidence:
`ev_587e4a153a7a8d64ebffce5b9e797df65ba2d41cc4237b587891da95c62757ff`.

`just test-native-dp-mounted reference/hazeron` passes read-only/offline against
the supplied installation:64 numbered hand resources decode, none missing,
11,573 raw vertices, plus the PA hand. The Rust integration test is opt-in and
requires an explicit installation path; ordinary tests do not depend on local
reference material. This is resource/decode verification, not hand-length,
mesh deformation, animation or rendered-native-avatar parity.
HandLength's apparent decompiled void return is contradicted by its actual
`cvtsd2ss` and `movaps xmm6,xmm0` return path; use the float return when porting
the retained geometry/cache calculations, not the recovered signature.

`NativeDpHandGeometry` and `NativeDpHandCache` now provide that mounted
HandLength dependency. Bounds include native TestForExtents-eligible objects;
the first object retained after station/weapon/up-vector removal supplies its
ConnectPt and first Hand-tagged group pivot. ConnectPt uses the final serialized
point. The Hand-tagged group does so only on the multi-record path: the native
single-record fast path does not assign a Hand-flagged anchor. This is not a
weighted centroid. Group names have ordered, case-sensitive classification;
ScalePt/ConnectPt/MidPt/BottomSkin/Torso and exact special names precede Hand.
Raw vertex-group name/member chunk order is now retained by the decoder and
respected by metadata assignment. Raw records, source bytes and groups remain
unchanged; unknown/missing group order in manually constructed storage errors.

Hand arithmetic keeps the native f32 size/0.84 division, double multiply/divide
inside the broad factor before narrowing, initial double coordinate scaling,
unscaled pivot, conditional broad/width stages (including non-Y subtract/add
round trips), and cached `[0,maxY-connectY,connectZ-minZ]`. HandLength returns the
cached Y narrowed to float. The mounted cache uses the verified model-key order
and treats an all-zero offset as a cache miss, as native does. It is tied to one
borrowed mount, with a20,000-entry Rust safety budget; switching installations
requires a new cache. Missing geometry/errors propagate without a fallback.
Screen-bearing first objects or models without extent-bearing/retained objects
remain explicit Unsupported rather than fabricated metadata.

Full independent HandLength assembly is retained in
`docs/evidence/native-dp-hand-length-gnu.txt`. Additional matching REA results
are retained in the same warm session directory: `glsl-vgroup-loadchunk.json`
(name priority/flags), `glsl-vgroup-loadvertices.json`
(`ev_ee5907c272b7213258a4647ea21954cbfc8217d4be3d3d67e1a1cebe0c8bcf08`),
`glsl-object-loadmeshchunk.json` (group transfer/object bounds),
`object-test-extents.json`
(`ev_92a28710d471af6e6808fc4a47ee34036bb14434d07c20a762af7ca9120132ac`),
`glsl-model-loadeditor.json` (bounds aggregation), `glsl-model-load.json`
(`ev_8fb5472cb9b6fe59577bb644df4e7dc5d9151d73c31b9df5591f59e2cfe33057`),
`modelcache-creategeometry.json`, and `modelinstance-loadmodels.json`.
The raw HandLength record was already retained; it was not decompiled again.
Selected SSE instruction ranges corroborate numeric widths where the decompiler
loses them. All of this is static reconstruction, not original-game execution.

Synthetic tests cover precedence, single/multiple points, reversed chunk order,
first-object anchors, excluded helpers and exact offset bits. The mounted test
now also computes and repeats HandLength for all64 numbered resources and PA,
checking finite nonnegative results and stable cached bits. This tests actual
user assets in Rust, not native runtime or rendering parity. Body initialization
now consumes this provider in the arm part OneTimeCalcs phase, with the other
three nonempty part calculations reconstructed as described below. Real
poses/animation and production dispatch integration remain required.

### Dp part calculation state

`NativeDpInitialization` composes the existing allocation and dimension readers
with Torso's no-op and the Tail, HeadAndNeck, ArmAndHand and LegAndFoot
OneTimeCalcs. Connection and joint coordinates retain their native offsets in
the public state names. The implementation preserves mixed float/double stages,
the unsigned arm-index subtraction, signed leg-index division and negative zero.
The retained `dp-{torso,tail,head,arm,leg}-onetime.json` records in
`target/native-model-analysis-4` are the static reconstruction evidence; these
are not original-game runtime observations.

Arms call the concrete mounted HandLength cache with the owner body's original
DNA, even when allocation cleared wing bits in an individual arm's copy. Missing
resources propagate an error after the native pre-call connection/elbow/wrist
writes; no synthetic hand point is published. The Rust progress cursor retries
the failing part on resumption and does not recalculate completed parts. That
cursor is orchestration state, not a claimed field from the native executable.

Offline tests cover no-arm bodies, arm partial writes and retry, bilateral
coordinates, duplicate heads and signed zero. The opt-in mounted test initializes
a sixteen-part body through this public API, verifies the actual mounted hand
offsets and repeated completion. These tests do not establish native render
parity. This is deliberately an initialization phase, not a complete DpBody:
remaining constructor state, pose/animation, full mesh
generation and production Person context still need implementation.

### Dp bone transform initialization and animation reset

`NativeDpInitialization` now owns all61 primary transform slots, matching
`DpBody::Init`'s full identity reset at body offset0x70 with stride0x108.
`NativeDpBoneTransforms::reset_for_animation` separately implements
`InitBoneMatrices`: DNA selects spine, neck, tail, arm and leg ranges, plus the
fixed slots, while inactive slots retain their previous contents. The primary
matrix's sixteen doubles and the byte at slot+0x100 are the represented writes.
The reset never initializes native bytes0x80..0x100, so this Rust phase does not
invent a second identity matrix there or claim to model those untouched fields.

Complete static REA Evidence is retained in
`docs/evidence/native-dp-bone-reset.json`, including target SHA-256,
provider/profile and limitations (Evidence
`ev_416f549167537f850d27a95edbde05f7071c0b975c573dc06a04fb0b97ff0395`).
Tests cover all4096 combinations of the six count selectors against native
body-offset ranges, including inactive-slot preservation and exact positive-zero
identity bits. This implements reset state, not skeletal animation or rendering.

The same still-live analyzed session supplied the next pose dependencies without
reimport: successful queries took0.084–0.150s each. Retained files now also include
`dp-position-{torso,heads,tail,wings,arms,legs}.json` and
`dp-cg-height-const.json`. These are evidence for the next implementation, not
claims those operations are ported. The initial non-const CGHeight symbol was
rejected; its error remains in `dp-cg-height.json`, and symbol inventory selected
the actual const method. The torso decompiler's integer return requires checking
the native float return register before translating it.

All previously queued queries completed successfully after the single672.161s
import; complete results remain in that directory. Their successful retention
does not imply every function is reviewed or ported. In particular AuDNA equality
has a decompiled void return and needs return-register/child equality evidence;
DpBody::DeleteBodyParts resolves a wrapper whose apparent recursive name must
not be copied as a recursive Rust implementation. Preserve raw limitations and
follow the actual target addresses when resolving those dependencies.

### Mounted feet and standing-height dependency

`NativeDpFootResource` selects the exact numbered FootRight resources, or PA
FootRight0/1 using `HasAnkleBend` (DNA byte10 masked with0xa0), from the full
body4068 value. FootLength's mode affects the PA key, not resource selection;
FootHeight always uses mode=false. Both measurements share a mount-owned offset
cache with the native all-zero miss sentinel and complete model-DNA keys.
Missing or unsupported geometry propagates an error instead of native loader
failure's zero fallback. The20000-key limit is a Rust safety budget.

Foot metadata uses the first retained object's first Foot-flagged group, not
Hand, retaining name/member chunk order and the single-member fast-path behavior.
The verified common extent/scaling pipeline is shared with HandLength, but feet
use DNA bytes3,2,10 in place of8,6,11. Numeric stages follow SSE, including the
double intermediates absent from the decompiler's broad-factor expression.
The cache stores [0,length,height]; FootLength returns Y and FootHeight returns
Z as f32, despite the decompiler reporting void for both.

`NativeDpFootCache::standing_height` composes the mounted measurements with the
actual initialized dimensions and ankle/class branches. No-leg bodies do not
request feet. Legged bodies always request FootLength, even when its result is
discarded, and eligible height-inclusive branches call FootHeight twice. The
native instruction order adds the two segment lengths before adding foot length;
the decompiler's differing association was not copied.

Full static records are retained in `docs/evidence/native-dp-foot-{length,height}.json`,
`native-dp-standing-height{,-part0}.json` and `native-dhdna-ankle-bend.json`;
`native-dp-feet-gnu.txt` corroborates the numeric widths, sums and float returns.
Offline fixtures cover all256 foot-selector bytes and runtime-field/mode cases,
pivot priority/chunk order, exact scalar offset fixtures, all standing branches
and error propagation. `just test-native-dp-mounted reference/hazeron` now also
decodes128 foot requests (64 numbered resources, repeated PA requests covering
both ankle variants), checks cached length/height bits and exercises standing
heights for all four classes. These are real user assets executed by Rust, not
native executable observations or proof of render parity. Full skeletal
animation remains an unimplemented dependency.

### Pose-specific body height

`NativeDpFootCache::pose_height` now consumes `NativeDpBodyPoseState`'s actual
fourth word (body offset0x48). It implements CGHeight's fixed-height, width-only,
neck-dimension, standing and scaled-standing branches. Only legged standing
branches call the mounted standing-height provider, with the original include
flag. Fixed/dimension-only/default branches neither require geometry nor
invent a standing result. Full-width unknown pose values follow the observed
native default rather than being truncated to a byte or categorized by name.
The neck lookup preserves its native comparisons and f32 operation order.

The138-entry fixture `tests/fixtures/native_dp_height_kinds.tsv` is decoded
directly from the SHA-bound native signed switch offsets and the two immediate
dispatch masks by `tests/native_dp_height.py`. `just test-native-dp-height`
checks that fixture, scalar constants and SSE comparisons, then runs the Rust
branch fixtures. Complete REA Evidence and address-bounded GNU corroboration are
retained in `docs/evidence/native-dp-pose-height{.json,-gnu.txt}`. The bounded
range avoids objdump's symbol selection including adjacent unrelated functions.
No analyzed executable session was restarted or repeated for this work.

Tests cover every byte pose, full-width default values, leg/class combinations,
include-flag propagation, geometry error propagation, neck selector boundaries
and MAXSS signed-zero/NaN selection. The mounted installation test additionally
exercises every byte pose with actual numbered/PA foot measurements and verifies
repeat results. This composes the pose-height dependency with existing body pose
state and real mounted geometry, but does not implement the remaining bone
position/rotation routines, body mesh generation or production Person context.

### Remaining integration gates

`NativeDpPartState::rotate_neck` now implements the concrete
DpHeadAndNeck::RotateNeck wrapper and nonzero path. It obtains the current owner
attachment before mutation, builds the native pivot/RotateX/negative-pivot
transform with the widened f32 angle, then premultiplies the inclusive head/neck
bone range. Index1 selects the second head; other indices retain native first
head selection. Only affected slot+0x100 bytes are cleared. Both signed zeros
skip the operation entirely, while NaN enters the native arithmetic path.
Non-head parts return Unsupported rather than borrowing a head bone mapping.

The scalar implementation retains the inline multiply's observed addition
grouping, rather than the decompiler's misleading reassociation. Its complete
static record is `docs/evidence/native-dp-head-rotate-part0.json`; wrapper and
instruction corroboration are in `native-dp-neck-rotation-gnu.txt`. The query
reused the live analyzed database (0.134s). Platform sin/cos and NaN payload
limitations from the underlying transform reconstruction still apply.

`just test-native-dp-neck` covers pivot preservation, inclusive ranges for every
count/second-head combination, out-of-range head-index native fallback, unchanged
neighbors/flags, zero/NaN paths and cancellation-sensitive multiplication.
The mounted sixteen-part initialization test additionally reads every concrete
attachment and rotates each actual initialized head around its attachment.
This is a real skeletal mutation, not full TiltHeadTo or PositionHeads:
head matrix assembly and body pose dispatch still require composition.
The retained `dp-head-bend-part0.json` supplies the later bending phase described
below. Production Person context and presentation remain
required gates below.

`NativeDpPartState::lean_neck` now implements the LeanNeck companion operation.
It uses the same current-attachment pivot and inclusive bone range, but calls
native RotateY rather than RotateX (or an inferred Z-axis lean). The shared Rust
path is justified by a static comparison of both complete nonzero routines:
all396 normalized instructions match after identifying their single differing
axis call. Zero-angle wrappers also match. The verifier preserves target symbol
names, local branch offsets, actual RIP-target comments and instruction operands;
it removes differing absolute branch addresses/RIP displacements, not arbitrary
constants or control flow. This is structural static evidence, not original-game
runtime verification.

`just test-native-dp-neck` now runs `tests/native_dp_neck.py` before fixtures.
Complete records are `docs/evidence/native-dp-head-lean{,-part0}.json`, with
wrapper/clone instructions in `native-dp-neck-lean-gnu.txt`. The wrapper's
decompiled apparent self-call is resolved by the actual jump to the nonzero
clone; no recursive Rust implementation was introduced. The existing retained
analysis was reused without another executable import or decompilation query.
Tests exercise both operations across head/count ranges, verify Y-axis signs and
pivot translation, zero/NaN behavior and unsupported non-head parts. The mounted
sixteen-part body test composes rotation and leaning on each initialized head.
The remaining full head/body animation gates still apply.

`NativeDpPartState::bend_neck` now implements BendNeck's distinct staged path.
Starting at the last neck joint, it projects the stored local connection through
the current joint matrix, constructs the expanded native pivot transform, and
applies it to the inclusive prefix of head/neck bones. It then advances local Y
by body40a8*0.5 rounded in f32 before widening, descends to the next joint, and
reprojects against already-mutated bones. A single pivot, forward traversal or
one shared rotation for all neck bones would not implement this routine.
The real owner dimensions are required, not inferred from the part's DNA copy.

Expanded coefficient/pivot arithmetic and literal zero products are retained,
including homogeneous-row writes, native addition grouping and flag clears.
Signed-zero input angles skip the whole operation; NaN enters the arithmetic;
non-head parts report Unsupported. Full static Evidence is retained in
`docs/evidence/native-dp-head-bend-part0.json`, with complete wrapper/clone GNU
corroboration in `native-dp-neck-bend-gnu.txt`. No new analysis import or repeated
REA query was needed. The neck static verifier now checks all three zero-angle
wrappers plus the mixed-precision spacing and descending-prefix loop anchors;
the attachment verifier also binds the non-torso concrete ConnectLoc slots.

Fixtures verify per-bone accumulated angles across both head selections/all count
combinations, unchanged neighboring slots, actual-owner spacing, zero/NaN and
unsupported paths, and exact dense nonaffine scalar bit patterns independently
evaluated from the instruction stages. Mounted sixteen-part initialization now
composes bending, rotation and leaning on each head and checks preserved current
attachments. This remains Rust/offline evidence, not native runtime parity.
Full body animation, production Person ownership and
actual rendered native avatar state still remain required work.

Concrete `TiltHeadTo` now composes the verified head operations in
`dp/head_pose`: capture attachment, reset the inclusive selected head/neck range,
restore attachment using the actual virtual MoveBy, extract the body-up pitch
correction for segmented bodies, RotateNeck, divide the remaining pitch plus
f32 pi/2 across the neck, BendNeck, and assemble the final head matrix using
the original pitch and roll around the projected local head point. Missing Rust
head-joint state is rejected before mutation; other part kinds are Unsupported.
The body-level PositionHeads dispatcher is not implemented by this operation.

`NativeDpInitialization::position_heads` now implements that separate stage
for calculated body parts. It consumes the actual body pose words at40/54
(indices1/6 in the existing pose state), explicit body pitch30/roll34, the
original body DNA and the required native rhythm timer interface. Both pitch
selections run in order before any matrix mutation, even with one head. The
one-head branch tilts the first concrete head and returns; the two-head branch
tilts both in part order, then leans them apart with the native f32 spacing.
This composes real head/neck transforms rather than returning pose metadata.
Calling before part calculations complete is a Rust InvalidData safety error.

`docs/evidence/native-dp-position-heads.json` retains the complete existing REA
record, corroborated by `native-dp-position-heads-gnu.txt`; no reimport or new
query was needed. `just test-native-dp-position-heads` verifies both 53-entry
native jump tables, four additional float constants, and all seven virtual
IsHeadAndNeck bindings, then runs head-pitch fixtures. Full-width unknown poses
retain native default behavior, not byte truncation. Mounted-body fixtures
compare complete bone state with the ordered tilt/lean composition, exercise
distinct rhythm phases and timer order, and check that a second-pose timer
failure preserves all bones in both one- and two-headed bodies. Complete
Animate, production timer ownership, Person context and renderer publication
are still required; this stage does not claim those are implemented.

Retained complete REA records are `docs/evidence/native-dp-head-tilt.json` and
`native-dp-head-move-by.json`, with full GNU disassembly retained in
`native-dp-head-pose-gnu.txt`. MoveBy's zero guard and expanded non-affine
translation were corroborated against GNU disassembly. Orientation preserves
unordered native comparison branches rather than copying the decompiler's
plain inequalities. `just test-native-dp-head-pose` verifies SHA-bound virtual
dispatch and threshold/angle constants, then runs fixtures for both head slots,
every count combination, attachment preservation, neighboring-state retention,
non-affine translation and unordered orientation. Mounted part tests also apply
the complete operation. These are static and Rust/offline checks, not observed
original-client execution, NaN-payload parity or cross-platform libm bit parity.

Bone transforms now implement the native in-place RotateX/RotateY/RotateZ and
Translate operations on `NativeDpBoneTransform`. These mix the existing columns
including their homogeneous fourth row, preserve unaffected columns, and clear
the native slot+0x100 byte. Zero-angle/zero-translation calls still execute their
arithmetic and invalidate the flag; the fresh affine atom-position helper was
not substituted. Multiplication/addition stages and the native SSE lane-pair
operand expressions are retained. Rust's platform sin/cos implementation is
used, without claiming native libm bit identity or NaN payload parity.

Complete static records are `docs/evidence/native-aumatrix-{rotate-x,rotate-y,
rotate-z,translate}.json`, corroborated by `native-bone-transforms-gnu.txt`.
Tests use dense nonaffine matrices, independent column-product expectations,
signed/zero angles and nonfinite zero-operation cases. This is a required matrix
dependency, not complete skeletal positioning. The still-live REA session
returned these queries in0.077–0.093s without another import.

The next retained dependencies include `dp-tail-{dead-down,dead-left,fold,bend}.json`
and `aumatrix-project.json` under `target/native-model-analysis-4`. Body wrapper
dispatch and the actual tail bend/Slither/Fold paths must still be composed;
retention is not implementation. The Project decompilation's hidden-return ABI
artifacts and unusual origin branch have now been resolved as described below.

`NativeDpBoneTransform::project` now implements the native sparse coordinate
paths and homogeneous normalization. Non-origin zero coordinates skip their
columns entirely (including unused nonfinite values). Origin copies translation
but computes W from all four homogeneous-row entries, not just matrix[15].
Division is skipped only for W=0 or1. The native SIMD groups each column sum
before adding translation; this differs from the decompiler's apparent
left-associated expression and is covered by a cancellation-sensitive fixture.
Projection is read-only and does not invalidate slot+0x100.

`NativeDpPartState::tail_current_connect_loc` composes the stored ConnectLoc
with actual owner slot17 (body offset0x11f8), including multi-bone tail DNA.
Other part kinds return Unsupported rather than borrowing this tail mapping.
This tail-specific entry point does not implement tail bending. Other concrete
part readers are described below. The caller must supply the actual owner
bones; production ownership/context assembly remains required.

The generic `NativeDpPartState::current_connect_loc` now extends that concrete
reader to all seven allocated part kinds. Torso projects origin through slot
`max(WaistVertex-1,0)` from its own DNA, deliberately ignoring stored ConnectLoc.
HeadAndNeck uses the native count/index selection in slots7..16. Limbs call the
actual `Bones(ref1,ref2,ref3)` mapping and project stored ConnectLoc through the
first out-reference, not an inferred upper-limb/root slot. Left/right pair0..2
triplets are explicit. Arm indices other than1/2 retain native pair0 fallback;
leg indices outside0..2 return Unsupported because native leaves the outputs
unwritten. The tail-specific convenience method still rejects other part kinds.

`just test-native-dp-connections` verifies all seven factory vptr+0x30 bindings,
four concrete vptr+0xa0 Bones bindings and first-out-reference selection against
the SHA-bound executable before running offline reader fixtures. Complete static
records are `docs/evidence/native-dp-{torso,head,arm,leg}-current-connect.json`
and `native-dp-{left,right}-{arm,leg}-bones.json`, with GNU corroboration in
`native-dp-connections-gnu.txt`. Reused-session queries took0.079–0.126s each.
Tests cover bilateral triplets, invalid indices, unique owner-slot translations,
all packed head/count combinations, torso waist-zero boundaries and ignored
connection data. These are body-local readers; full pose generation and world
placement/context composition remain required before production presentation.

Complete static records are `docs/evidence/native-aumatrix-project.json` and
`native-dp-tail-current-connect.json`; GNU corroboration is in
`native-bone-projection-gnu.txt`. The existing session was reused for the one
new tail-connection query (0.087s); Project's retained result was read without
decompiling it again. Offline fixtures cover sparse coordinate subsets,
origin/nonaffine W cases, read-only flags, skipped nonfinite columns, numerator
grouping and concrete tail slot selection. NaN payload identity and native
runtime/render parity remain unclaimed.

The shared native pose-rhythm dependency is now implemented in
`native_creature/pose_rhythm`: full-width pose classification, exact immediate
zero versus sine paths, signed reference milliseconds modulo60000 and the
observed ordered f32 stages. DhDNA's low-nibble rate adjustment starts its timer
before entering AuPose, even for an immediate-zero pose. A required timer
interface exposes start and msecsSinceReference separately; there is no guessed
elapsed-time or wall-clock fallback. The production time-domain provider remains
part of the production context work.

Complete static records are retained in `docs/evidence/native-pose-rhythm.json`
and `native-dhdna-rhythm.json`, with GNU return-register/call-site corroboration
in `native-pose-rhythm-gnu.txt`. The decompiler misidentified AuPose's return as
an integer and confused DhDNA's void timer-start return with a float argument.
The SSE instructions prove the rate/phase argument registers and float return;
those misleading expressions were not copied into Rust. `just
test-native-pose-rhythm` checks the SHA-bound native switch targets, mask and
scalar constants, timer calls and numeric widths, then runs offline fixtures.
Tests cover every byte pose, full-width defaults, signed reference extremes,
all256 DNA selector bytes, timer call order/errors and nonfinite inputs. Rust's
platform sine implementation is used; no cross-platform bit-identical native
libm or original-game runtime parity is claimed. Complete body animation still
requires composing this operation with the real pose, bone and resource state.

1. Implement the real model/part state and mounted-resource selection from the
   verified constructor/SetDNA paths. Unresolved generation must fail explicitly;
   an empty model or successful no-op is not a fallback implementation.
2. Connect actual part/chromatid volumes, wing capabilities and invalidation to
   the existing HP/growth/movement helpers; connect real animation to pose/head
   transitions and preserve timer/owner ordering.
3. Assemble a production Person context and session/dispatch owner, not a second
   parallel source decoder. Keep native/source boundaries explicit.
4. Exercise fresh and retained full Person packets carrying actual DNA, nested
   state and later fields, then publish actual native avatar body presentation.
   Diagnostic markers and fixture contexts do not prove this gate.
5. Verify malformed/unsupported cases, sparse repeated updates and mounted user
   assets offline; use the owned loopback source server only where its protocol
   overlaps. Record incompatibilities rather than claim native login parity.

This plan preserves the original end-to-end objective; model/body fidelity and
production context remain required work, not optional polish after the preview.

### Concrete tail bending

`NativeDpPartState::bend_tail` now mutates the actual tail bone range using
the owner body's length at40a8. The high two bits of copied DNA14 select the
last joint (17 for zero, otherwise16+2*selector). The initial local offset is
rounded in f32; joints then descend to17 with f64 local-Y additions. Each
projected joint pivot rotates its inclusive suffix, not a neck-style prefix.
Both pitch and yaw are widened before the native sin/cos coefficient stages;
homogeneous rows and the slot+100 invalidation byte are included. Zero angles
still perform the native writes.

Nonnegative clearance enables the native repeated ground scan over joints
last..18 and its fixed double-coefficient correction pass. Negative clearance
and NaN skip that pass. The scan preserves the origin homogeneous sum and
separate fifth-joint branch; it does not substitute an affine translation.
A Rust-only20,000-pass guard reports InvalidData rather than hanging on a
degenerate state; already-applied bone writes survive that error. Other part
kinds report Unsupported before writes. This guard is not a native limit.

The existing complete REA record is retained as
`docs/evidence/native-dp-tail-bend.json`, with GNU corroboration in
`native-dp-tail-bend-gnu.txt`; no additional import/query was performed.
`just test-native-dp-tail-bend` checks source digest, spacing/selection/loop
anchors and correction constants before Rust fixtures. Fixtures cover all
four ranges, suffix rotation accumulation, mixed-width pivot placement,
combined axes, dense non-affine matrices, homogeneous ground projection,
ground correction, negative/NaN clearance, zero angles and wrong-kind errors.
The mounted-part test also bends its initialized tail. Static checks and Rust
tests are not execution evidence from the original client, and no NaN-payload
or cross-platform libm bit parity is claimed. Body-level tail pose routines,
the remaining body animation, production Person ownership and actual avatar
render publication remain required work.

### Tail slither and death-tail stages

`slither_tail` now applies the concrete Slither(float,float) joint mutations.
It uses the same native segment selection/owner spacing as Bend, but reverses
both f32 angles at the middle joint and halves the current angles at root17,
where native flow bypasses the middle comparison. Its projected pivots,
expanded translation accumulation and per-column premultiplication are kept
separate where their operation ordering differs from Bend. Zero angles still
write the selected slots and clear their flags. Wrong part kinds fail before
mutation. `tail_can_bend_laterally` and `tail_can_bend_vertically` implement
the exact DNA14 shape/DNA16 form masks; fixtures cover all65,536 raw byte pairs.

The calculated-body `dead_tail_facing_down` and `dead_tail_facing_left` stages
select the first concrete tail, apply the native respective lateral/vertical
capability gate, and invoke Slither on the correct axis. No-tail or incapable
tail is an observed native no-op, not a missing-operation fallback. Pre-init
Rust calls report InvalidData. Mounted fixtures compare both an incapable
no-op and a capable complete bone mutation with direct Slither composition.

Complete REA records for both capabilities, angle Slither and both body death
stages are retained under `docs/evidence/native-dp-tail-*.json`; full GNU
corroboration is `native-dp-tail-slither-gnu.txt`. Five newly required related
queries reused the existing analyzed session, each completing in0.08–0.13s
without reimport. The additional pose-Slither and concrete Fold records are
retained in the working analysis directory for subsequent implementation;
neither operation is claimed implemented here. `just test-native-dp-tail-slither`
verifies seven IsTail slots, capability masks, loop/float-stage anchors and
both death-stage calls before Rust tests. Original-client runtime parity and
libm/NaN-payload bit parity remain unproven. Full tail pose dispatch, other body
animation, production Person state/context and actual rendered native avatar
presentation remain required parts of the original objective.
