"""Interactive, static-only REA MCP session. One import, many function queries.

stdin: JSON lines {"procedure": "symbol-or-address", "output": "new-name.json"}.
Send {"close": true} or EOF to close gracefully. Never executes the target.
The output directory must be new; complete MCP responses are retained there.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import time


class Rpc:
    def __init__(self, process):
        self.process = process
        self.serial = 0

    def send(self, method, params, request_id=None):
        message = {"jsonrpc": "2.0", "method": method, "params": params}
        if request_id is not None:
            message["id"] = request_id
        self.process.stdin.write(json.dumps(message) + "\n")
        self.process.stdin.flush()

    def request(self, method, params):
        self.serial += 1
        request_id = self.serial
        self.send(method, params, request_id)
        for line in self.process.stdout:
            response = json.loads(line)
            if response.get("id") == request_id:
                if "error" in response:
                    raise RuntimeError(response["error"])
                return response["result"]
            # Server notifications are not a response or a terminal condition.
            if "method" in response and "id" in response:
                raise RuntimeError("unexpected server request: " + response["method"])
        raise RuntimeError("REA exited before returning a response")


def retain(path, value):
    with path.open("x") as output:
        json.dump(value, output, indent=2)
        output.write("\n")


def run(launcher, target, directory):
    directory.mkdir(parents=True, exist_ok=False)
    environment = os.environ.copy()
    environment["REA_GHIDRA_STARTUP_TIMEOUT_MS"] = "1800000"
    with (directory / "server.log").open("x") as log:
        process = subprocess.Popen(
            [str(launcher.resolve()), "--mcp"], stdin=subprocess.PIPE,
            stdout=subprocess.PIPE, stderr=log, text=True, bufsize=1,
            env=environment,
        )
        rpc = Rpc(process)
        opened = False
        try:
            retain(directory / "initialize.json", rpc.request("initialize", {
                "protocolVersion": "2025-03-26", "capabilities": {},
                "clientInfo": {"name": "openshores-static-analysis", "version": "1"},
            }))
            rpc.send("notifications/initialized", {})
            catalog = rpc.request("tools/list", {})
            retain(directory / "tools.json", catalog)
            names = {tool["name"] for tool in catalog["tools"]}
            required = {"open_binary", "close_binary", "procedure_pseudo_code"}
            if not required <= names:
                raise RuntimeError("REA lacks required static session tools")
            result = rpc.request("tools/call", {"name": "open_binary", "arguments": {
                "path": str(target.resolve()), "provider_id": "ghidra",
            }})
            retain(directory / "open.json", result)
            if result.get("isError"):
                raise RuntimeError("REA could not open target; see open.json")
            opened = True
            print("Session ready; enter procedure/output JSON lines; EOF closes.", flush=True)
            for line in sys.stdin:
                command = json.loads(line)
                if command.get("close"):
                    break
                filename = command["output"]
                if Path(filename).name != filename or not filename.endswith(".json"):
                    raise ValueError("output must be a plain .json filename")
                output = directory / filename
                if output.exists():
                    print(f"Skipped existing output {output}; no query sent for "
                          f"{command['procedure']}. Reuse retained evidence or choose "
                          "a new output for a different procedure.", flush=True)
                    continue
                started = time.monotonic()
                result = rpc.request("tools/call", {
                    "name": "procedure_pseudo_code",
                    "arguments": {"procedure": command["procedure"]},
                })
                retain(output, result)
                print(f"Retained {output}; {time.monotonic() - started:.3f}s; "
                      f"isError={result.get('isError', False)}", flush=True)
        finally:
            try:
                if opened and process.poll() is None:
                    retain(directory / "close.json", rpc.request("tools/call", {
                        "name": "close_binary", "arguments": {},
                    }))
            finally:
                process.stdin.close()
                process.wait()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("launcher", type=Path)
    parser.add_argument("target", type=Path)
    parser.add_argument("output_directory", type=Path)
    args = parser.parse_args()
    run(args.launcher, args.target, args.output_directory)


if __name__ == "__main__":
    main()
