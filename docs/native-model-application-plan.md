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

`native_dna/volume.rs` now implements Head, Ear and Mammary from the independent
GNU disassembly retained in `docs/evidence/native-chromosome-volume-gnu.txt`.
Head reads stored shorts1e/20/22 with double scaling then float multiplication;
Ear uses short12 and preserves division after two float products. Mammary reads
short12, early-outs for zero or field0c==1, and otherwise uses ordered
double/float scaling and cubing. The twenty shared base methods return positive
zero. The other five overrides explicitly return Unsupported; chromatid summing
propagates that error, so this is not yet a general production volume provider.
Tests exercise wire-decoded records, unsigned-short extremes, exact float bits,
Mammary suppression, ordered sums and malformed storage. The static verifier
also pins the six consumed constant bit patterns to the original PE.
Preserve exact conversions and sum order, not simplified geometric formulas.

## Required implementation and verification

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
