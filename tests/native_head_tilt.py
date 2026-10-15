"""Static Person scalar binding and head-angle operations; no target execution."""
import hashlib
from pathlib import Path
import struct
import subprocess

target = Path("reference/openshores-full/OpenShores.exe")
data = target.read_bytes()
assert hashlib.sha256(data).hexdigest() == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
pe = struct.unpack_from("<I", data, 60)[0]
optional = pe + 24
base = struct.unpack_from("<Q", data, optional + 24)[0]
headers = optional + struct.unpack_from("<H", data, pe + 20)[0]


def read(address, length):
    for index in range(struct.unpack_from("<H", data, pe + 6)[0]):
        rva, size, offset = struct.unpack_from("<III", data, headers + 40 * index + 12)
        if base + rva <= address and address + length <= base + rva + size:
            start = offset + address - base - rva
            return data[start:start + length]
    raise AssertionError(f"unbacked head-tilt address {address:#x}")


def assembly(symbol):
    result = subprocess.check_output(["objdump", "-d", f"--disassemble={symbol}", str(target)], text=True)
    assert f"<{symbol}>:" in result
    return result


assert "14248c5b0 <_ZTV8DaPerson+0x10>" in assembly("_ZN8DaPersonC1Ev")
assert struct.unpack("<Q", read(0x14248C5B0 + 0x1E0, 8))[0] == 0x140988990
outer = assembly("_ZN10DaCreature10TiltHeadToEf")
for anchor in ("$0x39,0x30(%rsp)", "$0xf,0x30(%rsp)", "ucomiss %xmm7,%xmm6",
               "140988ad0:", "_ZN6DpBody11TiltHeadsToEff", "140988ad8:",
               "0x5e8(%r12)", "_ZNK6AuPose8IsLockedEv", "140988ba7:",
               "_ZN6DpBody11AnimatePoseEbf", "140988bb4:", "_ZN6DaAtom11SetInMotionEv",
               "$0x17,0x40(%rdx)", "$0x17,0x54(%rdx)", "$0x17,0x5c(%rdx)",
               "$0x17,0x78(%rdx)", "_ZTI10DaCreature", "_ZTI9DaVehicle",
               "140988d1b:", "0x68(%rax)", "0x64(%rax)", "140988e5e:", "140988e6f:"):
    assert anchor in outer, anchor
dp = assembly("_ZN6DpBody11TiltHeadsToEff")
assert "maxss  %xmm1,%xmm0" in dp and "movlps %xmm1,0x30(%rcx)" in dp
for address, bits in ((0x1421CA02C, 0x3FC90FDB), (0x1421CA030, 0xBFC90FDB),
                      (0x1422BFA48, 0x3FC90FDB), (0x1422BFA50, 0xBFC90FDB)):
    assert read(address, 4) == struct.pack("<I", bits)
locked = assembly("_ZNK6AuPose8IsLockedEv")
for anchor in ("$0x49,%eax", "$0x38,%eax", "$0x3,%eax", "$0x5,%eax", "$0xa,%eax", "$0x4c,%eax", "$0x3b,%eax"):
    assert anchor in locked, anchor
print("Native Person+1e0 -> TiltHeadTo binding, float limits, pose gates and ordered operation anchors verified statically.")
