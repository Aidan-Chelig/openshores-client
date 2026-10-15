# Native DNA receive evidence

Target: reference/openshores-full/OpenShores.exe, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
All observations below are static Ghidra 12.1.2, not target execution.
New retained records use the complete MCP envelope; evidence is in
`structuredContent`, with its duplicate text content also preserved.

- `native-dhdna-fields.json`: `ev_6cdd61fe18befa543a015f9d0c9a5cd510bf19b4342f1327c637213d9dda22b4`.
  DhDNA_ receives a QByteArray, warns if its length is not 24, then copies
  24 bytes, clears bit 7 of byte 5 and writes byte 23 zero. It does not decode
  24 individual Qt scalar fields or reverse the payload's byte order.
  Independent GNU assembly at `0x1412abe30..0x1412abe68` confirms the imported
  QByteArray reader/length check; `0x1412abf98..0x1412abfb2` confirms raw copy
  and both normalization writes. Native wrong-length behavior is not a safe
  bounded read; Rust should reject null/wrong-length payloads explicitly.
  Constructor vptr `0x14247c3f0` slot `+0x20` at `0x14247c410` contains
  `0x1412ac0d0` (DhDNA wrapper), independently checked in `.rdata`.
- `native-dna-dynamic.json`: `ev_2d95c5beb70a6a21d608044dfeb3e4ceceba4a147a6ea4afb7983fd41de886c9`.
  Read one type byte: 3 constructs DhDNA and calls virtual receive `+0x20`;
  4 constructs AuDNA and calls its reader. Other types return null after
  consuming only that byte. Do not confuse unknown types with known but
  unimplemented type 4 or silently skip a guessed body.
- `native-audna-reader.json`: `ev_169a199e8dbf16da02475e7ec9d1a8c73da03e4868e6d82743f0218e87a9cfab`.
  Read one byte, then call AuChromatid::ReadStream twice for fields `0x08`
  and `0x10`, forwarding that byte to both. The reconstructed chromatid format
  is documented in native-chromosomes.md.
- `native-species-reader.json`: `ev_105347d63c396b94e47fb5ca7b1e4942f369b402d7bf2cb370fe2fda26a5a10a`.
  AuSpecies receives QByteArray at object `+0x08`, not QString or DhDNA.
  The decompiler reports an imported tail-jump warning; semantic species
  interpretation and later consumers are not established by this reader.

Both DNA variants now have wire readers and Pregnant composition; next compose
them into Creature reception. Wire storage is not body-model generation.

Rust `world::native_dna` now reads normalized DhDNA bytes and raw AuSpecies
byte arrays, preserving null versus empty species storage. DhDNA rejects null
and lengths other than 24 atomically; species storage has a Rust-only 1 MiB
budget. These safety policies do not claim native malformed-input parity.
ImplantDNA and ImplantSpecies are connected to the real condition decoder,
without inserting an extra dynamic-DNA discriminator. Tests cover all 256
DhDNA byte values, every truncated DhDNA/implant prefix, wrong lengths, null,
raw non-text species bytes, storage bounds and following-field alignment.
Dynamic AuDNA/chromatid readers and Pregnant composition are now implemented.

## Chromatid receive boundary

Reviewed `native-chromatid-reader.json`,
`ev_9de4b6a262f0a898ad3a9cc533ecbdb79d2c2b0667e9eb0ba489219fb4f34a93`:
clear existing chromatid, read short count, then per entry read type byte,
CreateChromosome, receive through virtual `+0x20` forwarding AuDNA's leading
byte, and append. Null factory results warn and continue without guessed
body skipping. GNU count corroboration is now retained in
evidence/native-chromatid-count-gnu.txt: signed-short overload, zero-extended
word load, low-word decrement/store, zero test. All 16-bit patterns therefore
represent that many iterations, subject to the explicit Rust storage budget.

Reviewed factory `native-chromosome-factory.json`,
`ev_3e1690f65965b3d640869ce299508c3f780c8485ef4da8fed1e9420833087d6b`:
types 0..27 map in order to Annelida, Arthropoda, Chordata, Mollusca,
Feathers, Fur, Scales, Skin, Head, Torso, Arm, Leg, Fin, Tentacle, Wing,
Antenna, Horn, Ear, Eye, Nose, Mouth, Lungs, Gills, Blood, EatsAnimals,
EatsPlants, Reproduction, Mammary. Type 28 explicitly returns null; other
values warn and return null. Factory initializers are retained in full evidence,
and reader layouts/vtable bindings are now reconstructed in native-chromosomes.md. In
particular, a raw QByteArray is not the complete AuDNA representation.

The 26 distinct reader records are now retained and reviewed; see
native-chromosomes.md for ordered fields and independently checked virtual
dispatch bindings. GeneChain and QColor have since been resolved below.
A rejected guessed factory symbol
is retained only under target/avatar-analysis-session/chromosome-factory.json;
the successful factory evidence uses the exact COFF CTYPE symbol.

## Current Rust DNA composition

`NativeAuDna::read_qt7` reads the leading version and both actual counted
chromatids, including all 28 chromosome types and their optional counted gene
chains. `NativeDna::read_qt7` adds the dynamic discriminator (3 DhDNA, 4 AuDNA);
unknown types consume only one byte and return None, matching the null factory
boundary. No opaque body skipping or byte-array substitute is used for AuDNA.
All currently known chromosome/gene readers ignore the forwarded AuDNA version
byte; it is retained rather than rejected or rewritten.

Fresh-value decoding is atomic on Rust errors, unlike native in-place clearing
and partial appends. This is an explicit malformed-input safety policy, not
native stream-failure parity. One Rust-only 20,000-entry budget spans both
chromatids and every gene chain, including null-factory entries; counts never
directly preallocate storage. This bounds nested allocation amplification.
Tests exercise all chromosome types together, both chains, every version byte,
null factories, nested concrete genes, every truncated full-DNA prefix,
dynamic discriminators, high-bit values, field order and budget failure.
Complete Person construction/reception and presentation are not yet integrated.
