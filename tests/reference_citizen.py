"""Offline player bootstrap profile parity; copied from source _pb2 layout."""
from pathlib import Path
import struct
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "reference/OpenShores-server/src"))
from openshores.protocol.stream import QDS
from openshores.protocol.atoms.gear import _pack_au_gear
from openshores.protocol.atoms.gear_variety import _build_variety_gear_entries
from openshores.protocol.framing import encode_size
from openshores.gameplay.story_targoss import build_job_blob


def main():
    name = QDS(); name.write_qstring("Explorer 🚀")
    home = QDS(); home.write_qstring("Home 🚀")
    creature = (bytes([1]) + name.getvalue() + struct.pack(">i", 55) + bytes([0, 0x0d, 0x24])
                + struct.pack(">I", 24) + bytes(range(24)) + struct.pack(">h", 46)
                + bytes([0x24]) * 10 + struct.pack(">I", 0) + bytes([90, 0, 0xed]))
    gear = _pack_au_gear(_build_variety_gear_entries())
    def body(flags):
        return (creature + bytes([flags]) + struct.pack(">I", 42) + home.getvalue()
                + struct.pack(">ffIBi", 1.25, -2.5, 55, 0, 0)
                + struct.pack(">H", 123) + bytes([5, 9, 2, 1]) + gear)
    stats = struct.pack(">iihddB", -3, 7, -2, 123.5, -4.25, 0)
    with tempfile.TemporaryDirectory(prefix="openshores-citizen-") as directory:
        path = Path(directory) / "bootstrap.frames"
        def inspect(payload):
            packet = bytes([0x12]) + struct.pack(">IqB", 42, 1000, 0) + payload
            path.write_bytes(encode_size(len(packet)) + packet)
            return subprocess.run([str(ROOT / "target/debug/openshores-client"), "inspect-scene", str(path)],
                                  capture_output=True, text=True, check=True, timeout=30).stdout
        output = inspect(body(0x68) + stats)
        for expected in ["home_system: Some(42)", 'home_name: Some(Some("Home 🚀"))',
                         "home_coordinates: Some([1.25, -2.5])", "hunger_raw: Some(123)",
                         "cursor_raw: Some([9, 2, 1])", "declared_slots: 7", "stop: None",
                         "xp: -3", "reputation: 7", "bank: 123.5", "second_balance_raw: -4.25"]:
            assert expected in output, (expected, output)
        # Jobs announce an unresolved variable block after gear: never read it as stats.
        output = inspect(body(0x6c) + struct.pack(">hB", 1, 0xFE) + b"opaque jobs" + stats)
        assert "declared_slots: 7" in output and "statistics: None" in output, output
        story = Path(directory) / "fixture.story"
        story.write_text(':\nstory.title("Fixture Story"); story.author("Fixture Author"); story.brief("Brief");')
        output = inspect(body(0x6c) + build_job_blob(path=str(story)) + stats)
        assert "declared_count: 1" in output and "jobs: Some(JobList" in output, output
        assert "statistics: Some(PlayerStatistics" in output and "bank: 123.5" in output, output
        assert 'author: Some("Fixture Author")' in output and 'title: Some("Fixture Story")' in output, output
    print("Player bootstrap: citizen/home, full variety gear, hunger/cursor, story jobs and statistics verified; unknown jobs remain opaque.")


if __name__ == "__main__":
    main()
