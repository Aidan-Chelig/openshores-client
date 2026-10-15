# Native pose application data and markers

Static target: reference/openshores-full/OpenShores.exe, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
No supplied executable was run or public server contacted.

Reviewed complete REA records and IDs remain in native-creature-application-index.json:

- native-dpbody-pose.json:
  `ev_a31c55746df3797fefe4f6753da895adf85015b17b90849b520502c7b819ff8f`.
- native-animalia-pose.json:
  `ev_497ad7a3b037c5352765552b3ebc149a4725bd40fd266e20263ce89f84a4e353`.
- native-creature-station-pose.json:
  `ev_02c19d3a07f8095325f69d923d606908e413b5cb60bab62ee9c88c1391a4c23b`.
- native-station-occupant-position.json:
  `ev_6f7574a64e5244d38066fa603de2adc93b5af1f71f910845d2e06840c3bcb59f`.

Independent `evidence/native-body-pose-gnu.txt` retains complete exact-symbol
disassembly of all four, plus the raw occupant lookup table. It is static GNU
corroboration, not REA Evidence or execution. Existing constructor records
establish the zero words/booleans/float and invalid elapsed timer used below.

## Body pose state

`NativeDpBodyPoseState::apply` compares and replaces ten words at3c..60 and
byte6c. Any changed word **or boolean** sets the returned changed flag. It calls
the required animation callback only when the caller establishes nonempty
model data, including on unchanged input. Speed is forwarded raw, does not
affect changed, and is not independently stored by this wrapper.

`NativeAnimaliaPoseState::apply` compares and replaces ten words at6c..90, then
assigns bool40 and float94. Only differences in the incoming **words** set
changed; boolean/speed changes do not. Final word outside8b..9f is diagnosed
and replaced with8c **after** change detection. The result reports that raw
invalid value. Animation always runs with the normalized state, including on
unchanged input. Thus a first all-zero pose returns unchanged but normalizes
its final word; a repeated invalid zero pose then returns changed against8c.

Both APIs require animation callbacks rather than offering a success-returning
default. The callback must implement the actual model/identity/timer effect or
return Unsupported. The state mutations preceding a callback error survive;
that is a Rust error policy, not emulation of native faults. These wrappers do
not reconstruct bone matrices or claim complete AnimatePose implementation.
Raw NaN, infinities and signed-zero speed values are preserved.

## Body-fallback markers

After the fallback body Pose returns successfully, `NativeCreaturePoseMarkers`
tests **changed OR station != field5c0**. When true: assign5c0, restart the
monotonic timer, assign492. Otherwise retain all three, even if492 differs due
to console state. It never samples the clock on an unchanged path. A real
Instant-based method and an injected-clock test path are provided. This is not
the separate wall-clock body timestamp and must not replace SitAtConsole's
different marker ordering.

## Station-pose request

`NativeStationPoseRequest::from_received_pose` uses word3 for DpBody but word0
for Animalia. It passes the received short into both station arguments. The
request preserves the observed station type, occupant byte and variant/index.
Type0 is a valid **present** request for pose41, not an absent request.
Unsupported pose words return None, meaning normal body fallback, not a
successful station application.

The133 occupant bytes at1422de160..1422de1e5 are retained in
`evidence/native-station-occupant-table.json`; poses3..135 index that table,
others return3. This is separate from accepted SetStationPose cases: a word
can have an occupant byte without being a station pose.

The133 signed little-endian relative entries at1421c6438..1421c664c are
independently retained in native-station-pose-jump-table-gnu.txt. Resolving each
branch against the disassembly's r9d type and stack28 variant immediates yields
`native-station-pose-map.csv`, used as an independent fixture for every case.
The type0 branch uses xor r9d,r9d; extraction stops at its tail jump, not the
following block's nonzero immediate. All133 mappings now agree with Rust.

A must-use request still requires actual SitAtConsole. Complete reviewed
`evidence/native-creature-sit-at-console.json`,
`ev_d098c3b5ad82cce591ef46706abdcda430843fb740188b354e8924908cf0673e`,
shows that it computes OpPose, reuses/replaces consoles by CurrentGeneralPose,
can notify a parent capsule, applies body poses, conditionally restarts5c0's
timer and **unconditionally** assigns trailing492 afterward. OpPose,
Animalia InitPoses and CurrentGeneralPose are now implemented from complete
independent assembly; see native-sitting-pose-data.md. Console lifecycle,
capsule notification and actual animation still require concrete application.
It is not safe to equate the request with those completed effects.

## Verification and remaining boundary

Offline tests cover every word/byte position, all wire final-pose bytes plus
wide invalid values, pre-normalization comparisons, boolean/speed differences,
animation call conditions/failure ordering, every station short, native jump
table mappings and every marker/station combination for four high-bit priors.
`just check` passes363 headless and384 client library tests, CLI tests,
formatting and strict client-feature all-target Clippy.

These components are ready for the required Creature application context; that
concrete context, actual animation/sitting effects, Person dispatch and avatar
presentation remain incomplete. No mock callback success is counted as final
runtime integration.
