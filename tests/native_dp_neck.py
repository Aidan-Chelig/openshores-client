"""Static RotateNeck/LeanNeck comparison. No supplied executable is run."""
import hashlib
from pathlib import Path
import re
import subprocess

target = Path("reference/openshores-full/OpenShores.exe")
assert hashlib.sha256(target.read_bytes()).hexdigest() == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
rotate = "_ZN13DpHeadAndNeck10RotateNeckEf"
lean = "_ZN13DpHeadAndNeck8LeanNeckEf"


def assembly(name):
    return subprocess.check_output(["objdump","-d","--no-show-raw-insn","--disassemble="+name,str(target)],text=True)


def normalize(text,name):
    instructions = []
    for line in text.splitlines():
        match = re.match(r"\s*([0-9a-f]+):\s*(.*)",line)
        if not match:
            continue
        instruction = match[2].replace(name,"CURRENT")
        instruction = instruction.replace("_ZN11AuMatrix4x47RotateXEd","ROTATE_AXIS")
        instruction = instruction.replace("_ZN11AuMatrix4x47RotateYEd","ROTATE_AXIS")
        # Absolute branch/call addresses differ between the two functions, but
        # preserve symbol names and local offsets. RIP targets remain explicit
        # in objdump's comments even though displacement bytes differ.
        instruction = re.sub(r"\b[0-9a-f]+ (?=<)","TARGET ",instruction)
        instruction = re.sub(r"0x[0-9a-f]+\(%rip\)","RIP",instruction)
        instructions.append(instruction)
    return instructions


for suffix in ["", ".part.0"]:
    a = assembly(rotate+suffix)
    b = assembly(lean+suffix)
    left,right = normalize(a,rotate+suffix),normalize(b,lean+suffix)
    assert left == right, suffix
    if suffix:
        assert len(left) == 396
        assert a.count("<_ZN11AuMatrix4x47RotateXEd>") == 1
        assert b.count("<_ZN11AuMatrix4x47RotateYEd>") == 1
        assert "call   *0x30(%rax)" in a and "call   *0x30(%rax)" in b
    else:
        assert "ucomiss" in a and "jp" in a and "je" in a and "ret" in a
bend = "_ZN13DpHeadAndNeck8BendNeckEf"
assert normalize(assembly(bend),bend) == normalize(assembly(rotate),rotate)
bending = assembly(bend+".part.0")
for instruction in ["mulss  0x40a8(%rax),%xmm6", "cvtss2sd %xmm6,%xmm2",
                    "lea    -0x1(%rbx),%edx", "cmp    %ebx,%ecx", "jbe", "<sincos>"]:
    assert instruction in bending, instruction
print("Verified all three zero-angle wrappers,396 shared rotation/lean instructions, and bending's f32 spacing/descending-prefix loop anchors. Static structural checks, not runtime parity.")
