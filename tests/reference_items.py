"""Offline DaItem parity with supplied server writers; no sockets/database."""
from pathlib import Path
import struct
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "reference/OpenShores-server/src"))
from openshores.protocol.atoms.daitem import pack_floor_item
from openshores.protocol.atoms.daitem_drop import (
    _build_daitem_drop_packet, _build_daitem_keepalive_packet,
)
from openshores.protocol.atoms.item_seed import _pack_auitem_seed_body
from openshores.protocol.framing import encode_size


def main():
    binary = ROOT / "target/debug/openshores-client"
    with tempfile.TemporaryDirectory(prefix="openshores-items-") as path:
        path = Path(path) / "items.frames"
        def inspect(packets):
            path.write_bytes(b"".join(encode_size(len(p)) + p for p in packets))
            return subprocess.run([str(binary), "inspect-scene", str(path)], check=True,
                                  capture_output=True, text=True, timeout=30).stdout
        floor = pack_floor_item(42, 7, xyz=(1.25, -2.5, 3), timestamp_ms=1000,
                                timeCreate_ms=900, timeModified_ms=950)
        text = inspect([floor])
        for expected in ["Item: 1", "parent=Some(7)", "commodity_id: 1", "byte14: 1", "quality: None",
                         "position: [1.25, -2.5, 3.0]"]:
            assert expected in text, (expected, text)
        body = _pack_auitem_seed_body(typeId=1, cid=130, byte14=5, quality=61, name="Specimen 🚀")
        drop = _build_daitem_drop_packet(42, struct.pack(">I", 8), (4, 5, 6), 1, body,
                                        time_created_ms=2000)
        keepalive = _build_daitem_keepalive_packet(42, 1, bytes([0]) + struct.pack(">hB", 130, 7))
        text = inspect([floor, drop, keepalive])
        for expected in ["parent=Some(8)", "position: [4.0, 5.0, 6.0]", "commodity_id: 130",
                         "byte14: 7", 'name: Some(Some("Specimen 🚀"))', "quality: Some(61)"]:
            assert expected in text, (expected, text)
        weapon = _pack_auitem_seed_body(typeId=8, cid=31, byte14=5, quality=61, name="Pistol")
        text = inspect([_build_daitem_keepalive_packet(43, 8, weapon)])
        assert "type_id: 8" in text and "commodity_id: 31" in text and "opaque_tail: [" in text
        text = inspect([_build_daitem_keepalive_packet(44, 0xFE, bytes([9, 8, 7]))])
        assert "type_id: 254, common: None" in text
        assert "raw_payload: [9, 8, 7], opaque_tail: [9, 8, 7]" in text
    print("DaItem: floor/drop/keepalive, sparse name/quality, weapon prefix and unknown type verified offline.")


if __name__ == "__main__":
    main()
