"""Source-generated server chat envelopes, decoded offline by Rust."""
from pathlib import Path
import asyncio
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "reference/OpenShores-server/src"))
from openshores.protocol.atoms.aucomm import build_chat_aucomm_v4
from openshores.gameplay.natives.conversation import build_choice, build_chat, build_hail_reply
from openshores.gameplay.story_targoss import _build_narrate, _build_line
from openshores.gameplay import story_targoss


def main():
    async def controls():
        waiting = asyncio.get_running_loop().create_future()
        story_targoss._PENDING_CHOICE[512] = waiting
        try:
            assert await story_targoss.on_story_op({}, 512, bytes([0x7e]))
            assert not waiting.done()
            assert await story_targoss.on_story_op({}, 512, bytes([0x7f]))
            assert not waiting.done()
            assert await story_targoss.on_story_op({}, 512, bytes([0x7d]))
            assert waiting.cancelled() and 512 in story_targoss._DROPPED
        finally:
            story_targoss._PENDING_CHOICE.pop(512, None)
            story_targoss._DROPPED.discard(512)
    asyncio.run(controls())
    with tempfile.TemporaryDirectory(prefix="openshores-chat-") as directory:
        path = Path(directory) / "chat.bin"
        text = "Hello 🌊".encode("utf-16-be")
        tail = len(text).to_bytes(4, "big") + text
        for flags in (0, 0x40, 0x80, 0xe5):
            path.write_bytes(build_chat_aucomm_v4(0x29, tail, 512, "Rust Explorer", 0,
                             channel_index=2, flags_byte=flags, range_byte=0xa3, aux_i32=-7))
            output = subprocess.check_output([str(ROOT / "target/debug/openshores-client"),
                                              "inspect-chat", str(path)], text=True)
            assert "Hello 🌊" in output and "sender: 512" in output
            assert "channel: Indexed(2)" in output
            assert ("range: Some((3, 10))" in output) == bool(flags & 0x40)
            assert ("auxiliary: Some(-7)" in output) == bool(flags & 0x80)
        path.write_bytes(build_choice("Go on 🌊", 700, "Native", 512, choice_index=3))
        output = subprocess.check_output([str(ROOT / "target/debug/openshores-client"), "inspect-chat", str(path)], text=True)
        assert "Choice(" in output and "index: 3" in output and "instance: 1" in output
        assert 'prompt: Some("Go on 🌊")' in output and "offered_to: 512" in output
        path.write_bytes(build_chat("x" * 300, 700, "Native", 512))
        output = subprocess.check_output([str(ROOT / "target/debug/openshores-client"), "inspect-chat", str(path)], text=True)
        assert "x" * 300 in output and "Message(" in output
        path.write_bytes(build_hail_reply("Greetings", 123, 700, "Native", 512))
        output = subprocess.check_output([str(ROOT / "target/debug/openshores-client"), "inspect-chat", str(path)], text=True)
        assert "Hail(" in output and "environment: 123" in output and "Greetings" in output
        for kind in (0x4a, 0x4c, 0x4e):
            path.write_bytes(_build_narrate("Continue the story 🌊", "Getting Started", 512,
                                           type_byte=kind, choice_index=2))
            output = subprocess.check_output([str(ROOT / "target/debug/openshores-client"), "inspect-chat", str(path)], text=True)
            assert "Continue the story 🌊" in output and "instance: 4097" in output
            assert 'Named(Some("Getting Started"))' in output
            if kind == 0x4c:
                assert "index: 2" in output and "offered_to: 512" in output and "Choice(" in output
        path.write_bytes(_build_line("Story speech", "Targoss", 512))
        output = subprocess.check_output([str(ROOT / "target/debug/openshores-client"), "inspect-chat", str(path)], text=True)
        assert "Story speech" in output and "kind: 45" in output
    print("Chat/story parity: chat, native choice, continued text, hail, narration/title/choice and story speech decoded from actual source builders")


if __name__ == "__main__":
    main()
