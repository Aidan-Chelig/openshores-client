"""Compare Rust commodity decoding with all supplied GD rows and source writer."""
import json
import re
from pathlib import Path
import subprocess
import sys
import tempfile
from types import SimpleNamespace

ROOT = Path(__file__).resolve().parents[1]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "reference/OpenShores-server/src"))
from openshores.gameplay.gd_tables import load_commodity_rows, _read_commodity_row
from openshores.protocol.db_static_tables import _build_dbcommodity_row, build_scene_db_commodity_0x2b
from openshores.protocol.framing import encode_size


def optional(value):
    return "None" if value is None else "Some(" + json.dumps(value, ensure_ascii=False) + ")"


def main():
    rows = [row for row in load_commodity_rows(ROOT / "reference/OpenShores-server/GD") if row.cid and not row.vacant]
    assert len(rows) == 615
    packets = [build_scene_db_commodity_0x2b(row) for row in rows]
    # Qt null differs from empty; the GD reader normalizes null to empty, so
    # separately check the source writer profile's actual nullable strings.
    fixture = _build_dbcommodity_row(30000, ccc_flags=0x102, name="Калашников 🌊", description=None, bitmap="", model=":data/model.3dsN")
    assert _read_commodity_row(fixture, 0)[3] == fixture
    packets.append(build_scene_db_commodity_0x2b(SimpleNamespace(raw=fixture)))
    with tempfile.TemporaryDirectory(prefix="openshores-commodities-", dir="/tmp") as temporary:
        capture = Path(temporary) / "commodities.frames"
        capture.write_bytes(b"".join(encode_size(len(packet)) + packet for packet in packets))
        result = subprocess.run([str(ROOT / "target/debug/openshores-client"), "inspect-commodities", str(capture)],
                                check=True, capture_output=True, text=True, timeout=30).stdout
        assert "Commodity catalog: 616 server-provided rows." in result
        # Rust Debug escapes combining marks as \u{...}; compare actual text,
        # not Python JSON's different spelling of the same Unicode value.
        lines = [re.sub(r"\\u\{([0-9a-f]+)\}", lambda match: chr(int(match[1], 16)), line)
                 for line in result.splitlines()]
        for row in rows:
            # Read raw Qt values, preserving null rather than the GD helper's
            # deliberate normalization to empty strings.
            from openshores.protocol.stream import QDS
            stream = QDS(row.raw)
            stream._take(59)
            description = stream.read_qstring()
            stream._take(2)
            bitmap, model = stream.read_qstring(), stream.read_qstring()
            stream._take(29)
            name = stream.read_qstring()
            expected = f"{row.cid}: name={optional(name)} description={optional(description)} bitmap={optional(bitmap)} model={optional(model)} flags={row.flags:#x} unparsed=0B"
            assert expected in lines, expected
        assert '30000: name=Some("Калашников 🌊") description=None bitmap=Some("") model=Some(":data/model.3dsN") flags=0x102 unparsed=0B' in lines
    print("All 615 supplied commodity labels/resources and nullable Unicode writer fixture match Rust.")


if __name__ == "__main__":
    main()
