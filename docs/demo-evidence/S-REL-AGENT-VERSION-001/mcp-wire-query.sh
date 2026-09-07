#!/usr/bin/env python3
# Helper: send MCP initialize to prism via stdin and print the JSON response.
# Called by AC-002 VHS tape. Usage: python3 mcp-wire-query.sh <binary> <config-dir>

import subprocess, json, threading, time, os, sys, signal

binary = sys.argv[1] if len(sys.argv) > 1 else "./target/debug/prism"
config_dir = sys.argv[2] if len(sys.argv) > 2 else "/tmp/prism-demo-config"

initialize_msg = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "demo-recorder", "version": "1.0.0"}
    }
}

request_line = json.dumps(initialize_msg) + "\n"

proc = subprocess.Popen(
    [binary, "start", "--config-dir", config_dir, "--log-format", "json"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    bufsize=0
)

mcp_ready = threading.Event()

def read_stderr():
    while True:
        line = proc.stderr.readline()
        if not line:
            break
        if b'mcp_server_started' in line:
            mcp_ready.set()

result = {}

def read_response():
    try:
        line = proc.stdout.readline()
        if line:
            result['response'] = json.loads(line.decode())
        else:
            result['error'] = "no response"
    except Exception as e:
        result['error'] = str(e)

stderr_thread = threading.Thread(target=read_stderr, daemon=True)
stderr_thread.start()

mcp_ready.wait(timeout=6)
time.sleep(0.2)

response_thread = threading.Thread(target=read_response, daemon=True)
response_thread.start()

try:
    proc.stdin.write(request_line.encode())
    proc.stdin.flush()
except Exception as e:
    result['write_error'] = str(e)

response_thread.join(timeout=5)

os.kill(proc.pid, signal.SIGTERM)
time.sleep(1)
try:
    os.kill(proc.pid, signal.SIGKILL)
except:
    pass

if 'response' in result:
    resp = result['response']
    # Pretty-print only the serverInfo portion for the demo
    si = resp.get('result', {}).get('serverInfo', {})
    print(json.dumps({"serverInfo": si}, indent=2))
else:
    print(f"ERROR: {result.get('error', result.get('write_error', 'unknown'))}", file=sys.stderr)
    sys.exit(1)
