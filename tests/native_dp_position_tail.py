"""Verify complete native PositionTail dispatch fixture and special stages."""
from pathlib import Path
import runpy
import struct
import subprocess

c=runpy.run_path("tests/native_dp_connections.py")
read=c["read"]
targets={"relax_or_slither":0xC90,"receptive":0xD08,"wave_or_slither":0xCB0,
    "dead_down":0xCA0,"dead_left":0xCA8,"wag":0xD18,"wave_or_fixed_slither":0xCD0,
    "fold":0xCF0,"lash":0xD30,"noop":0xCF8,"lash_or_slither":0xCBB,
    "strike":0xD64,"swim":0xD00,"writhe_down":0xD55,"writhe_left":0xD5A,
    "writhe_right":0xD5F,"writhe_up":0xD10}
seen=set()
for row in Path("tests/fixtures/native_dp_tail_dispatch.tsv").read_text().splitlines():
    if row.startswith("#"): continue
    pose,action=row.split("\t");pose=int(pose);assert pose not in seen;seen.add(pose)
    expected=0x1412E0000+targets[action]
    actual=0x1422BE1F4+struct.unpack("<i",read(0x1422BE1F4+(pose-3)*4,4))[0] if 3<=pose<=159 else 0x1412E0CF8
    assert actual==expected,(pose,action)
assert set(range(160))<=seen and {160,255,65540,4294967295}<=seen
for address,value in [(0x1422BFC88,70.),(0x1422BFC8C,60.),(0x1422BF8F8,1.5),(0x1422BFCA4,15.)]:
    assert struct.unpack("<f",read(address,4))[0]==value
assert struct.unpack("<I",read(0x1422BFC90,4))[0]==0x3C8EFA35
methods=["_ZN6DpBody10StrikeTailEf","_ZN6DpBody20WritheTailFacingLeftEv",
    "_ZN6DpBody20WritheTailFacingDownEv","_ZN6DpBody21WritheTailFacingRightEv",
    "_ZN6DpBody18WritheTailFacingUpEv","_ZN6DpBody21ReceptiveTailFacingUpEv"]
for method in methods:
    asm=subprocess.check_output(["objdump","-d","--no-show-raw-insn","--disassemble="+method,str(c["target"])],text=True)
    for anchor in ["call   *0x50(%rax)","<_ZNK5DhDNA6RhythmERK6AuPoseff>","mulss","addss"]:
        assert anchor in asm,(method,anchor)
print("Verified all157 native PositionTail entries, unsigned defaults, special-motion constants and six real rhythm stages. Static evidence, not native execution.")
