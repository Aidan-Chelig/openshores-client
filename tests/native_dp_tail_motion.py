"""Static constants/stages for five actual body tail-motion routines."""
import re
import runpy
import struct
import subprocess

c = runpy.run_path("tests/native_dp_connections.py")
read = c["read"]
for address, bits in [(0x1422BFC94,0x3EB2B8C2),(0x1422BFCA0,0x3E567750),
                      (0x1422BFC78,0x3F060A92),(0x1422BFC7C,0x3E860A92),
                      (0x1422BFC80,0x3E32B8C2),(0x1422BFC98,0xBF800000),
                      (0x1422BFC9C,0xBE860A92)]:
    assert struct.unpack("<I",read(address,4))[0] == bits

def assembly(name):
    return subprocess.check_output(["objdump","-d","--no-show-raw-insn","--disassemble="+name,str(c["target"])],text=True)

for name, fixed in [("_ZN6DpBody8SwimTailEv",None),("_ZN6DpBody8WaveTailEff",None),
                    ("_ZN6DpBody8LashTailEff","1e"),("_ZN6DpBody7WagTailEf","2f")]:
    asm=assembly(name)
    for anchor in ["call   *0x50(%rax)","<_ZNK5DhDNA6RhythmERK6AuPoseff>",
                   "divss","mulss","ucomiss","<_ZN6DpTail4BendEfff>"]:
        assert anchor in asm,(name,anchor)
    if fixed:
        assert re.search(r"movl\s+\$0x"+fixed+r",",asm)
relax=assembly("_ZN6DpBody9RelaxTailEf")
assert "comiss" in relax and "ja" in relax
assert "<_ZN6DpTail4BendEfff>" in relax and "<_ZN6DpTail7SlitherEff>" in relax
print("Verified native tail motion constants, synthetic poses, mixed float stages, virtual tail lookup and concrete Bend/Slither calls. Static anchors, not runtime parity.")
