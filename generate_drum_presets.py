#!/usr/bin/env python3
"""
generate_drum_presets.py
Generate Summoner DrumMachineDevice .preset.toml files for all drum machine kits
from local/samplerrrrrr-07-22-13-11-56/SAMPLESWAP/DRUMS (FULL KITS)/DRUM MACHINES/
"""

import os
import re

SAMPLES_BASE = r"local\samplerrrrrr-07-22-13-11-56\SAMPLESWAP\DRUMS (FULL KITS)\DRUM MACHINES"
OUTPUT_DIR = r"local\presets\drum_machines"

# ---------------------------------------------------------------------------
# GM Drum Map (standard note assignments)
# ---------------------------------------------------------------------------
GM_MAP = {
    "kick":        35,  # Bass Drum 2
    "bd":          36,  # Bass Drum 1
    "bass":        36,
    "snare":       38,  # Snare
    "sd":          38,
    "rimshot":     37,  # Rim Shot
    "rim":         37,
    "handclap":    39,  # Hand Clap
    "clap":        39,
    "cla":         39,
    "cl_hihat":    42,  # Closed Hi-Hat
    "hhcl":        42,
    "hh":          42,
    "clshat":      42,
    "chi":         42,
    "80s-hhclose": 42,
    "open_hh":     46,  # Open Hi-Hat
    "hhop":        46,
    "hho":         46,
    "ophat":       46,
    "78-ho":       46,
    "808-ho":      46,
    "pops-hho":    46,
    "hihat-open":  46,
    "cym":         49,  # Crash Cymbal 1
    "crash":       49,
    "crashcym":    49,
    "tom1":        50,  # High Tom
    "hightom":     50,
    "thi":         50,
    "bhi":         50,
    "tom2":        48,  # Hi-Mid Tom
    "tme":         48,
    "bme":         47,
    "tom3":        45,  # Low-Mid Tom
    "tom":         45,
    "tlo":         43,  # High Floor Tom
    "blo":         43,
    "hi_conga":    62,  # Hi Conga
    "hiconga":     62,
    "conga":       63,  # Mid Conga
    "cong":        63,
    "con":         63,
    "bong":        60,
    "cowbell":     56,  # Cowbell
    "cow":         56,
    "maracas":     70,  # Maracas
    "ma":          70,
    "claves":      75,  # Claves
    "clave":       75,
    "tamb":        54,  # Tambourine
    "shake":       82,  # Shaker
    "tom5":        41,  # Low Floor Tom
    "bell":        53,  # Ride Bell
    "ping":        53,
    "pong":        53,  # repurpose
    "gui":         60,  # Generic percussion/guiro
    "me":          63,  # mid-conga fallback
    "mix":         56,  # misc
    "noisebd":     36,
    "tabla":       60,
    "fjutt":       56,
    "er1-bd":      36,
    "er1-snare":   38,
    "er1-clap":    39,
    "er1-hhcl":    42,
    "er1-hhop":    46,
    "er1-cymbnoiz":49,
    "er1-tom":     45,
    "er1-tomrock": 48,
    "er1-claves":  75,
    "er1-ping":    53,
    "er1-pong":    53,
    "er1-fjutt":   56,
    "er1-fjutt2":  56,
    "er1-tabla":   60,
    "er1-noisebd": 36,
    "snaps":       39,
    "brush_slap":  38,
    "brush_roll":  38,
    "brush_swish": 46,
    "castanets":   75,
    "clsves":      75,
    "orchestra_hit":36,
    "piatti":      49,
    "splash":      55,
    "ride":        51,
    "chinese":     52,
    "scissors":    56,
    "sonar":       56,
    "spark":       56,
    "pipe":        53,
    "plink":       53,
    "log_drum":    56,
    "timpani":     43,
    "light_shot":  50,
    "bass1":       36, "bass2":36, "bass3":36, "bass4":36, "bass5":36, "bass6":36,
    "bell1":53,  "bell2":53,  "bell3":53,
    "can":         56,
    "pop":         39,
    "pa":          39,
    "snare1":38, "snare2":38, "snare3":38, "snare4":38,
    "snare5":38, "snare6":38, "snare7":38, "snare8":38, "snare9":38,
    "rim1":37, "rim2":37, "rim3":37,
    "zip":         56,
    "zipshot":     56,
    "glassham":    56,
    "tom4":        41,
    "p400_ethnic": 60,
    "noise_pitch": 36,
    "noizhit":     56,
    "q_hifi":      56,
    "perc_pitch":  56,
    "lo_vibey":    50,
    "excellent_melodic": 56,
    "pops-bd":     36,
    "pops-sd":     38,
    "pops-rim":    37,
    "pops-hh":     42,
    "pops-con":    63,
    "pops-clave":  75,
    "pops-mix":    56,
    "78-bd":       36,
    "78-sd":       38,
    "78-rim":      37,
    "78-hh":       42,
    "78-tam":      54,
    "78-cla":      39,
    "78-cow":      56,
    "78-gui":      60,
    "78-bhi":      50,
    "78-bme":      48,
    "78-blo":      43,
    "78-me":       63,
    "808-bd":      36,
    "808-sd":      38,
    "808-hh":      42,
    "808-ho":      46,
    "808-chi":     42,
    "808-clo":     42,
    "808-cme":     63,
    "808-thi":     50,
    "808-tme":     48,
    "808-tlo":     43,
    "808-cym":     49,
    "808-clap":    39,
    "808-cla":     39,
    "808-rim":     37,
    "808-ma":      70,
    "80s-bdrum":   36,
    "80s-snare":   38,
    "80s-cowbell": 56,
    "80s-crash":   49,
    "80s-hhclose": 42,
    "80s-hhopen":  46,
    "80s-hiconga": 62,
    "80s-lowconga":43,
    "80s-midconga":63,
    "80s-tamb":    54,
    "80s-tom":     45,
    "dx100":       36,
    "hh_acoustic_closed": 42,
    "hh_acoustic_open": 46,
    "hh_acoustic_pedal": 44,
    "hh_heavy_closed": 42,
    "hh_heavy_open": 46,
    "hh_heavy_pedal": 44,
    "hh_tip_pedal": 44,
}

# Pad name display strings, mapped from resolved note
NOTE_TO_NAME = {
    35: "Bass Drum 2",
    36: "Kick",
    37: "Rim Shot",
    38: "Snare",
    39: "Hand Clap",
    40: "Snare (Elec)",
    41: "Low Floor Tom",
    42: "Hi-Hat Closed",
    43: "High Floor Tom",
    44: "Hi-Hat Pedal",
    45: "Low Tom",
    46: "Hi-Hat Open",
    47: "Low-Mid Tom",
    48: "Hi-Mid Tom",
    49: "Crash Cymbal",
    50: "High Tom",
    51: "Ride Cymbal",
    52: "Chinese Cymbal",
    53: "Ride Bell",
    54: "Tambourine",
    55: "Splash Cymbal",
    56: "Cowbell/Perc",
    57: "Crash Cymbal 2",
    60: "Hi Bongo",
    62: "Hi Conga",
    63: "Mid Conga",
    70: "Maracas",
    75: "Claves",
    82: "Shaker",
}

def resolve_note(filename: str) -> int:
    """Match filename stem to a GM note. Returns 56 (cowbell/misc) as fallback."""
    stem = os.path.splitext(filename)[0].lower()
    # Normalize common separators/case
    stem_clean = re.sub(r'[\s\-_]+', '_', stem)

    # Try longest-match from GM_MAP keys
    best_key = None
    best_len = 0
    for k in GM_MAP:
        k_norm = re.sub(r'[\s\-_]+', '_', k)
        if stem_clean.startswith(k_norm) and len(k_norm) > best_len:
            best_key = k
            best_len = len(k_norm)

    if best_key:
        return GM_MAP[best_key]

    # Heuristic fallbacks
    if "kick" in stem or "bd" in stem:
        return 36
    if "snare" in stem or "sd" in stem:
        return 38
    if "hat" in stem or "hh" in stem:
        if "open" in stem or "_o" in stem:
            return 46
        return 42
    if "crash" in stem or "cym" in stem:
        return 49
    if "rim" in stem:
        return 37
    if "clap" in stem:
        return 39
    if "tom" in stem:
        return 45
    if "cow" in stem:
        return 56
    if "clave" in stem or "cla" in stem:
        return 75
    if "tamb" in stem:
        return 54
    if "mara" in stem or "shake" in stem:
        return 70
    if "conga" in stem:
        return 63

    return 56


def pad_name_for_note(note: int, pad_index: int) -> str:
    return NOTE_TO_NAME.get(note, f"Pad {pad_index}")


def env_params_for_note(note: int) -> tuple:
    """Return (attack, decay, sustain, release, gain) tuned per drum type."""
    if note in (35, 36):          # Kick
        return (0.001, 0.35, 0.0, 0.1, 1.0)
    elif note in (38, 40):        # Snare
        return (0.001, 0.18, 0.0, 0.08, 0.9)
    elif note in (37, 39):        # Rim / Clap
        return (0.001, 0.12, 0.0, 0.05, 0.85)
    elif note in (42, 44):        # Closed HH / Pedal
        return (0.001, 0.05, 0.0, 0.03, 0.75)
    elif note == 46:              # Open HH
        return (0.001, 0.5, 0.0, 0.25, 0.8)
    elif note in (49, 52, 55, 57):# Crash/Chinese/Splash
        return (0.001, 1.2, 0.0, 0.6, 0.85)
    elif note in (51, 53):        # Ride
        return (0.001, 0.8, 0.0, 0.4, 0.8)
    elif note in (50, 48, 47, 45, 43, 41):  # Toms
        return (0.001, 0.25, 0.0, 0.12, 0.88)
    elif note in (60, 62, 63):    # Congas/Bongos
        return (0.001, 0.3, 0.0, 0.15, 0.82)
    elif note == 54:              # Tambourine
        return (0.001, 0.4, 0.0, 0.2, 0.75)
    elif note in (70, 82):        # Maracas/Shaker
        return (0.001, 0.15, 0.0, 0.08, 0.7)
    elif note == 75:              # Claves
        return (0.001, 0.2, 0.0, 0.1, 0.75)
    else:                         # Generic percussion
        return (0.001, 0.25, 0.0, 0.12, 0.8)


def sanitize_id(s: str) -> str:
    return re.sub(r'[^a-z0-9_]', '_', s.lower().replace(' ', '_').replace('-', '_'))


def generate_preset(kit_name: str, kit_dir: str, rel_samples_base: str) -> str:
    """Generate a .preset.toml for a single drum machine kit."""
    wav_files = sorted(f for f in os.listdir(kit_dir) if f.lower().endswith('.wav'))

    # Group files by resolved MIDI note
    note_to_files: dict[int, list[str]] = {}
    for fname in wav_files:
        note = resolve_note(fname)
        note_to_files.setdefault(note, []).append(fname)

    # Build TOML content
    lines = []
    lines.append(f'# Summoner DrumMachineDevice Preset')
    lines.append(f'# Kit: {kit_name}')
    lines.append(f'# Generated from: {rel_samples_base}\\{os.path.basename(kit_dir)}')
    lines.append(f'# Format: Summoner DrumMachineDevice v1.0')
    lines.append('')
    lines.append(f'[meta]')
    lines.append(f'name = "{kit_name}"')
    lines.append(f'version = "1.0"')
    lines.append(f'instrument_type = "DrumMachineDevice"')
    lines.append(f'samples_base_dir = "{rel_samples_base}\\{os.path.basename(kit_dir)}"')
    lines.append(f'pad_count = {len(note_to_files)}')
    lines.append('')

    # Sort pads by MIDI note for readability
    for pad_idx, (midi_note, fnames) in enumerate(sorted(note_to_files.items())):
        atk, dec, sus, rel, gain = env_params_for_note(midi_note)
        pad_label = pad_name_for_note(midi_note, pad_idx)
        lines.append(f'[[pads]]')
        lines.append(f'name = "{pad_label}"')
        lines.append(f'midi_note = {midi_note}')
        lines.append(f'gain = {gain:.2f}')
        lines.append(f'[pads.envelope]')
        lines.append(f'attack  = {atk:.4f}')
        lines.append(f'decay   = {dec:.4f}')
        lines.append(f'sustain = {sus:.4f}')
        lines.append(f'release = {rel:.4f}')
        for fname in fnames:
            lines.append(f'[[pads.samples]]')
            lines.append(f'sample_path = "{fname}"')
            lines.append(f'lokey = {midi_note}')
            lines.append(f'hikey = {midi_note}')
            lines.append(f'pitch_keycenter = {midi_note}')
            lines.append(f'loop_mode = "NoLoop"')
        lines.append('')

    return '\n'.join(lines)


def main():
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    # Canonical relative path from repo root for sample references
    rel_base = r"local\samplerrrrrr-07-22-13-11-56\SAMPLESWAP\DRUMS (FULL KITS)\DRUM MACHINES"

    kit_dirs = [
        d for d in os.listdir(SAMPLES_BASE)
        if os.path.isdir(os.path.join(SAMPLES_BASE, d))
    ]

    manifests = []
    for kit_name in sorted(kit_dirs):
        kit_dir = os.path.join(SAMPLES_BASE, kit_name)
        wav_count = len([f for f in os.listdir(kit_dir) if f.lower().endswith('.wav')])
        if wav_count == 0:
            print(f"  [SKIP] {kit_name} — no WAV files")
            continue

        toml_content = generate_preset(kit_name, kit_dir, rel_base)
        safe_name = sanitize_id(kit_name)
        out_path = os.path.join(OUTPUT_DIR, f"{safe_name}.preset.toml")
        with open(out_path, 'w', encoding='utf-8') as f:
            f.write(toml_content)
        print(f"  [OK] {kit_name} -> {out_path} ({wav_count} samples)")
        manifests.append((kit_name, safe_name, wav_count))

    # Write index manifest
    manifest_lines = [
        '# Drum Machine Kit Index',
        '# Auto-generated manifest of all DrumMachineDevice presets',
        '',
        '[[kits]]',
    ]
    for kit_name, safe_name, wav_count in manifests:
        manifest_lines += [
            f'  name = "{kit_name}"',
            f'  preset_file = "{safe_name}.preset.toml"',
            f'  sample_count = {wav_count}',
            '',
            '[[kits]]',
        ]
    manifest_lines.pop()  # remove trailing [[kits]]
    manifest_path = os.path.join(OUTPUT_DIR, "INDEX.toml")
    with open(manifest_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(manifest_lines))
    print(f"\n  [INDEX] -> {manifest_path}")
    print(f"\nTotal: {len(manifests)} drum machine kits converted.")


if __name__ == '__main__':
    main()
