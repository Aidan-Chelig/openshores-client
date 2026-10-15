# Person receive-visible construction

`NativePersonSuffixState::from_default_constructor_subset()` seeds proven
constructor writes, unlike observation-only `Default`. It is not full Person
allocation: inherited Unit/Creature/Sentient, skeletal/model state, QObject
signals and owner lifecycle must still be constructed and integrated.

The retained Person constructor record is
[native-person-default-constructor.json](evidence/native-person-default-constructor.json),
REA Evidence `ev_372afccd2f5bf68461eece6a46942f116f301dfb259ce0e34ef654f65a79f3b1`.
The inherited Creature record is
[native-creature-default-constructor.json](evidence/native-creature-default-constructor.json),
Evidence `ev_73addb5598f07be19980d8559b023c97261fcb582e138e205c608122f5719ae3`.
Both are static decompiler evidence, not original source or runtime observations.
`just test-native-person-construction` independently checks GNU instructions
against the selected PE's SHA-256 and constructor call chain. C1 and C2 share
addresses; GNU's selected C1 labels are Person 0x140a3f930 and Creature
0x140973ab0. No new Ghidra import is needed to inspect the retained records.

Established suffix defaults:

- Creature writes short660=0; Person writes state1130=0. Elapsed timer668 starts
  invalid, represented by no Instant, not a fabricated construction epoch.
- Identity IDs10ec/10f8/1110 are zero; QStrings10f0/1100/1118 are known null;
  the eight-byte zero at1108 covers both stored GlobeLLF floats.
- Scalars838/839/f80/f84/f88/fa8/fa9/fb0/10d0/f58/1408/f60/f68/10e8 and
  weight13a8 are zero. The qword10cc zero covers int10d0; the vector storef60
  covers both doubles. Constructor initialization is byte-width-sensitive.
- Byte83b=15 and short83c=0x0f0f establish byte83d=15, not zero.
- Contentfa0/jobs13b8 start as Qt shared-null empty lists; Domain1018 is an
  empty QHash; Gear1390's actual constructor stores the shared-null list;
  inventory1458 has an empty hashtable (zero element/node counts).

Identity1114 has no established constructor write in the retained body, so it
remains unknown. HitNotify's delegated constructor is not established by this
subset; all its observations stay unknown. Do not fill those fields with zeros
merely because adjacent IDs or containers are initialized.

Tests exercise first masked updates without caller-injected priors, omitted
fields, known-null versus unknown, nonzero nibble defaults, sparse repeated
updates, sticky state bits and timer-change behavior. These prove the suffix
subset only; native packet-to-avatar integration remains unfinished.
