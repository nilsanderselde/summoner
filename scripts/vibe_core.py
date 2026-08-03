#!/usr/bin/env python3
"""
Summoner DAW — Shared Vibe Runner Core Library
Contains shared execution logic, JSON stream parsing, quota detection,
exponential backoff, error extraction, and logging for autonomous runners.
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

QUOTA_PATTERNS = [
    r"resets?\s+in",
    r"resets?\s+at",
    r"quota\s+exceeded",
    r"rate\s*limit",
    r"resource_exhausted",
    r"too\s+many\s+requests",
    r"429\b",
    r"503\b",
    r"unavailable",
    r"over_query_limit"
]

def log(msg, color="\033[36m"):
    timestamp = datetime.now().strftime("%H:%M:%S")
    reset = "\033[0m"
    print(f"{color}[{timestamp}] {msg}{reset}", flush=True)

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

def parse_quota_reset_seconds(text):
    """
    Parses the quota reset time from error output.
    Strategy (in priority order):
      1. Explicit "Resets in X" or "Resets at Y" clauses.
      2. Relative duration strings: "3h18m38s", "45m", "90s" etc.
      3. Absolute date+time: "M/D/YYYY, H:MM:SS AM/PM"
      4. Absolute time only: "H:MM:SS AM/PM" or "HH:MM:SS" (Fallback, prone to matching log timestamps)
    Always adds a 90-second safety buffer. Caps at 24 hours max.
    """
    now = datetime.now()
    BUFFER = 90   # seconds of extra cushion after reset time
    MAX_SLEEP = 86400  # 24 hours hard cap

    # 1. Target explicit "Resets in" or "Resets at" sentences first to avoid log noise
    m_resets = re.search(r"resets?\s+(in|at)\s+(.*?)(?:\.|,|;|:|$)", text, re.IGNORECASE)
    if m_resets:
        preposition = m_resets.group(1).lower()
        target_str = m_resets.group(2)

        if preposition == "in":
            h_match = re.search(r"(\d+)\s*(?:h|hr|hour|hours)(?![a-z])", target_str, re.IGNORECASE)
            m_match = re.search(r"(\d+)\s*(?:m|min|minute|minutes)(?![a-z])", target_str, re.IGNORECASE)
            s_match = re.search(r"(\d+)\s*(?:s|sec|second|seconds)(?![a-z])", target_str, re.IGNORECASE)
            
            h = int(h_match.group(1)) if h_match else 0
            m = int(m_match.group(1)) if m_match else 0
            s = int(s_match.group(1)) if s_match else 0
            
            if h or m or s:
                return min((h * 3600) + (m * 60) + s + BUFFER, MAX_SLEEP)
                
        elif preposition == "at":
            t_match = re.search(r"(\d{1,2}):(\d{2}):(\d{2})\s*(AM|PM)?", target_str, re.IGNORECASE)
            if t_match:
                hour, minute, second = int(t_match.group(1)), int(t_match.group(2)), int(t_match.group(3))
                ampm = (t_match.group(4) or "").upper()
                if ampm == "PM" and hour != 12:
                    hour += 12
                elif ampm == "AM" and hour == 12:
                    hour = 0
                try:
                    reset_dt = now.replace(hour=hour, minute=minute, second=second, microsecond=0)
                    delta = (reset_dt - now).total_seconds() + BUFFER
                    if delta < 0:
                        delta += 86400
                    return min(int(delta), MAX_SLEEP)
                except Exception:
                    pass

    # 2. General Relative duration strings: "2h", "30m", "90s", "3h18m38s"
    found_any = False
    h, m, s = 0, 0, 0

    m_h = re.search(r"(\d+)\s*(?:h|hr|hour|hours)(?![a-z])", text, re.IGNORECASE)
    if m_h:
        h = int(m_h.group(1))
        found_any = True

    m_m = re.search(r"(\d+)\s*(?:m|min|minute|minutes)(?![a-z])", text, re.IGNORECASE)
    if m_m:
        m = int(m_m.group(1))
        found_any = True

    m_s = re.search(r"(\d+)\s*(?:s|sec|second|seconds)(?![a-z])", text, re.IGNORECASE)
    if m_s:
        s = int(m_s.group(1))
        found_any = True

    if found_any:
        total = (h * 3600) + (m * 60) + s + BUFFER
        return min(total, MAX_SLEEP)

    # 3. Absolute date+time: e.g. "7/29/2026, 4:13:12 PM"
    m_abs = re.search(
        r"(\d{1,2})/(\d{1,2})/(\d{4}),?\s+(\d{1,2}):(\d{2}):(\d{2})\s*(AM|PM)?",
        text, re.IGNORECASE
    )
    if m_abs:
        month, day, year = int(m_abs.group(1)), int(m_abs.group(2)), int(m_abs.group(3))
        hour, minute, second = int(m_abs.group(4)), int(m_abs.group(5)), int(m_abs.group(6))
        ampm = (m_abs.group(7) or "").upper()
        if ampm == "PM" and hour != 12:
            hour += 12
        elif ampm == "AM" and hour == 12:
            hour = 0
        try:
            reset_dt = now.replace(year=year, month=month, day=day,
                                   hour=hour, minute=minute, second=second, microsecond=0)
            delta = (reset_dt - now).total_seconds() + BUFFER
            if delta < 0:
                delta += 86400
            return min(int(delta), MAX_SLEEP)
        except Exception:
            pass

    # 4. Absolute time only: "4:13:12 PM" or "16:13:12"
    m_time = re.search(
        r"\b(\d{1,2}):(\d{2}):(\d{2})\s*(AM|PM)?\b",
        text, re.IGNORECASE
    )
    if m_time:
        hour, minute, second = int(m_time.group(1)), int(m_time.group(2)), int(m_time.group(3))
        ampm = (m_time.group(4) or "").upper()
        if ampm == "PM" and hour != 12:
            hour += 12
        elif ampm == "AM" and hour == 12:
            hour = 0
        try:
            reset_dt = now.replace(hour=hour, minute=minute, second=second, microsecond=0)
            delta = (reset_dt - now).total_seconds() + BUFFER
            if delta < 0:
                delta += 86400
            return min(int(delta), MAX_SLEEP)
        except Exception:
            pass

    return 3600

def is_quota_error(output_text):
    """Checks if output contains any quota or rate limiting signatures."""
    for pattern in QUOTA_PATTERNS:
        if re.search(pattern, output_text, re.IGNORECASE):
            return True
    return False

def extract_error_snippet(full_output_str, log_file_path):
    """Extracts non-JSON, explicit error lines, and backend logs from agy log file."""
    extracted = []
    
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

    if os.path.exists(log_file_path):
        try:
            with open(log_file_path, "r", encoding="utf-8", errors="replace") as f:
                log_lines = f.readlines()
                for l in log_lines:
                    l_str = l.strip()
                    if (l_str.startswith("E") or "error" in l_str.lower() or "failed" in l_str.lower()) and "singleflight" not in l_str.lower():
                        extracted.append(f"[Backend Log] {l_str}")
        except Exception:
            pass

    return extracted[-8:] if extracted else lines[-8:]

def run_vibe_turn(step_num, build_prompt_fn, log_file_path):
    latest_roadmap = get_latest_roadmap_path()
    prompt = build_prompt_fn(latest_roadmap)
    
    log(f"🤖 Starting Vibe Turn #{step_num} (Active Roadmap: {latest_roadmap})...", "\033[1;36m")
    
    agy_flags = [
        "agy", "-p", prompt,
        "--add-dir", ".",
        "--output-format", "stream-json",
        "--dangerously-skip-permissions",
        "--log-file", log_file_path
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
    
    if os.path.exists(log_file_path):
        try:
            with open(log_file_path, "r", encoding="utf-8", errors="replace") as f:
                full_output.append(f.read())
        except Exception:
            pass

    return proc.returncode, "\n".join(full_output)

def run_vibe_loop(build_prompt_fn, log_file_name, runner_title):
    log_file_path = os.path.join(SCRIPT_DIR, log_file_name)
    step = 1
    consecutive_failures = 0
    log(f"🚀 {runner_title} Started", "\033[1;35m")
    
    while True:
        code, output = run_vibe_turn(step, build_prompt_fn, log_file_path)
        
        if code == 0:
            consecutive_failures = 0
            log(f"🎉 Step #{step} complete. Sleeping 10s before next task...", "\033[32m")
            step += 1
            time.sleep(10)
        else:
            consecutive_failures += 1
            
            log("\n❌ ------------------- TURN ERROR DETECTED -------------------", "\033[1;31m")
            error_snippet = extract_error_snippet(output, log_file_path)
            if error_snippet:
                log(f"   Detailed Log Snippet ({log_file_path}):", "\033[33m")
                for err_line in error_snippet:
                    log(f"     > {err_line}", "\033[37m")
            
            if is_quota_error(output):
                sleep_seconds = parse_quota_reset_seconds(output)
                resume_time = (datetime.now() + timedelta(seconds=sleep_seconds)).strftime("%H:%M:%S")
                log("   Detected Issue: Quota / Rate limit / Backend 503 Service Outage reached.", "\033[1;33m")
                log(f"   👉 Handling Strategy: Service outage / Quota backoff. Sleeping for {sleep_seconds}s ({sleep_seconds // 60}m). Will resume automatically at ~{resume_time}.", "\033[1;32m")
                log("----------------------------------------------------------------\n", "\033[1;31m")
                time.sleep(sleep_seconds)
            else:
                backoff_seconds = min(30 * (2 ** (consecutive_failures - 1)), 600)
                log(f"   Detected Issue: Process exited with code {code} (Consecutive failures: {consecutive_failures}).", "\033[1;31m")
                log(f"   👉 Handling Strategy: Applying exponential backoff. Will retry in {backoff_seconds}s.", "\033[1;33m")
                log("----------------------------------------------------------------\n", "\033[1;31m")
                time.sleep(backoff_seconds)
