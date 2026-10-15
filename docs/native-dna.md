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
  and `0x10`, forwarding that byte to both. The chromatid format is pending.
- `native-species-reader.json`: `ev_105347d63c396b94e47fb5ca7b1e4942f369b402d7bf2cb370fe2fda26a5a10a`.
  AuSpecies receives QByteArray at object `+0x08`, not QString or DhDNA.
  The decompiler reports an imported tail-jump warning; semantic species
  interpretation and later consumers are not established by this reader.

Next reconstruct chromatids, then compose both DNA variants into Pregnant,
ImplantDNA and Creature reception. Wire storage is not body-model generation.

Rust `world::native_dna` now reads normalized DhDNA bytes and raw AuSpecies
byte arrays, preserving null versus empty species storage. DhDNA rejects null
and lengths other than 24 atomically; species storage has a Rust-only 1 MiB
budget. These safety policies do not claim native malformed-input parity.
ImplantDNA and ImplantSpecies are connected to the real condition decoder,
without inserting an extra dynamic-DNA discriminator. Tests cover all 256
DhDNA byte values, every truncated DhDNA/implant prefix, wrong lengths, null,
raw non-text species bytes, storage bounds and following-field alignment.
Dynamic AuDNA/chromatid and Pregnant decoding remain outstanding.
