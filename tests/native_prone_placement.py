"""SHA-bound adult nonvehicle prone draw translation; no target execution."""
import struct
import subprocess
from native_dp_connections import read, target

def assembly(start, stop):
    return subprocess.check_output(["objdump", "-d", "-C", f"--start-address={start:#x}", f"--stop-address={stop:#x}", str(target)], text=True)

for vtable in [0x1423CA560, 0x14248C5A0]:
    assert struct.unpack("<Q", read(vtable + 16 + 0xF8, 8))[0] == 0x140978ED0
wrapper = assembly(0x140978F20, 0x140978F8B)
assert "typeinfo for DaVehicle" in wrapper and "setne  %r14b" in wrapper
assert "mov    %r14d,0x20(%rsp)" in wrapper and "DaCreature::CGOffset(DpBody const&" in wrapper
for pose in [0x20, 0x21]:
    table = 0x1421C5E34
    assert table + struct.unpack("<i", read(table + (pose - 3) * 4, 4))[0] == 0x140978BE7
offset = assembly(0x140978BE7, 0x140978C2E)
assert "test   %r8b,%r8b" in offset and "test   %r9b,%r9b" in offset
assert "DpBody::HeadLocation() const" in offset and "xorpd" in offset
head = assembly(0x1412D0630, 0x1412D078B)
for token in ["cmpl   $0x37,0x48(%rdx)", "testb  $0x2,0x1d(%rsi)",
              "cmpb   $0x0,0x3a(%rsi)", "call   *0x40(%rax)",
              "movupd 0x50(%rax),%xmm2", "$0x7a8,%edx", "$0xcd0,%ecx",
              "AuMatrix4x4::Project", "DpHeadAndNeck::CurrentHeadLoc"]:
    assert token in head
render = assembly(0x1409A1A0A, 0x1409A1A55)
assert "lea    0x140(%rsp),%rcx" in render and "call   *0xf8(%rax)" in render
assert "je     1409a2738" in render
ordinary = assembly(0x1409A2738, 0x1409A2815)
for token in ["0x140(%rsp),%xmm1", "0x148(%rsp),%xmm2", "0x150(%rsp),%xmm3", "jmp    1409a1c0b"]:
    assert token in ordinary
translation = assembly(0x1409A1C0B, 0x1409A1C58)
assert "AuMatrix4x4::Translate(double, double, double)" in translation
assert "DaCreature::BodyPoseAdjustTransform" in assembly(0x1409A17BB, 0x1409A17CF)
print("Verified adult prone HeadLocation offset, typed CGOffset virtual binding, vehicle suppression and rotation-before-local-translation render path. Static evidence only.")
