# Native sitting pose calculations

Target: reference/openshores-full/OpenShores.exe, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
Static only: no supplied executable ran and no public server was contacted.

The former analyzed session is gone. These three unresolved functions are
small enough to reconstruct from independent GNU assembly without a new
Ghidra import. Complete exact-symbol output, raw jump tables and referenced
constants are retained in `evidence/native-pose-initialization-gnu.txt`.
This is independent static evidence, not a REA Evidence ID or runtime oracle.
The already retained SitAtConsole REA record establishes the enclosing calls;
its signatures/offsets remain static observations, not recovered source.

## Implemented helpers

`native_animalia_initialize_poses` implements AuAnimalia::InitPoses at1411e9f90.
For general words8b..9f it returns nine24 words followed by the input. Otherwise
it returns nine input words followed by8c. The sixteen SIMD constant bytes at
1422b2820 decode to four u32 values24. This fresh-value API models the separate
source/destination storage used by the observed sitting/SetDNA callers; it
does not claim to reproduce arbitrary overlapping native pointer writes.

`native_current_general_pose` implements CurrentGeneralPose at14097ed20:
use DpBody's word48 (array index3) when its pointer exists; otherwise Animalia's
word6c (index0). DpBody wins even if both are provided. Raw wide values remain
raw. Missing both returns a Rust InvalidData safety error instead of attempting
the native null dereference. No semantic fallback pose is invented.

`native_station_body_pose` prepares the observed SitAtConsole body argument:
DpBody repeats the computed general word ten times, whereas Animalia uses the
real InitPoses helper above. This is data preparation, not completed animation.

## Complete OpPose calculation

`native_station_operator_pose` implements AuStation::OpPose at1414bde20.
The 60 signed-relative main entries at1422ddf34..1422de024 choose constant,
occupant-table, signed-index or shared-random branches. Twelve six-entry tables
at1422de024..1422de144 resolve occupant cases. The independently decoded map is
retained in `evidence/native-station-operator-pose-map.csv` and used by fixtures.
Decoding stops at each block's unconditional jump/return; adjacent block
immediates are not attributed to an earlier constant case.

Constant branches ignore occupant and index. Occupant branches accept0..5 and
use the retained exact table; larger raw values fall back to0x24. Types0x39..0x3b
select0x86 for occupants0,1,2,5 and0x87 for3,4, otherwise0x24. Types0x21/0x22 select
0x42..0x45 for signed index0..3; type0x23 selects0x46..0x49. Other indices and unknown
station types use24, not a guessed console or opaque packet skip.

Types0x24..0x38 mutate shared AuGlobal u32 atf4:

`next = old * 0x41c64e6d + 0x3039` modulo2^32.

The assembly zero-extends that next u32 to64 bits, converts to f64, multiplies
by4.0 and2^-32, then truncates. Both constants are exact powers of two, so the
choice index is exactly `next >> 30` for every u32 state. Index argument0
chooses from4c/4d/4e/4f; **any** nonzero signed index chooses50/51/52/53.
Occupant position does not affect this branch. Both choice arrays and scaling
constants at1422de1e8..1422de220 are retained.

The caller must supply the actual shared mutable random state. None produces
Unsupported only on those random branches; deterministic paths do not require
or mutate a seed. There is no guessed initial seed, private per-creature RNG,
extra random advance or unverified default. Shared-state initialization and
runtime ownership still belong to the concrete client context.

## Sitting markers and remaining application

`NativeCreaturePoseMarkers::after_console_pose` performs the sitting variant:
conditionally assign5c0 and restart its monotonic timer when body change or a
different station requires it; then **always** assign trailing492. It differs
from fallback body reception, where492 is assigned only on restart. Both real
Instant and injected-clock paths preserve that distinction.

The complete SitAtConsole operation still needs actual console creation/reuse/
destruction, parent-capsule notification and rendering/animation effects.
These helpers supply its real pose values and marker rules; they do not mark
it or the required Creature context complete. Person dispatch/presentation
also remains outstanding.

## Verification

Fixtures cover all byte station/occupant inputs, wide bounds, signed indices,
the independent 60-case table, ignored inputs and unchanged random state on
deterministic branches. Fixed LCG vectors cover overflow and all four choice
quadrants; repeated calls share the updated seed and missing state explicitly
fails. InitPoses tests cover every u16 word plus wide bounds, body selection
priority and all station shorts for console-marker timing. Existing pose and
wire-layer tests remain in the full check suite.
`just check` passes 368 headless and 389 client library tests, CLI tests,
formatting and strict client-feature all-target Clippy.
