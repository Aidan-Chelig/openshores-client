"""Source-bound static checks for PositionTorso's geometric prelude."""
import hashlib
import json
import struct
import subprocess
from pathlib import Path

target = Path("reference/openshores-full/OpenShores.exe")
assert hashlib.sha256(target.read_bytes()).hexdigest() == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"

def assembly(symbol):
    return subprocess.check_output(
        ["objdump", "-d", "--no-show-raw-insn", "--disassemble=" + symbol, str(target)],
        text=True,
    )

walk = assembly("_ZNK5DhDNA12WalksOnHandsEv")
for anchor in ["0x13(%rcx)", "0x9(%rcx)", "0xa(%rcx)", "$0xffffffe0", "$0x60", "$0xc0", "$0x6"]:
    assert anchor in walk, anchor
wrist = assembly("_ZNK5DhDNA12HasWristBendEv")
for anchor in ["0x13(%rcx)", "$0xffffffa0", "$0x80"]:
    assert anchor in wrist, anchor
torso = assembly("_ZN6DpBody13PositionTorsoEff")
for anchor in ["<_ZNK5DhDNA12WalksOnHandsEv>", "<_ZNK5DhDNA12HasWristBendEv>", "<_ZNK6DpBody10HandLengthEb>", "<asinf>", "0x40a4(%r12)", "0x40b4(%r12)", "0x4088(%r12)"]:
    assert anchor in torso, anchor
prelude = torso[torso.index("1412e37d4:"):torso.index("1412e3820:")]
for anchor in ["addss  %xmm11,%xmm0", "mulss", "comiss %xmm0,%xmm1", "ja     1412e33b7", "comiss %xmm10,%xmm1", "jae    1412e33b7", "divss", "<asinf>"]:
    assert anchor in prelude, anchor
hand = torso[torso.index("1412e387e:"):]
assert hand.index("addss  %xmm11,%xmm9") < hand.index("addss  %xmm9,%xmm0") < hand.index("mulss")

# Resolve the literal through the PE section table rather than assume raw offsets.
data = target.read_bytes()
pe = struct.unpack_from("<I", data, 0x3c)[0]
count = struct.unpack_from("<H", data, pe + 6)[0]
optional_size = struct.unpack_from("<H", data, pe + 20)[0]
image_base = struct.unpack_from("<Q", data, pe + 24 + 24)[0]
sections = pe + 24 + optional_size
for i in range(count):
    section = sections + i * 40
    rva, size, offset = struct.unpack_from("<III", data, section + 12)
    relative = 0x1422bfc08 - image_base - rva
    if 0 <= relative <= size - 4:
        assert struct.unpack_from("<I", data, offset + relative)[0] == 0x3f733333
        break
else:
    raise AssertionError("unmapped torso scale literal")
for name in ["native-dp-position-torso", "native-dhdna-walks-on-hands", "native-dhdna-wrist-bend"]:
    evidence = json.loads(Path("docs/evidence", name + ".json").read_text())
    assert evidence["subject"]["digest"]["sha256"] == hashlib.sha256(data).hexdigest()
print("Verified torso geometry selectors, ordered hand/limb additions, f32 scale and unordered-preserving comparison branches. Static evidence, not target execution.")
