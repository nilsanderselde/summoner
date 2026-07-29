#!/usr/bin/env python3
"""
Summoner DAW — Smart Vibe Runner (Real-Time JSON Streamed)
Runs agy in autonomous loop, parsing stream-json events to render live tool calls,
thinking progress, code edits, and streaming agent text.
"""

import sys
import subprocess
import json
import time
from datetime import datetime

PROMPT = (
    "Read local/ROADMAP_20260729.md and previous analysis. "
    "Proceed with implementing the next incomplete logical step on the roadmap, "
    "edit the code, run tests to verify, and commit all changes with a detailed commit message "
    "before finishing or beginning the next step."
)

AGY_FLAGS = [
    "agy", "-p", PROMPT,
    "--output-format", "stream-json",
    "--dangerously-skip-permissions"
]

def log(msg, color="\033[36m"):
    timestamp = datetime.now().strftime("%H:%M:%S")
    reset = "\033[0m"
    print(f"{color}[{timestamp}] {msg}{reset}", flush=True)

def run_vibe_turn(step_num):
    log(f"🤖 Starting Vibe Turn #{step_num}...", "\033[1;36m")
    
    proc = subprocess.Popen(
        AGY_FLAGS,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1
    )

    full_output = []

    for line in proc.stdout:
        line_str = line.strip()
        full_output.append(line_str)
        
        if not line_str.startswith("{"):
            if line_str:
                print(f"  {line_str}", flush=True)
            continue
            
        try:
            data = json.loads(line_str)
            event = data.get("event")
            
            if event == "step_update":
                update = data.get("step_update", {})
                tool_calls = update.get("tool_calls", [])
                for call in tool_calls:
                    fn = call.get("function", {})
                    name = fn.get("name", "unknown_tool")
                    raw_args = fn.get("arguments", "{}")
                    try:
                        args = json.loads(raw_args)
                        summary = args.get("toolSummary") or args.get("toolAction") or args.get("CommandLine") or args.get("TargetFile") or ""
                    except Exception:
                        summary = ""
                    log(f"  🛠️  Tool: {name} {f'— {summary}' if summary else ''}", "\033[33m")
                
                text_delta = update.get("text_delta")
                if text_delta:
                    sys.stdout.write(text_delta)
                    sys.stdout.flush()

            elif event == "result":
                res = data.get("result", {})
                status = res.get("status")
                if status == "SUCCESS":
                    log(f"  ✅ Turn #{step_num} finished successfully!", "\033[32m")
                else:
                    log(f"  ⚠️ Turn finished with status: {status}", "\033[31m")

        except json.JSONDecodeError:
            print(f"  {line_str}", flush=True)

    proc.wait()
    return proc.returncode, "\n".join(full_output)

def main():
    step = 1
    log("🚀 Summoner Autonomous Vibe Runner Started", "\033[1;35m")
    
    while True:
        code, output = run_vibe_turn(step)
        
        if code == 0:
            log(f"🎉 Step #{step} complete. Sleeping 10s before next task...", "\033[32m")
            step += 1
            time.sleep(10)
        else:
            if "Resets in" in output:
                import re
                m_h = re.search(r"(\d+)\s*h", output)
                m_m = re.search(r"(\d+)\s*m", output)
                h = int(m_h.group(1)) if m_h else 0
                m = int(m_m.group(1)) if m_m else 0
                seconds = (h * 3600) + (m * 60) + 120
                log(f"⏳ Quota limit reached. Sleeping for {seconds}s ({h}h {m}m)...", "\033[33m")
                time.sleep(seconds)
            else:
                log(f"⚠️ agy process exited with code {code}. Retrying in 30s...", "\033[31m")
                time.sleep(30)

if __name__ == "__main__":
    main()
