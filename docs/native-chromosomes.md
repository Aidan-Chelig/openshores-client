# Native chromosome receive ledger

Target and static-analysis profile are those in native-dna.md. Complete REA
records are `evidence/native-chromo-<16-digit COFF offset>.json`; their
`structuredContent.evidence_id` binds each observation to the target digest.
COFF section-1 offsets below add to `0x140001000` for the reader address.
No executable was run. These are receive layouts, not biological semantics.

## Dispatch binding

Factory evidence `ev_3e1690f65965b3d640869ce299508c3f780c8485ef4da8fed1e9420833087d6b`
establishes the type allocations. Independent GNU `.rdata` reads of each
factory vptr's `+0x20` slot corroborate the following reader addresses.
Skin/Fur constructor disassembly at `0x141217130`/`0x141217c00` establishes
their vptrs (`0x1423dab10`/`0x1423d0500`). This avoids relying only on
symbol aliases. The vptr column is the object vptr, not the preceding RTTI.

| Type | Class | Vptr | Reader COFF offset |
| --- | --- | --- | --- |
| 0 | Annelida | 14241a510 | 1204b20 |
| 1 | Arthropoda | 142435960 | 1204b20 |
| 2 | Chordata | 14241a660 | 1205f30 |
| 3 | Mollusca | 14241a900 | 1205f60 |
| 4 | Feathers | 14241a7b0 | 1205ab0 |
| 5 | Fur | 1423d0500 | 1205ab0 |
| 6 | Scales | 1423fc250 | 1205f90 |
| 7 | Skin | 1423dab10 | 1205ab0 |
| 8 | Head | 1423da720 | 121ff40 |
| 9 | Torso | 1423eaa40 | 121ffd0 |
| 10 | Arm | 1423cffc0 | 1220050 |
| 11 | Leg | 1423d0650 | 12200e0 |
| 12 | Fin | 1423d03b0 | 1220180 |
| 13 | Tentacle | 14241b6b0 | 12201e0 |
| 14 | Wing | 1423dac60 | 1220230 |
| 15 | Antenna | 14240a5a0 | 1206170 |
| 16 | Horn | 1423da870 | 12061c0 |
| 17 | Ear | 1423d0110 | 1220280 |
| 18 | Eye | 1423d0260 | 12202e0 |
| 19 | Nose | 1423da9c0 | 1220350 |
| 20 | Mouth | 1423ea8f0 | 12203a0 |
| 21 | Lungs | 1423ea7a0 | 1205fc0 |
| 22 | Gills | 1423ea650 | 1205fe0 |
| 23 | Blood | 1423ea500 | 1206030 |
| 24 | EatsAnimals | 142441330 | 1205fc0 |
| 25 | EatsPlants | 142435ab0 | 1205b20 |
| 26 | Reproduction | 14244b4f0 | 1206060 |
| 27 | Mammary | 14240a6f0 | 1206200 |

## Ordered fields

Offsets refer to native destination storage. `b` is one wire byte; `s` is a
Qt short (retain all 16 raw bits until signed semantics are established).
`b→u32`/`s→u32` indicate unsigned widening in the recovered reader.

- Annelida/Arthropoda: no body. Base chromosome reader `17714a0` is also a no-op.
- Chordata/Mollusca/Blood: b→u32 at 08.
- Skin/Fur/Feathers: QColor at 08, QColor at 18. Scales: QColor at 08, b at 18.
- Lungs/EatsAnimals: b at 08. EatsPlants: s→u32 at 08.
- Gills: b→u32 at 08, b→u32 at 0c.
- Reproduction: b→u32 at 08, b at 0c, packed b (high nibble to 0d,
  low nibble to 0e), b→u32 at 10.
- Antenna: s at 08, b→u32 at 0c, b at 10. Horn: b→u32 at 08, b at 0c.
- Mammary: packed b (bit 0 to 08, bit 1 to 10), s at 12, b→u32 at 0c, b at 14.

All following readers first call WithGenes (`121fe70`), forwarding the AuDNA
version byte unchanged, then receive:

- Head: b→u32 at 10, eight s at 14/16/18/1a/1c/1e/20/22.
- Torso: eight s at 10/12/14/16/18/1a/1c/1e.
- Arm: s10, s12, s14, b16, s18, s1a, b1c, s1e, b20, s22.
- Leg: nine s at 10..20, b22, s24, b26.
- Fin: s10, b→u32 at 14, b18, b19. Tentacle: b→u32 at 10, b14.
- Wing: s10, b→u32 at 14, b18.
- Ear: b→u32 at 14, b18, s10, s12 (wire order differs from field order).
- Eye: s→u32 at 10, s14, b→u32 at 18, s1c, b1e.
- Nose: s10, s12, s14, s16.
- Mouth: s10, s12, s14, s16, b→u32 at 18, s1c, s1e, s20.

WithGenes evidence
`ev_37579874db446d2f979a8dd1733e17070e7254e81f92c56ee4c326b643a63621`
reads one flag byte. Bit 0 present ensures an AuGeneChain at pointer 08 and
receives it with the version byte. Absent clears/deletes an existing chain
and sets the pointer null. Other flag bits do not receive additional fields.

## Outstanding dependencies and limits

Rust now implements the 13 scalar/no-body factory types: 0..3, 15, 16 and
21..27, retaining the type separately even where wire-body shapes are shared.
All other known types return Unsupported without consumption; null factory
types return None without consumption. Tests cover all 256 factory type
values, all byte values for widening/packed fields, short high bits, every
truncated scalar prefix and following-field alignment. This is a body reader,
not yet a chromatid container or complete AuDNA implementation.

Signed interpretation of stored short fields by later consumers still needs
evidence. Gene and chromatid-count dependencies are now resolved below.
Reader pseudocode includes imported tail-call warnings;
retain these in the complete records rather than treating the warnings as
actual switch statements. Some recovered signatures misidentify the `this`
register; field ordering here follows calls and destination offsets, not those
misrecovered parameter labels. Wire decoding does not establish model generation.

## QColor dependency resolved for scene version 7

Independent static GNU evidence is retained in
`evidence/native-qt-color-gnu.txt` for the supplied Qt5Gui.dll (SHA-256
`5d87baabd75693f81e43eb492e384611228073bedcda98a2b0438072180d9170`).
Export ordinal 8244/RVA 21be70 is `operator>>(QDataStream&, QColor&)`.
At `0x3356abe78` it tests stream version against 6. Version 7 takes the
branch at `0x3356abf20`: signed-char read, then five short reads via the
same imported overload, followed by sign extension of the spec byte to
32-bit storage and raw 16-bit component assignments. All specs consume the
same 11 bytes; no RGB-only shortcut, variable spec payload or float is read.
The <=6 branch differs and is not implemented by `NativeColorQt7`.

The scene's explicit version 7 was independently established in
native-entity-transforms.md. `NativeChromosome::read_qt7` now also handles
Feathers/Fur/Skin (two colors) and Scales (color then byte), bringing support
to 17 factory types. Tests cover all 256 spec bytes, high-bit components,
every truncated prefix and color-chromosome suffix alignment. Storage is raw:
this does not claim conversion to renderable RGBA or color-space validation.

## GeneChain and full wire composition

Independent GNU evidence `evidence/native-genes-gnu.txt` establishes:

- GeneChain at `0x141220c90`: Clear before signed-short count, decrement/test
  as a 16-bit word, then byte type, CreateGene, virtual receive `+0x18`, append.
  Null factory entries continue with no guessed body skip.
- Gene factory at `0x141210820`: the five jump-table destinations from
  `0x1422b37a0` bind types 0 Feathers, 1 Hair, 2 Scales, 3 Spines, 4 null;
  higher types also warn/return null. Slots `+0x18` at vptrs
  `1423fe710`, `1423c3530`, `1423dd080`, `1423dd110` respectively bind to
  `1412070d0`, `141206b50`, `141206af0`, `141206af0`.
- All concrete genes receive QColor08 then short18. Feathers additionally
  receives byte1a; Hair additionally receives qword20 via the signed 64-bit
  overload (raw bits retained). Scales/Spines have no further wire fields.
  None of these readers branches on the forwarded AuDNA version byte.

All 28 chromosome bodies now decode in Rust, with explicit native destination
offsets and raw byte/short/unsigned-widened values for gene-bearing fields.
The complete AuDNA reader composes both chromatids and gene chains; dynamic
DNA dispatch is also implemented. See native-dna.md for atomic error policy,
aggregate storage budget and offline coverage. The REA batch now independently
corroborates the GNU observations. Complete reviewed records in evidence/:

- native-genechain-reader.json: ev_4d7465f6977c9de8513bbb08072e6f39beb284b472cc2664adbcf36be0c9db63.
- native-gene-factory.json: ev_24b1900622e47dbdc9ed36565a93569e7e807b9c51450fc71739772023837601.
- native-gene-scales-spines.json: ev_1f24229b0301bc477aa6e8e56e188dd3fb01be2990aa9a82e858bae92fe1f75c.
- native-gene-hair.json: ev_eae4e0d046595edc688848e2a28c620d0899f888abf87ae8e4afc4c86fb02f15.
- native-gene-feathers.json: ev_e1c9ccc5c0ba60cb93fe93d25441d6c41a5024b4a5e0c8f8c634af0e81332ed1.

Imported tail-jump warnings and misrecovered `this` signatures remain in the
complete responses; independently checked GNU overloads resolve the wire
widths. Initial query took 640.443 seconds; subsequent queries in this batch
took 0.004–0.019 seconds, without another import.
