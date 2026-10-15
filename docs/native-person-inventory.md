# Native Person inventory reception

Target: reference/openshores-full/OpenShores.exe, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
Static only: no supplied executable ran and no public server was contacted.

## Evidence and implemented order

Complete AuItemHash::ls assembly at1414f11d0 was already retained in
evidence/native-person-item-hash-gnu.txt and is reused here. Its independent
factory/filter/insertion dependencies are now retained in
evidence/native-inventory-dependencies-gnu.txt and the complete byte factory
in evidence/native-item-byte-factory-gnu.txt. These are GNU static observations,
not REA Evidence IDs, original source or runtime oracles. No Ghidra reimport
was needed for these dependencies.

NativeItemHash clears previous values **before** reading its signed32-overload
count. The native decrement-to-zero loop uses all32 raw bits. The Rust reader
preserves those bits and never allocates from the count alone. Each real item
body completes before the obsolete-commodity test; obsolete values are deleted,
not skipped by guessing their body length. Commodity2 then requires replacement
through the commodity-based factory (still Unsupported), before insertion.
Completed prior inserts remain on a later Rust error.

The factory's25 signed-relative entries at1422e1338 establish null types
0/2/3/11/15/21; bytes above24 also return null. Inventory continues after these
nulls, unlike the job list's null-factory stop. NativeItemHashReceive records
their wire indices/types. Every other factory is a real body: seven established
NativeItemValue types1/6/7/8/9/12/17 are composed directly. The remaining valid
types fail Unsupported, not guessed null or opaque skipping.

DbCommodity::IsObsolete14162fec0 returns true, for received unsigned16 bits,
exactly for36/46/47/67/ed/118/119/27b/27c/27d/27e (hex). The native bitset and
range branches are preserved in the evidence; all65536 input values are tested
against an independently expressed range/bitset fixture. Signed-short commodity
storage is zero-extended at this call site, so ffff is not treated as negative
or rejected using an invented catalog range.

The STL insertion at1414cd800 uses non-unique emplace. Its bounded matching-key
path compares hash/key, links a newly allocated node and increments count even
when the commodity already exists. All duplicate values must survive. Rust
stores a BTreeMap<u16, Vec<NativeItemValue>>, retaining wire order within groups;
it does **not** claim native bucket/iterator order, rehashing, node layout or
native iteration/presentation behavior. No unique-key replacement is used.

## Person composition and explicit boundaries

Person's C80 flag now invokes the real inventory reader after extended fields
and state1130 writes, before the trailing A01 identity byte and C40 nibble byte.
The nested receive report propagates through Person/body results as must-use.
Absent C80 retains prior inventory; present C80 replaces it, even with count0.
Tests actually compose the identity and inventory readers before both late tails.

Unimplemented valid item types and commodity2 replacement remain explicit
Unsupported errors. Supported nested item reads are atomic on malformed Rust
input; null type consumption, completed appends and successful earlier Person
mutations retain their documented order. This is not Qt failed-stream, object
constructor/catalog-default, allocation/destructor or locking parity. Existing
item records retain wire observations rather than invent unobserved defaults.

Remaining item bodies/fixup, Gear weight, concrete Creature/model runtime effects,
inherited native construction/dispatch and actual avatar presentation still
belong to the unchanged end-to-end goal. This is real inventory wire composition,
not proof of those unfinished gates or complete native Person packets.

## Verification

Fixtures cover all256 factory bytes, the65536-value obsolete predicate, all
seven supported item families, read-before-discard/truncation, duplicates and
raw high-bit commodity IDs, counts through65536, missing high-bit/maximal32-bit
counts without preallocation, clear-before-count, retained earlier values on
error, explicit fixup rejection, Person tail alignment in all precision profiles
and sparse retention. `just test-native-inventory` independently checks current
target bytes, all25 table entries/null cases, obsolete constants and the bounded
matching-key insertion path. It executes GNU tools only, never the target.

`just check` passes423 headless and444 client library tests,9 CLI tests per
profile,1 graphical-entry CLI test, formatting and strict client-feature
all-target Clippy. The static inventory verifier also passes.
