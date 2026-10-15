# Native body HP and growth

The `native_creature::health` implementation uses the complete retained GNU
records in [native-movement-dependencies-gnu.txt](evidence/native-movement-dependencies-gnu.txt),
not source-protocol health fields or inferred bounding-box volumes.
`just test-native-health` checks actual instruction anchors, selected target
digest and the original float scale words. These are static observations;
neither the original executable nor its model code is executed.

DpBody::MaxHitPoints uses cached float68. Exactly zero (either sign) causes an
ordered f32 sum of actual part virtual98 Volume values. Nonzero/NaN caches are
retained. The constructor-established cache is zero; actual model mutation must
perform its separately verified invalidation, not an invented reset in this
helper. Completed sums remain after later Rust provider failures. A stable
model-part collection is required during this exclusive Rust operation; native
concurrent/list-mutating callbacks are not emulated. The 20000-part bound is a
Rust safety policy, not an inferred native maximum.

HP uses f32 sqrt(volume) multiplied by exact PE float41ec86eb for Dp or421cf478
for Animalia, then x86 CVTTSS2SI. Rust's saturating cast is not equivalent for
NaN/overflow: conversion deliberately preserves x86 integer-indefinite behavior.
Only a negative LOW signed short triggers replacement by32767. Diagnostics are
returned as a must-use flag, not emitted in native Qt logging order. Native
errno/FP exceptions and logging parity are not claimed.

Dp byte1d bit2 enables a reduction: absent byte19 mask3 or byte1a mask6 uses
floor(2*HP/3); otherwise logical short shift-right halves HP. Animalia instead
uses actual selected-chromatid Volume, scans actual parts for fly-capable wings
and halves HP once at the first match. Null/non-wing parts are not guessed wings;
unknown identities/capabilities must fail Unsupported in the required provider.
After those modifiers, a zero short becomes1. No catalog/model volume is invented.

Dp GrowthTime is byte25 OR7. Animalia GrowthTime takes its actual unscaled model
MaxHP and returns the low byte of signed-min(ceil(f32(sqrt(HP)+sqrt(HP))),255).
Creature's separate growth adjustment uses signed32 division, short truncation
and zero-to-one replacement only when growth494 is nonzero. Its zero-time divide
trap becomes a Rust InvalidData error. MovementSpeedMod now composes the same
verified arithmetic while preserving its own nonpositive-HP early return.

Fixtures verify cache retention/recomputation, partial sums, raw byte masks,
wing scanning, every raw signed16 Animalia growth input, growth bytes, conversion
limits and short diagnostics. On x86-64 a host-SSE intrinsic oracle independently
checks conversion/sqrt/multiply results; this is execution of our test arithmetic,
not execution of the supplied game. Actual morphology/chromatid volumes, wing
generation, model invalidation, weights/animation and the production native
client context remain unfinished. These helpers do not complete the avatar goal.
