"""Verify Stand identity and native composition anchors without target execution."""
import json
import runpy
import struct
import subprocess

context = runpy.run_path("tests/native_dp_connections.py")
read = context["read"]
path = context["Path"]

def assembly(symbol):
    return subprocess.check_output(["objdump", "-d", "--disassemble=" + symbol, str(context["target"])], text=True)

name = assembly("_ZNK6AuPose4NameEv")
for anchor in ["mov    %rdx,%rbp", "0x160(%rsp),%rcx", "142304911", "movslq 0x0(%rbp),%rax", "0x40(%rsp,%rax,8),%rax"]:
    assert anchor in name, anchor
assert 0x40 + 0x24 * 8 == 0x160
assert read(0x142304911, 6) == b"Stand\0"
for line in path("docs/evidence/native-neutral-pose-name-gnu.txt").read_text().splitlines():
    if line.strip().startswith("1415"):
        assert line in name, line

# Both leg and both arm dispatch tables select the verified standing branches.
# Second leg table is indexed after subtracting4; arm tables use raw indices.
for table, index, target in [(0x1422BF404, 0x24, 0x1412F1723),
                              (0x1422BF534, 0x24 - 4, 0x1412F1780),
                              (0x1422BF080, 0x24, 0x1412EE2C8),
                              (0x1422BF1B0, 0x24, 0x1412EE478)]:
    assert table + struct.unpack("<i", read(table + index * 4, 4))[0] == target
assert read(0x1412F1723, 3) == bytes.fromhex("4531c0")  # later pairs selector0
assert read(0x1412F1780, 6) == bytes.fromhex("41b801000000")  # first pair1
legs = assembly("_ZN6DpBody12PositionLegsEff")
arms = assembly("_ZN6DpBody12PositionArmsEff")
for anchor in ["0x4c(%rcx)", "0x3c(%r12)", "<_ZN6DpBody9StandLegsEfi>"]:
    assert anchor in legs
for anchor in ["0x44(%rcx)", "0x50(%r12)", "<_ZN6DpBody9StandArmsEfib>"]:
    assert anchor in arms

animate = assembly("_ZN6DpBody11AnimatePoseEbf.part.0")
assert animate.strip() == path("docs/evidence/native-dp-animate-pose-gnu.txt").read_text().strip()
for anchor in ["<_ZNK6DpBody8CGHeightEb>", "<_ZN6DpBody16InitBoneMatricesEv>",
               "<_ZN6DpBody12PositionLegsEff>", "<_ZN6DpBody12PositionTailEff>",
               "<_ZN6DpBody13PositionTorsoEff>", "<_ZN6DpBody13PositionHeadsEv>",
               "<_ZN6DpBody12PositionArmsEff>", "<_ZN6DpBody13PositionWingsEf>"]:
    assert anchor in animate
for stem in ["dp-animate-pose", "dp-position-legs", "dp-position-arms"]:
    record = json.loads(path(f"docs/evidence/native-{stem}.json").read_text())
    assert record["subject"]["digest"]["sha256"] == context["hashlib"].sha256(context["data"]).hexdigest()
    assert record["normalized_result"]
print("Verified pose0x24=Stand, both leg/arm dispatch tables and retained AnimatePose stages. Static evidence, not runtime/visual pose parity.")
