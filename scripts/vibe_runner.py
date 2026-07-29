#!/usr/bin/env python3
"""
Summoner DAW — Smart Vibe Runner (Real-Time JSON Streamed)
Runs agy in autonomous loop, parsing stream-json events to render live tool calls,
thinking progress, code edits, and streaming agent text.
Includes dynamic roadmap selection, robust quota detection, exponential backoff, rate limit safety, and detailed backend log error reporting.
"""

import sys
import subprocess
import json
import time
import re
import os
import glob
from datetime import datetime, timedelta

# Ensure stdout and stderr use UTF-8 on Windows
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
LOG_FILE_PATH = os.path.join(SCRIPT_DIR, "vibe_last.log")

def get_latest_roadmap_path():
    """
    Finds the latest dated roadmap file in local/ matching ROADMAP_YYYYMMDD.md
    or ROADMAP*.md, sorting by date/name.
    """
    base_dir = os.path.dirname(SCRIPT_DIR)
    local_dir = os.path.join(base_dir, "local")
    dated_roadmaps = glob.glob(os.path.join(local_dir, "ROADMAP_[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9].md"))
    
    if dated_roadmaps:
        latest = sorted(dated_roadmaps)[-1]
        rel_path = os.path.relpath(latest, base_dir)
        return rel_path.replace("\\", "/")
    
    all_roadmaps = glob.glob(os.path.join(local_dir, "ROADMAP*.md"))
    if all_roadmaps:
        latest = sorted(all_roadmaps)[-1]
        rel_path = os.path.relpath(latest, base_dir)
        return rel_path.replace("\\", "/")
        
    return "local/ROADMAP_20260729.md"

def build_prompt(latest_roadmap):
    return (
        f"Identify and read the latest roadmap file in local/ ({latest_roadmap}) as authoritative. "
        f"If needed for context, review older roadmap files in local/. "
        f"Proceed with implementing the next incomplete logical step on {latest_roadmap}, "
        f"edit the code, run tests to verify, and commit all changes with a detailed commit message. "
        f"If all tasks in {latest_roadmap} are complete, create a new roadmap file "
        f"local/ROADMAP_YYYYMMDD.md for subsequent steps before finishing."
    )

# Case-insensitive patterns indicating rate limits or quota exhaustion
QUOTA_PATTERNS = [
    r"resets?\s+in",
    r"resets?\s+at",
    r"quota\s+exceeded",
    r"rate\s*limit",
    r"resource_exhausted",
    r"too\s+many\s+requests",
    r"429\b",
    r"over_query_limit"
]

def log(msg, color="\033[36m"):
    timestamp = datetime.now().strftime("%H:%M:%S")
    reset = "\033[0m"
    print(f"{color}[{timestamp}] {msg}{reset}", flush=True)

def parse_quota_reset_seconds(text):
    """
    Parses reset time from error output (hours, minutes, seconds).
    Returns total seconds to sleep (with safety buffer) or default if unparseable.
    """
    hours, minutes, seconds = 0, 0, 0
    found_any = False

    # Check for hours: 2h, 2 hours, 2 hrs
    m_h = re.search(r"(\d+)\s*(?:h|hr|hour|hours)", text, re.IGNORECASE)
    if m_h:
        hours = int(m_h.group(1))
        found_any = True

    # Check for minutes: 30m, 30 mins, 30 minutes
    m_m = re.search(r"(\d+)\s*(?:m|min|minute|minutes)", text, re.IGNORECASE)
    if m_m:
        minutes = int(m_m.group(1))
        found_any = True

    # Check for seconds: 45s, 45 secs, 45 seconds
    m_s = re.search(r"(\d+)\s*(?:s|sec|second|seconds)", text, re.IGNORECASE)
    if m_s:
        seconds = int(m_s.group(1))
        found_any = True

    if found_any:
        total_seconds = (hours * 3600) + (minutes * 60) + seconds + 120  # 2 minute safety buffer
        return total_seconds
    else:
        # Default safe sleep time (1 hour = 3600s) if quota limit detected but exact time couldn't be parsed
        return 3600

def is_quota_error(output_text):
    """Checks if output contains any quota or rate limiting signatures."""
    for pattern in QUOTA_PATTERNS:
        if re.search(pattern, output_text, re.IGNORECASE):
            return True
    return False

def extract_error_snippet(full_output_str):
    """Extracts non-JSON, explicit error lines, and backend logs from agy log file."""
    extracted = []
    
    # 1. Parse JSON error events from stream output
    lines = [line.strip() for line in full_output_str.splitlines() if line.strip()]
    for line in lines:
        if line.startswith("{") and line.endswith("}"):
            try:
                data = json.loads(line)
                if data.get("event") in ("error", "result"):
                    err = data.get("error") or data.get("result", {}).get("error") or data.get("message")
                    if err:
                        extracted.append(f"[CLI Result] {err}")
                continue
            except Exception:
                pass
        extracted.append(line)

    # 2. Inspect backend CLI log file for underlying error/warning details
    if os.path.exists(LOG_FILE_PATH):
        try:
            with open(LOG_FILE_PATH, "r", encoding="utf-8", errors="replace") as f:
                log_lines = f.readlines()
                for l in log_lines:
                    l_str = l.strip()
                    if (l_str.startswith("E") or "error" in l_str.lower() or "failed" in l_str.lower()) and "singleflight" not in l_str.lower():
                        extracted.append(f"[Backend Log] {l_str}")
        except Exception:
            pass

    return extracted[-8:] if extracted else lines[-8:]

def run_vibe_turn(step_num):
    latest_roadmap = get_latest_roadmap_path()
    prompt = build_prompt(latest_roadmap)
    
    log(f"🤖 Starting Vibe Turn #{step_num} (Active Roadmap: {latest_roadmap})...", "\033[1;36m")
    
    agy_flags = [
        "agy", "-p", prompt,
        "--output-format", "stream-json",
        "--dangerously-skip-permissions",
        "--log-file", LOG_FILE_PATH
    ]
    
    proc = subprocess.Popen(
        agy_flags,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        encoding="utf-8",
        errors="replace",
        bufsize=1
    )

    full_output = []

    try:
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
                        err_msg = res.get("error") or res.get("message") or status
                        log(f"  ⚠️ Turn finished with error status: {status} ({err_msg})", "\033[31m")

            except json.JSONDecodeError:
                print(f"  {line_str}", flush=True)
    except Exception as e:
        log(f"  ⚠️ Stream read error: {e}", "\033[31m")

    proc.wait()
    
    # Read full output log if available to combine with stream output
    if os.path.exists(LOG_FILE_PATH):
        try:
            with open(LOG_FILE_PATH, "r", encoding="utf-8", errors="replace") as f:
                full_output.append(f.read())
        except Exception:
            pass

    return proc.returncode, "\n".join(full_output)

def main():
    step = 1
    consecutive_failures = 0
    log("🚀 Summoner Autonomous Vibe Runner Started", "\033[1;35m")
    
    while True:
        code, output = run_vibe_turn(step)
        
        if code == 0:
            consecutive_failures = 0
            log(f"🎉 Step #{step} complete. Sleeping 10s before next task...", "\033[32m")
            step += 1
            time.sleep(10)
        else:
            consecutive_failures += 1
            
            # Prominent Error Display Banner
            log("\n❌ ------------------- TURN ERROR DETECTED -------------------", "\033[1;31m")
            error_snippet = extract_error_snippet(output)
            if error_snippet:
                log(f"   Detailed Log Snippet ({LOG_FILE_PATH}):", "\033[33m")
                for err_line in error_snippet:
                    log(f"     > {err_line}", "\033[37m")
            
            if is_quota_error(output):
                sleep_seconds = parse_quota_reset_seconds(output)
                resume_time = (datetime.now() + timedelta(seconds=sleep_seconds)).strftime("%H:%M:%S")
                log("   Detected Issue: Quota / Rate limit reached.", "\033[1;33m")
                log(f"   👉 Handling Strategy: Quota limit backoff. Sleeping for {sleep_seconds}s ({sleep_seconds // 60}m). Will resume automatically at ~{resume_time}.", "\033[1;32m")
                log("----------------------------------------------------------------\n", "\033[1;31m")
                time.sleep(sleep_seconds)
            else:
                backoff_seconds = min(30 * (2 ** (consecutive_failures - 1)), 600)
                log(f"   Detected Issue: Process exited with code {code} (Consecutive failures: {consecutive_failures}).", "\033[1;31m")
                log(f"   👉 Handling Strategy: Applying exponential backoff. Will retry in {backoff_seconds}s.", "\033[1;33m")
                log("----------------------------------------------------------------\n", "\033[1;31m")
                time.sleep(backoff_seconds)

if __name__ == "__main__":
    main()
