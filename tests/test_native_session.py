"""Transport tests without REA, Ghidra, network or target execution."""
import io
import json
from types import SimpleNamespace
import unittest
from unittest.mock import patch
from pathlib import Path
from tempfile import TemporaryDirectory
from contextlib import redirect_stdout
from native_session import Rpc, run


class SessionTransportTests(unittest.TestCase):
    def test_duplicate_output_does_not_query_overwrite_or_close_live_session(self):
        class Stdin(io.StringIO):
            def close(self):
                pass  # retain sent messages for assertions after run's cleanup
        responses = [{}, {"tools": [{"name":name} for name in ["open_binary", "close_binary", "procedure_pseudo_code"]]},
                     {"isError":False}, {"evidence":"first"}, {"evidence":"second"}, {"closed":True}]
        process = SimpleNamespace(stdin=Stdin(), stdout=io.StringIO("".join(
            json.dumps({"jsonrpc":"2.0", "id":index, "result":result})+"\n"
            for index,result in enumerate(responses,1))), poll=lambda:None, wait=lambda:0)
        commands = [{"procedure":"first", "output":"first.json"},
                    {"procedure":"duplicate", "output":"first.json"},
                    {"procedure":"second", "output":"second.json"}, {"close":True}]
        with TemporaryDirectory(prefix="native-session-transport-") as temporary:
            output = Path(temporary)/"new-session"
            stdout = io.StringIO()
            with patch("native_session.subprocess.Popen", return_value=process), patch("native_session.sys.stdin", io.StringIO("".join(json.dumps(c)+"\n" for c in commands))), redirect_stdout(stdout):
                run(Path("unused-launcher"),Path("unused-target"),output)
            self.assertEqual(json.loads((output/"first.json").read_text()), {"evidence":"first"})
            self.assertEqual(json.loads((output/"second.json").read_text()), {"evidence":"second"})
            self.assertIn("no query sent for duplicate", stdout.getvalue())
        messages = [json.loads(line) for line in process.stdin.getvalue().splitlines()]
        calls = [m["params"] for m in messages if m["method"] == "tools/call"]
        self.assertEqual([c["name"] for c in calls], ["open_binary", "procedure_pseudo_code", "procedure_pseudo_code", "close_binary"])
        self.assertEqual([c["arguments"]["procedure"] for c in calls if c["name"] == "procedure_pseudo_code"], ["first", "second"])

    def test_notifications_do_not_end_or_restart_request(self):
        process = SimpleNamespace(stdin=io.StringIO(), stdout=io.StringIO(
            '{"jsonrpc":"2.0","method":"notifications/progress","params":{}}\n'
            '{"jsonrpc":"2.0","id":1,"result":{"complete":true}}\n'
            '{"jsonrpc":"2.0","id":2,"result":{"next":true}}\n'))
        rpc = Rpc(process)
        self.assertEqual(rpc.request("first", {}), {"complete": True})
        self.assertEqual(rpc.request("second", {}), {"next": True})
        sent = [json.loads(line) for line in process.stdin.getvalue().splitlines()]
        self.assertEqual([message["id"] for message in sent], [1, 2])

    def test_terminal_eof_is_an_error_not_implicit_relaunch(self):
        process = SimpleNamespace(stdin=io.StringIO(), stdout=io.StringIO())
        with self.assertRaisesRegex(RuntimeError, "exited"):
            Rpc(process).request("query", {})

    def test_protocol_errors_are_not_successful_evidence(self):
        process = SimpleNamespace(stdin=io.StringIO(), stdout=io.StringIO(
            '{"jsonrpc":"2.0","id":1,"error":{"code":-1,"message":"bad"}}\n'))
        with self.assertRaises(RuntimeError):
            Rpc(process).request("query", {})


if __name__ == "__main__":
    unittest.main()
