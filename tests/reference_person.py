"""Decode supplied server player builders offline; no sockets or database."""
from pathlib import Path
import struct
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "reference/OpenShores-server/src"))
from openshores.protocol.atoms.person import (
    _build_daperson_xform_update, _build_daperson_parent_update,
    _build_augear_only_daperson_update,
)
from openshores.protocol.atoms.effect import _build_player_effect_atom_pkt
from openshores.protocol.framing import encode_size
from openshores.protocol.stream import QDS


def inspect(packets, directory):
    path = directory / "person.bin"
    path.write_bytes(b"".join(encode_size(len(p)) + p for p in packets))
    return subprocess.run([str(ROOT / "target/debug/openshores-client"), "inspect-scene", str(path)],
                          capture_output=True, text=True, check=True, timeout=30).stdout


def main():
    name = QDS()
    name.write_qstring("Explorer 🚀")
    dna = bytes(range(24))
    # Same prefix construction as network/scene/connection.py bootstrap _pb2.
    prefix = (bytes([1]) + name.getvalue() + struct.pack(">i", -2)
              + bytes([0, 0x0D, 0x24]) + struct.pack(">I", 24) + dna
              + struct.pack(">h", 46) + bytes([0x24]) * 10 + struct.pack(">I", 0)
              + bytes([99, 0, 0xED]) + b"opaque citizen/gear/job payload")
    spawn = bytes([0x12]) + struct.pack(">IqB", 42, 1000, 0) + prefix
    movement = _build_daperson_xform_update(player_auid=42, parent_auid=7,
        x=1.25, y=-2.5, z=3, timestamp_ms=2000, agent_bits=33, _stamina_byte=lambda _: 92)
    parent = _build_daperson_parent_update(42, 8, timestamp_ms=3000,
        _stamina_byte=lambda _: 91, agent_bits_for=lambda _: 32)
    # network/bio_ticker.py live HP/hunger/stamina update layout.
    bio = (bytes([0x12]) + struct.pack(">IqB", 42, 4000, 0)
           + bytes([0, 4]) + struct.pack(">h", -30) + bytes([80, 0, 12])
           + struct.pack(">H", 0x8007) + bytes([5]))
    gear = _build_augear_only_daperson_update(struct.pack(">I", 42), b"opaque gear",
        _stamina_byte=lambda _: 81, agent_bits_for=lambda _: 5)
    effect = _build_player_effect_atom_pkt(42, next_effect_time_ms=lambda: 123,
        _stamina_byte=lambda _: 10, agent_bits_for=lambda _: 0)
    with tempfile.TemporaryDirectory(prefix="openshores-person-") as path:
        directory = Path(path)
        initial = inspect([spawn], directory)
        for expected in ['name=Some("Explorer 🚀")', "empire: Some(-2)", "hp: Some(46)",
                         "dna_tag: Some(36)", "stamina: Some(99)", "person_flags: Some(237)"]:
            assert expected in initial, (expected, initial)
        moved = inspect([spawn, movement], directory)
        assert "position: [1.25, -2.5, 3.0]" in moved
        assert "parent=Some(7)" in moved
        assert "agent_bits: Some(33)" in moved
        final = inspect([spawn, movement, parent, bio, gear, effect], directory)
        for expected in ["parent=Some(8)", "hp: Some(-30)", "hunger_raw: Some(32775)",
                         "stamina: Some(81)", "agent_bits: Some(5)", "dna: Some(Some([0, 1, 2",
                         "poses: Some([36, 36, 36", "empire: Some(-2)"]:
            assert expected in final, (expected, final)
    print("Person state: bootstrap, movement, parent, bio, gear-prefix and opaque effect replay verified offline.")


if __name__ == "__main__":
    main()
