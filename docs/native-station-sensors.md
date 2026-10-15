# Sensor, Weapon and Transporter station wire stages

Static target: reference/openshores-full/OpenShores.exe, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
No native executable was run and no public server was contacted.

Complete retained REA records in evidence/:

- native-console-reader-141108bc0.json (Sensor):
  `ev_675d5abc4d042083cc599aa13c619c38f33708b7b65f87bbb3d31b94c3d3b432`.
- native-console-reader-1411412e0.json (Weapon):
  `ev_cdf09507aad41b85cf7eadbf9dec91dc69b1830dd9665918412ca6a4eef2a8fa`.
- native-console-reader-141133c70.json (Transporter):
  `ev_446a47064f505456f9df66b7946c668524b7b5cc11a1fdf1e8700b9d8c6a5de2`.

Independent `evidence/native-station-sensors-gnu.txt` retains complete exact-symbol
GNU disassembly of all three functions, including branches outside the linear
wire prefix. This corroborates scalar imports, packed byte offsets, conditional
vectors, full virtual +158, and counted minimal RxMin calls. It is not a REA
Evidence ID or runtime observation. In particular, the decompiled base Refresh
alias is the no-op DpStation::Rx at1410bdc60, not a stream-consuming refresh.

## Implementation boundary

`world::native_creature` exports `NativeStationSensor`, `NativeStationWeapon`
and `NativeStationTransporter`. Their `receive_wire` methods perform ordered
state updates and compose the real full and minimal sensor-blip readers. They
do **not** claim to implement complete DpStation::Rx. The must-use result
reports null full factories, unknown minimal-list stops and consumed bytes.
The caller still owes the local capsule/environment augmentation described
below; a concrete dispatcher must not replace an unsupported context with a
silent no-op. No complete Creature dispatcher calls these stages yet.

Option scalar/group fields mean unobserved receive state, not zero-valued
native construction. Packed arrays list bytes in increasing native memory
offset order; names do not infer gameplay meaning. Fresh full blip values have
their own verified factory semantics (native-sensor-blips.md).

Each selected full blip is destroyed/replaced, not patched in place. Sensor
and Weapon read the type byte before destroying the old value. Transporter
destroys it after its first two bytes, before its conditional type read. A null
factory consumes just that type byte and the station proceeds to the next
scalar; this is different from unknown **minimal-list** types, which stop the
list and leave the remaining entries unread. Both are reported explicitly.

Earlier station field writes survive later errors. Full blip bodies and vector
assignments complete before committing their values; on an interrupted full
body the old selection is gone and kind remains recorded without a value.
These are Rust error policies, not Qt failed-stream mutation/destructor parity.
Nested minimal lists clear before count and retain completed appends on a later
error. A reported unknown list type is not permission to skip its opaque body.

## Ordered wire layouts

F means QDataStream float overload, V means three F values widened to f64,
ID is the real AuId/u32 reader, byte is raw eight bits and short preserves all
16 bits. DoublePrecision wire F values narrow through f32; TypeWidths/Single
consume f32. Raw NaN/infinity/signed zero survive wire state. Rendering safety
is not performed here.

Sensor at141108bc0:

1. Short flags S. Bytes14..17 get bits0,1,2,3; bytes18..19 bits4,5; byte1a
   bit6; bytes40..43 bits12,15,13,14.
2. Byte type, destroy old full50, factory and +158 if nonnull.
3. V20, F38, F3c, ID1c, minimal list58.

Weapon at1411412e0:

1. Short S. Bytes14..17 bits0,1,2,3; byte18 bit4; byte48 gets1 when bit6,
   otherwise ff when bit7, otherwise0. GNU1411418e0..8e8 confirms signed
   byte shift, not boolean normalization. Bytes3d..40 bits12,15,13,14.
2. Raw byte41, byte type, destroy old full70, factory and +158 if nonnull.
3. V20, F38, raw byte3c, F4c, short50, shorta2, i32 at44, ID1c, minimal list78.

Transporter at141133c70:

1. Byte A: u32 at18=A&3. Byte B widened unsigned to u32 at1c.
2. Destroy old full70. If A&10 read type/factory/+158, otherwise clear it.
3. If A&20 receive V50, otherwise retain V50. Then byte4d=A bit5.
   Presence is assigned after the optional vector, before mandatory V20.
4. V20, F38, short S. Bytes40..41 bits2,3; bytes3c..3f bits4,5,0,1;
   byte42 bit6; byte4c bit14; bytes48..4b bits11,12,15,13.
5. F44, shortae, ID14, minimal list78.

## Local augmentation still required

After the final RxMin all three inspect capsule SensorsWorking (+d40), find
a parent rendering environment (+7b0), construct a temporary sensor list, and
query environment +858 using capsule transform/identity/system context. Sensor
passes mask0x01, Weapon mask0x02, Transporter mask0x04 or0x44 based on byte48. Additional
blips are cloned/merged through virtual predicates. None of this consumes more
station packet fields. Weapon also has a specialized DaVessel power/capacitor
branch, plus a generic virtual SensorsWorking path. The complete retained
records preserve both; these effects still need a real native world context.

This milestone does not fake those effects or equate a successful wire stage
with complete console application, Creature reception or avatar presentation.

## Verification

Independent Rust fixtures embed actual Animal, Bd, shielded spacecraft and
Anomaly full bodies, followed by all four distinct minimal layouts. They test:

- Every profile, exact suffix consumption, high-bit raw values and f32/f64
  point distinctions inside the shielded spacecraft.
- All 65536 packed short values for each station, all256 Transporter presence
  bytes, all raw station bytes, and every null factory discriminator.
- Repeated Transporter updates retaining the omitted vector while clearing the
  omitted full selection and resetting its presence flag.
- Every truncated wire prefix across representative full bodies and all
  Transporter optional layouts; ordered Sensor/Weapon field retention and
  Transporter destruction/presence timing on partial updates.
- Unknown minimal-list stop propagation from each station, destructive list
  replacement, partial retention, non-finite wire values and signed zero.

These are offline native-layout fixtures. The owned Python reference server's
source packet profile is not evidence for this patched native console format.
