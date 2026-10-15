# Native typed console station state

Target: reference/openshores-full/OpenShores.exe, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
The digest was rechecked against the current file. No supplied executable ran
and no public server was contacted.

`NativeConsoleState` now composes constructor-established receive subsets,
verified wire readers and the primary+e0/+f0 metadata getters. This is not yet
the enclosing Creature context, Qt/UI construction, console settings, sitting
lifecycle or sensor augmentation.

## Construction and wire bindings

All existing complete wrapper-constructor REA records were reviewed again:
ordinary consoles call their DpStation constructor then initialize a separate
DpConsole subobject; spacecraft wrappers initialize fields beyond the station
portion. They do not overwrite the receive-visible defaults used here.
Complete constructor records/IDs remain in native-console-constructors-index.json.

Independent `evidence/native-station-constructors-gnu.txt` retains the full
default DpStation constructor and eleven station-family constructors.
Referenced Nav/Power/Sensor constants and Empty+c0 are retained separately in
`native-station-constructor-constants-gnu.txt`. These are static GNU observations,
not REA Evidence IDs or runtime execution.

Explicit `from_native_constructor` methods are separate from existing Default
observation states. In particular, None in Power/Sensor station observations
must not become a guessed zero. Constructor receive-visible facts include:

| Family | Constructor receive subset |
| --- | --- |
| Engineer | zero receive fields; short2a from station argument |
| Helm | zero receive fields, including short26 |
| Game | raw byte14 from kind, short16 from station |
| Shield | six float fields and double30 zero |
| Nav | field14=0x01000000, field20=3, field60=6; other receive scalars/vectors/IDs zero; empty route |
| Surgery | short38=ffff, byte16=ff; other receive scalars zero; nullable string null; empty points/conditions; short28 from station |
| Power | two eight-entry value arrays at14/3c start17; other receive fields/groups zero |
| Sensor | bytes14..17=1,1,1,0; bytes18..19=1,1; byte1a=1; bytes40..43 all1; float38 bits492c0d47; other receive values zero/null/empty |
| Weapon | bytes14..17 all1, byte18=1, byte48=0; bytes3d..40=0,1,0,1; float38 bits492c0d47; other receive values zero/null/empty; shorta2 from station |
| Transporter | bytes3c..3f/40..41 all1, byte42=1; bytes48..4b=0,1,1,1; byte4c=1, byte4d=0; float38=1; vectors zero; other receive values zero/null/empty; shortae from station |

The typed factory covers ordinary1..5/7..10, spacecraft0b..10/12..15,
Game21..23 and Empty0 (hex type values). Unsupported class types return
Unsupported instead of an opaque skip. Empty0 is a real sitting-console class,
but **is not a case in Creature's separate temporary receive switch**. The
future context must preserve that enclosing distinction rather than call this
factory for every packet type indiscriminately.

`receive_wire_qt7` invokes the bound station reader. Power and the three sensor
families keep their ordered in-place/sparse behavior; other families retain
their documented fresh-value atomic error policy. Empty and ScCommand use the
base no-op reader. Empty's primary+c0 at142407d70 resolves to1410bdc60, independently
corroborating that it consumes no body, rather than guessing from its name.

The must-use wire result includes the Sensor/Weapon/Transporter stage result,
including null full factories and unknown minimal-list stops. That marker
explicitly still requires the local capsule/environment augmentation; this
wrapper neither performs nor silently claims those effects.

## Real metadata getters

`evidence/native-console-metadata-bindings.json` indexes all21 constructor-bound
primary vptrs and exact getter bodies through their first return; associated
bounded GNU windows are retained in native-console-metadata-gnu.txt. Bytes
after a getter's first return are unrelated padding/code, not part of its body.

Primary+e0 returns Engineer2a, Surgery28, Weapona2, Transporterae or Game16;
the remaining classes return literal0. In particular Helm26 is **not** its
station number. `station_number()` reflects this and explicitly rejects an
unobserved optional Weapon/Transporter station field rather than guessing0.

Primary+f0 returns class constants except Game and Empty. Game zero-extends its
**current** raw byte14, so it can differ from the constructor/factory type.
Empty tests its64-bit field20: zero yields0, nonzero yields3d. The receive
subset retains that predicate rather than inventing the unresolved field's
ownership/meaning. It startsfalse by constructor evidence and the no-op wire
reader leaves it unchanged. Future actual settings/UI mutations must maintain
this state; they are not implemented here. ScCommand overrides its Empty base
with the constant0b getter.

## Verification and next integration

Tests cover all256 factory bytes, all station shorts, every constructed family,
all profiles, following-field alignment, every truncated wire prefix, real
sparse Power updates retaining17, nonzero constructor flags, all Game type
bytes, Empty's predicate, native station getters and explicit unobserved errors.
`just check` passes375 headless and396 client library tests, 9 CLI tests per
profile, 1 graphical-entry CLI test, formatting and strict client-feature
all-target Clippy.

These concrete receive subsets and getters are ready for console matching and
sitting lifecycle integration. Actual lifecycle/settings/capsule effects,
Creature context, inherited Person construction/dispatch and avatar presentation
remain incomplete. Native-format component fixtures do not prove those gates.
