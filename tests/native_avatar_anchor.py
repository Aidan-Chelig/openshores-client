"""Focused SHA-bound Stand anchor checks; static evidence, no target execution."""
import runpy
import struct
import subprocess

context = runpy.run_path("tests/native_dp_connections.py")
read = context["read"]
target = context["target"]


def assembly(start, stop):
    return subprocess.check_output([
        "objdump", "-d", "-C", f"--start-address={start:#x}",
        f"--stop-address={stop:#x}", str(target)], text=True)


def dispatch(table, index):
    return table + struct.unpack("<i", read(table + index * 4, 4))[0]


# General Stand is fourth pose word at DpBody+0x48. Its CGOffset is zero.
assert dispatch(0x1421C5E34, 0x24 - 3) == 0x140978B10
offset = assembly(0x140978AC0, 0x140978CD0)
assert "mov    0x48(%rdx),%ecx" in offset
assert "sub    $0x3,%ecx" in offset
assert "140978b10:" in offset and "movq   $0x0,0x10(%rax)" in offset
assert "pxor   %xmm0,%xmm0" in offset and "movups %xmm0,(%rax)" in offset

# Stand does not add a separate pose-specific world rotation/translation.
assert dispatch(0x1421C664C, 0x24 - 4) == 0x1409808E7
adjust = assembly(0x140980830, 0x1409809E0)
assert "mov    0x48(%rax),%ebx" in adjust
assert "sub    $0x4,%ebx" in adjust
assert "1409808e7:" in adjust and "1409808f9:" in adjust
caller = assembly(0x1409A17BB, 0x1409A17CF)
assert "DaCreature::BodyPoseAdjustTransform(AuMatrix4x4&) const" in caller

# Normal Stand ground clearance is CGHeight(), whose active-body path requests
# CGHeight(true), then subtracts CGOffset.z. It is NOT the pose's false height.
ground = assembly(0x14097C670, 0x14097C710)
assert "cmp    $0x39,%ebx" in ground and "cmp    $0xf,%ebx" in ground
assert "jmp    14097c4d0 <DaCreature::CGHeight() const>" in ground
height = assembly(0x14097C568, 0x14097C5A8)
assert "mov    $0x1,%edx" in height
assert "DpBody::CGHeight(bool) const" in height
assert "DaCreature::CGOffset(DpBody const&, unsigned char, unsigned char, bool)" in height
assert "subsd  0x40(%rsp),%xmm6" in height
print("Verified Stand zero CGOffset, no BodyPoseAdjustTransform, and ground clearance via CGHeight(true). Static checks only; wire origin and terrain contact require separate evidence.")
