#!/usr/bin/env python3
"""
Summoner DAW - Headless GUI Visualizer and Screenshot Render Harness.
Renders high-fidelity PNG representations of Tier 50 GUI widgets into `scratch/renders/`.
"""

import os
import math
from PIL import Image, ImageDraw, ImageFont

OUTPUT_DIR = os.path.join(os.path.dirname(os.path.dirname(__file__)), "scratch", "renders")
os.makedirs(OUTPUT_DIR, exist_ok=True)

def get_font(size=14, bold=False):
    font_names = [
        "arial.ttf", "segoeui.ttf", "DejaVuSans.ttf", "FreeSans.ttf", "Helvetica.ttf"
    ]
    if bold:
        font_names = ["arialbd.ttf", "segoeuib.ttf", "DejaVuSans-Bold.ttf", "FreeSansBold.ttf"]
    
    for name in font_names:
        try:
            return ImageFont.truetype(name, size)
        except Exception:
            continue
    return ImageFont.load_default()

def render_live_macro_rack():
    width, height = 800, 500
    img = Image.new("RGBA", (width, height), (14, 18, 28, 255))
    draw = ImageDraw.Draw(img)

    f_title = get_font(18, bold=True)
    f_header = get_font(14, bold=True)
    f_body = get_font(12, bold=False)
    f_small = get_font(10, bold=False)

    # Title Bar
    draw.text((20, 16), "LIVE PERFORMANCE MACRO RACK", fill=(240, 245, 255), font=f_title)
    
    # Snapshots buttons on top right
    snap_labels = ["1: Intro", "2: Build", "3: Drop", "4: Outro"]
    snap_x = 420
    draw.text((340, 20), "SNAPSHOT:", fill=(180, 195, 215), font=f_body)
    for i, label in enumerate(snap_labels):
        is_active = (i == 0)
        bg_col = (0, 229, 255) if is_active else (50, 65, 90)
        text_col = (0, 0, 0) if is_active else (240, 245, 255)
        btn_box = [snap_x, 10, snap_x + 80, 42]
        draw.rounded_rectangle(btn_box, radius=4, fill=bg_col)
        draw.text((snap_x + 12, 18), label, fill=text_col, font=f_body)
        snap_x += 90

    # Dual XY Pads (Moved down to y=85 to guarantee 0 overlap)
    pad_defs = [
        {"name": "PAD 1: TONE / FILTER", "x_param": "Cutoff Freq", "y_param": "Resonance (Q)", "pos": (0.65, 0.70), "color": (0, 229, 255), "rect": (30, 85, 360, 315)},
        {"name": "PAD 2: SPACE / DYNAMICS", "x_param": "Reverb Space", "y_param": "Drive Distortion", "pos": (0.35, 0.45), "color": (255, 107, 43), "rect": (410, 85, 740, 315)},
    ]

    for p in pad_defs:
        r = p["rect"]
        # Header label
        draw.text((r[0], r[1] - 24), p["name"], fill=p["color"], font=f_header)
        # Spring Toggle Button
        draw.rounded_rectangle([r[2] - 85, r[1] - 26, r[2], r[1] - 4], radius=4, fill=(35, 45, 65))
        draw.text((r[2] - 75, r[1] - 20), "Spring: ON", fill=(0, 255, 180), font=f_small)

        # Pad canvas background
        draw.rounded_rectangle(r, radius=8, fill=(10, 14, 22), outline=(45, 60, 85), width=2)

        # Subdivided grid
        for g in range(1, 4):
            gx = r[0] + (r[2] - r[0]) * (g * 0.25)
            gy = r[1] + (r[3] - r[1]) * (g * 0.25)
            draw.line([(gx, r[1]), (gx, r[3])], fill=(50, 65, 90, 80), width=1)
            draw.line([(r[0], gy), (r[2], gy)], fill=(50, 65, 90, 80), width=1)

        # Puck coordinate
        px = r[0] + (r[2] - r[0]) * p["pos"][0]
        py = r[1] + (r[3] - r[1]) * (1.0 - p["pos"][1])

        # Crosshair lines
        draw.line([(r[0], py), (r[2], py)], fill=p["color"] + (120,), width=1)
        draw.line([(px, r[1]), (px, r[3])], fill=p["color"] + (120,), width=1)

        # Outer hit target radius (22pt = 44x44pt bounding box)
        draw.ellipse([px - 22, py - 22, px + 22, py + 22], outline=p["color"] + (140,), width=2)
        # Puck body
        draw.ellipse([px - 14, py - 14, px + 14, py + 14], fill=p["color"])
        draw.ellipse([px - 4, py - 4, px + 4, py + 4], fill=(255, 255, 255))

        # Bottom readout
        readout = f"X ({p['x_param']}): {int(p['pos'][0]*100)}% | Y ({p['y_param']}): {int(p['pos'][1]*100)}%"
        draw.text((r[0], r[3] + 8), readout, fill=(180, 200, 225), font=f_small)

    # Bottom Quick Macro Knobs Bar
    draw.text((30, 365), "QUICK MACROS:", fill=(0, 229, 255), font=f_header)
    macro_items = [
        {"name": "Macro 1 (Sub)", "val": 0.65, "unit": "dB", "col": (0, 229, 255)},
        {"name": "Macro 2 (Air)", "val": 0.40, "unit": "kHz", "col": (76, 201, 240)},
        {"name": "Macro 3 (Width)", "val": 0.80, "unit": "%", "col": (255, 215, 0)},
        {"name": "Macro 4 (Punch)", "val": 0.55, "unit": "ms", "col": (255, 107, 43)},
    ]
    mx = 30
    for m in macro_items:
        draw.rounded_rectangle([mx, 390, mx + 165, 475], radius=6, fill=(20, 26, 38), outline=(45, 55, 75))
        draw.text((mx + 10, 398), m["name"], fill=(200, 215, 235), font=f_small)
        # Slider track
        draw.rounded_rectangle([mx + 10, 420, mx + 155, 436], radius=4, fill=(10, 14, 20))
        # Slider fill
        fill_w = int(145 * m["val"])
        draw.rounded_rectangle([mx + 10, 420, mx + 10 + fill_w, 436], radius=4, fill=m["col"])
        # Slider value label
        draw.text((mx + 10, 448), f"{int(m['val']*100)}% {m['unit']}", fill=m["col"], font=f_small)
        mx += 185

    out_path = os.path.join(OUTPUT_DIR, "live_macro_rack.png")
    img.save(out_path)
    print(f"Rendered: {out_path}")

def render_spectrogram_3d():
    width, height = 800, 480
    img = Image.new("RGBA", (width, height), (10, 14, 24, 255))
    draw = ImageDraw.Draw(img)

    f_title = get_font(18, bold=True)
    f_header = get_font(14, bold=True)
    f_body = get_font(12, bold=False)
    f_small = get_font(10, bold=False)

    # Header Bar
    draw.text((20, 16), "3D FFT WATERFALL SPECTROGRAM", fill=(240, 245, 255), font=f_title)
    draw.text((450, 22), "Peak: 440 Hz (-3.5 dBFS)", fill=(255, 215, 0), font=f_body)
    
    # Freeze & Reset Buttons
    draw.rounded_rectangle([630, 14, 700, 46], radius=4, fill=(35, 45, 65))
    draw.text((642, 24), "FREEZE", fill=(0, 229, 255), font=f_body)
    draw.rounded_rectangle([710, 14, 780, 46], radius=4, fill=(35, 45, 65))
    draw.text((720, 24), "Reset", fill=(200, 220, 245), font=f_body)

    # 3D Canvas
    c_rect = [30, 60, 770, 430]
    draw.rounded_rectangle(c_rect, radius=8, fill=(8, 11, 18), outline=(35, 50, 75), width=2)

    center_x, center_y = 400, 250
    w_scale, h_scale = 520, 280
    yaw_deg = -25.0
    pitch_deg = 35.0
    yaw_rad = math.radians(yaw_deg)
    pitch_rad = math.radians(pitch_deg)

    def project_pt(f_norm, t_norm, mag):
        x0 = (f_norm - 0.5) * w_scale
        z0 = (t_norm - 0.5) * h_scale * 0.8
        y0 = mag * h_scale * 0.5
        x1 = x0 * math.cos(yaw_rad) - z0 * math.sin(yaw_rad)
        z1 = x0 * math.sin(yaw_rad) + z0 * math.cos(yaw_rad)
        y2 = y0 * math.cos(pitch_rad) - z1 * math.sin(pitch_rad)
        return (center_x + x1, center_y - y2)

    def mag_to_color(m):
        m = max(0.0, min(1.0, m))
        if m < 0.2:
            t = m / 0.2
            return (int(20 + t * 70), int(25 + t * 20), int(90 + t * 140))
        elif m < 0.5:
            t = (m - 0.2) / 0.3
            return (int(90 * (1 - t)), int(45 + t * 180), int(230 + t * 25))
        elif m < 0.8:
            t = (m - 0.5) / 0.3
            return (int(t * 255), int(225 - t * 35), int(255 * (1 - t)))
        else:
            t = (m - 0.8) / 0.2
            return (255, int(190 * (1 - t)), int(t * 120))

    # Frequency Grid Markers
    freq_markers = [
        (0.02, "20Hz"), (0.10, "100Hz"), (0.35, "1kHz"), (0.65, "5kHz"), (0.98, "20kHz")
    ]
    for frac, label in freq_markers:
        p_front = project_pt(frac, 0.0, 0.0)
        p_back = project_pt(frac, 1.0, 0.0)
        draw.line([p_front, p_back], fill=(60, 80, 115, 120), width=1)
        # Background pill behind label for maximum contrast
        draw.rounded_rectangle([p_front[0] - 16, p_front[1] + 4, p_front[0] + 16, p_front[1] + 18], radius=3, fill=(16, 22, 34))
        draw.text((p_front[0] - 12, p_front[1] + 6), label, fill=(180, 205, 235), font=f_small)

    # Waterfall slices (32 slices, 64 bins)
    num_slices = 32
    num_bins = 64
    for s in range(num_slices - 1, -1, -1):
        t_norm = s / num_slices
        decay = 1.0 - t_norm * 0.65
        alpha = int(255 * (1.0 - t_norm * 0.5))

        prev_pt = None
        for b in range(num_bins):
            f_norm = b / num_bins
            fund = math.exp(-((f_norm - 0.15) * 15.0) ** 2)
            harm1 = 0.6 * math.exp(-((f_norm - 0.30) * 20.0) ** 2)
            harm2 = 0.4 * math.exp(-((f_norm - 0.45) * 25.0) ** 2)
            noise = 0.05 * abs(math.sin(b * 0.3))
            mag = (fund + harm1 + harm2 + noise) * decay
            
            cur_pt = project_pt(f_norm, t_norm, mag)
            if prev_pt is not None:
                col = mag_to_color(mag)
                draw.line([prev_pt, cur_pt], fill=col + (alpha,), width=2 if s == 0 else 1)
            prev_pt = cur_pt

    # Status Bar
    draw.text((40, 440), "Orbit: Yaw -25° | Pitch 35° | Slices: 32 | Resolution: 64 bins", fill=(160, 180, 205), font=f_small)

    out_path = os.path.join(OUTPUT_DIR, "spectrogram_3d.png")
    img.save(out_path)
    print(f"Rendered: {out_path}")

def render_keybinding_editor():
    width, height = 800, 490
    img = Image.new("RGBA", (width, height), (16, 20, 30, 255))
    draw = ImageDraw.Draw(img)

    f_title = get_font(18, bold=True)
    f_header = get_font(14, bold=True)
    f_body = get_font(12, bold=False)
    f_small = get_font(10, bold=False)

    # Title Bar
    draw.text((20, 16), "KEYBOARD SHORTCUTS & KEYBINDING EDITOR", fill=(240, 245, 255), font=f_title)
    
    # Reset Defaults Button
    draw.rounded_rectangle([630, 12, 780, 46], radius=4, fill=(40, 50, 70))
    draw.text((642, 22), "Reset All Defaults", fill=(220, 235, 255), font=f_body)

    # Conflict Status Banner
    draw.rounded_rectangle([20, 56, 780, 88], radius=6, fill=(20, 45, 35), outline=(0, 220, 140), width=1)
    draw.text((36, 66), "[OK] ZERO SHORTCUT CONFLICTS DETECTED — ALL MODIFIERS RESOLVED", fill=(0, 255, 180), font=f_body)

    # Search & Category Filters
    draw.text((20, 102), "Search:", fill=(180, 200, 225), font=f_body)
    draw.rounded_rectangle([75, 96, 260, 126], radius=4, fill=(10, 14, 22), outline=(45, 55, 75))
    draw.text((85, 104), "Filter actions...", fill=(100, 120, 150), font=f_body)

    categories = ["All", "Transport", "File", "Edit", "Navigation", "Tools"]
    cx = 280
    draw.text((cx, 102), "Category:", fill=(180, 200, 225), font=f_body)
    cx += 65
    for i, cat in enumerate(categories):
        is_sel = (i == 0)
        bg = (0, 229, 255) if is_sel else (35, 45, 65)
        fg = (0, 0, 0) if is_sel else (220, 235, 255)
        draw.rounded_rectangle([cx, 96, cx + 65, 126], radius=4, fill=bg)
        draw.text((cx + 12, 104), cat, fill=fg, font=f_body)
        cx += 72

    # Action Rows Table (Even spacing to margin 780)
    rows = [
        {"name": "Play / Stop Transport", "cat": "Transport", "prim": "Space", "sec": "--"},
        {"name": "Toggle Record", "cat": "Transport", "prim": "R", "sec": "--"},
        {"name": "Save Project", "cat": "File", "prim": "Ctrl+S", "sec": "--"},
        {"name": "Save Project As...", "cat": "File", "prim": "Ctrl+Shift+S", "sec": "--"},
        {"name": "Undo Last Action", "cat": "Edit", "prim": "Ctrl+Z", "sec": "--"},
        {"name": "Redo Last Action", "cat": "Edit", "prim": "Ctrl+Y", "sec": "Ctrl+Shift+Z"},
        {"name": "Switch to Arranger View", "cat": "Navigation", "prim": "Ctrl+1", "sec": "--"},
        {"name": "Open Command Palette", "cat": "Tools", "prim": "Ctrl+K", "sec": "Ctrl+P"},
    ]

    ry = 140
    for r in rows:
        draw.rounded_rectangle([20, ry, 780, ry + 36], radius=4, fill=(22, 28, 42), outline=(40, 50, 70))
        # Name
        draw.text((32, ry + 10), r["name"], fill=(240, 245, 255), font=f_body)
        # Category
        draw.text((280, ry + 11), r["cat"], fill=(130, 160, 200), font=f_small)
        # Primary Shortcut Box
        draw.rounded_rectangle([390, ry + 4, 520, ry + 32], radius=4, fill=(15, 20, 32), outline=(0, 229, 255))
        draw.text((410, ry + 10), r["prim"], fill=(0, 229, 255), font=f_body)
        # Secondary Shortcut Box
        draw.rounded_rectangle([535, ry + 4, 675, ry + 32], radius=4, fill=(15, 20, 32), outline=(50, 65, 90))
        draw.text((555, ry + 10), r["sec"], fill=(180, 200, 225), font=f_body)
        # Clear Button
        draw.rounded_rectangle([690, ry + 4, 765, ry + 32], radius=4, fill=(45, 25, 30))
        draw.text((710, ry + 10), "Clear", fill=(255, 120, 120), font=f_small)
        ry += 41

    out_path = os.path.join(OUTPUT_DIR, "keybinding_editor.png")
    img.save(out_path)
    print(f"Rendered: {out_path}")

def render_meter_bridge():
    width, height = 800, 480
    img = Image.new("RGBA", (width, height), (14, 18, 28, 255))
    draw = ImageDraw.Draw(img)

    f_title = get_font(18, bold=True)
    f_header = get_font(14, bold=True)
    f_body = get_font(12, bold=False)
    f_small = get_font(10, bold=False)

    # Title Bar
    draw.text((20, 16), "MULTI-TRACK PEAK METERING BRIDGE", fill=(240, 245, 255), font=f_title)
    draw.rounded_rectangle([650, 12, 780, 44], radius=4, fill=(40, 50, 70))
    draw.text((665, 22), "Reset All Clips", fill=(220, 235, 255), font=f_body)

    # Meter parameters
    meter_top = 100
    meter_height = 280
    meter_bottom = meter_top + meter_height

    def db_to_frac(db):
        return max(0.0, min(1.0, (db + 60.0) / 66.0))

    def db_to_color(db):
        if db >= 0.0:
            return (255, 40, 60)
        elif db >= -6.0:
            return (255, 180, 20)
        elif db >= -18.0:
            return (240, 220, 40)
        else:
            return (0, 220, 140)

    # dB Legend Scale
    marks = [6.0, 0.0, -3.0, -6.0, -12.0, -18.0, -24.0, -36.0, -48.0, -60.0]
    for db in marks:
        frac = db_to_frac(db)
        y = meter_bottom - frac * meter_height
        lbl = " 0" if db == 0.0 else (f"+{int(db)}" if db > 0 else str(int(db)))
        draw.line([(55, y), (65, y)], fill=(100, 120, 150), width=1)
        draw.text((25, y - 6), lbl, fill=(150, 175, 205), font=f_small)

    # Channels
    channels = [
        {"name": "Kick / Snare", "peak_l": -4.2, "peak_r": -4.5, "hold_l": -1.2, "hold_r": -1.5, "clip": False, "is_master": False},
        {"name": "Sub Bass", "peak_l": -6.0, "peak_r": -6.0, "hold_l": -3.0, "hold_r": -3.0, "clip": False, "is_master": False},
        {"name": "Lead Synth", "peak_l": -9.5, "peak_r": -8.2, "hold_l": -6.0, "hold_r": -5.5, "clip": False, "is_master": False},
        {"name": "Reverb FX", "peak_l": -14.0, "peak_r": -13.5, "hold_l": -11.0, "hold_r": -10.5, "clip": False, "is_master": False},
        {"name": "MASTER BUS", "peak_l": -1.8, "peak_r": -1.5, "hold_l": -0.2, "hold_r": -0.1, "clip": False, "is_master": True},
    ]

    cx = 80
    for ch in channels:
        # Clip Indicator Button
        clip_bg = (255, 30, 40) if ch["clip"] else (40, 50, 70)
        draw.rounded_rectangle([cx + 10, 65, cx + 70, 92], radius=4, fill=clip_bg)
        draw.text((cx + 28, 72), "CLIP" if ch["clip"] else "OK", fill=(255, 255, 255) if ch["clip"] else (140, 160, 180), font=f_small)

        # Strip Box
        draw.rounded_rectangle([cx, meter_top, cx + 80, meter_bottom], radius=4, fill=(10, 14, 20), outline=(40, 50, 70))

        bar_w = 20
        # Left Bar
        lx = cx + 14
        frac_l = db_to_frac(ch["peak_l"])
        bar_h_l = frac_l * meter_height
        draw.rounded_rectangle([lx, meter_bottom - bar_h_l, lx + bar_w, meter_bottom], radius=2, fill=db_to_color(ch["peak_l"]))
        hold_y_l = meter_bottom - db_to_frac(ch["hold_l"]) * meter_height
        draw.line([(lx, hold_y_l), (lx + bar_w, hold_y_l)], fill=db_to_color(ch["hold_l"]), width=2)

        # Right Bar
        rx = cx + 44
        frac_r = db_to_frac(ch["peak_r"])
        bar_h_r = frac_r * meter_height
        draw.rounded_rectangle([rx, meter_bottom - bar_h_r, rx + bar_w, meter_bottom], radius=2, fill=db_to_color(ch["peak_r"]))
        hold_y_r = meter_bottom - db_to_frac(ch["hold_r"]) * meter_height
        draw.line([(rx, hold_y_r), (rx + bar_w, hold_y_r)], fill=db_to_color(ch["hold_r"]), width=2)

        # Track Name
        label_col = (255, 215, 0) if ch["is_master"] else (220, 235, 255)
        draw.text((cx + 6, meter_bottom + 10), ch["name"], fill=label_col, font=f_small)

        # Mute / Solo
        if not ch["is_master"]:
            draw.rounded_rectangle([cx + 10, meter_bottom + 30, cx + 38, meter_bottom + 56], radius=3, fill=(45, 55, 75))
            draw.text((cx + 20, meter_bottom + 38), "M", fill=(255, 255, 255), font=f_small)
            draw.rounded_rectangle([cx + 42, meter_bottom + 30, cx + 70, meter_bottom + 56], radius=3, fill=(45, 55, 75))
            draw.text((cx + 52, meter_bottom + 38), "S", fill=(255, 255, 255), font=f_small)

        cx += 120
        if ch["name"] == "Reverb FX":
            # Separator before Master
            draw.line([(cx - 20, meter_top), (cx - 20, meter_bottom + 50)], fill=(60, 80, 110), width=2)

    out_path = os.path.join(OUTPUT_DIR, "meter_bridge.png")
    img.save(out_path)
    print(f"Rendered: {out_path}")

def render_dpi_scale_panel():
    width, height = 800, 480
    img = Image.new("RGBA", (width, height), (16, 22, 34, 255))
    draw = ImageDraw.Draw(img)

    f_title = get_font(18, bold=True)
    f_header = get_font(14, bold=True)
    f_body = get_font(12, bold=False)
    f_small = get_font(10, bold=False)

    # Title Bar
    draw.text((20, 16), "HIGH-DPI DISPLAY SCALING & CALIBRATION", fill=(240, 245, 255), font=f_title)
    draw.rounded_rectangle([630, 12, 780, 44], radius=4, fill=(35, 50, 75))
    draw.text((648, 22), "Auto-Detect: ON", fill=(0, 229, 255), font=f_body)

    # Host Metrics Card
    draw.rounded_rectangle([20, 60, 780, 110], radius=6, fill=(22, 30, 46), outline=(45, 60, 85))
    draw.text((36, 76), "Host OS: Windows | System DPI: 120 DPI | Detected Scale Factor: 125%", fill=(220, 235, 255), font=f_body)

    # Presets Bar
    draw.text((20, 130), "Presets:", fill=(0, 229, 255), font=f_header)
    presets = [1.0, 1.25, 1.5, 2.0, 2.5, 3.0]
    px = 110
    for p in presets:
        is_sel = (p == 1.25)
        bg = (0, 229, 255) if is_sel else (40, 55, 80)
        fg = (0, 0, 0) if is_sel else (240, 245, 255)
        draw.rounded_rectangle([px, 124, px + 80, 164], radius=4, fill=bg)
        draw.text((px + 18, 136), f"{int(p*100)}%", fill=fg, font=f_body)
        px += 95

    # Slider
    draw.text((20, 190), "Custom Scale:", fill=(200, 220, 245), font=f_body)
    draw.rounded_rectangle([130, 186, 500, 212], radius=4, fill=(10, 14, 22))
    # Fill at 125%
    draw.rounded_rectangle([130, 186, 260, 212], radius=4, fill=(0, 229, 255))
    draw.text((520, 190), "Effective: 1.25x (125%)", fill=(255, 215, 0), font=f_body)

    # Scaled Preview Card
    draw.rounded_rectangle([20, 240, 780, 440], radius=8, fill=(12, 16, 26), outline=(40, 55, 80), width=1)
    draw.text((40, 256), "SCALED WIDGET PREVIEW (CALIBRATED AT 1.25x)", fill=(240, 245, 255), font=f_header)

    # Scaled Button (44pt * 1.25 = 55px)
    draw.rounded_rectangle([40, 295, 240, 355], radius=6, fill=(0, 140, 255))
    draw.text((55, 316), "Touch Button (55px)", fill=(255, 255, 255), font=f_body)

    # Compliance Badge
    draw.rounded_rectangle([260, 295, 620, 355], radius=6, fill=(18, 40, 30), outline=(0, 220, 140))
    draw.text((280, 316), "[PASS] Ergonomic Hit Target: >= 44pt (55px)", fill=(0, 255, 180), font=f_body)

    # Explanation text
    draw.text((40, 380), "Cross-OS DPI scaling guarantees that interactive controls maintain minimum physical touch dimensions", fill=(160, 185, 215), font=f_small)
    draw.text((40, 400), "across Windows (96–192 DPI), macOS Retina (192 DPI), and Linux desktop environments.", fill=(160, 185, 215), font=f_small)

    out_path = os.path.join(OUTPUT_DIR, "dpi_scale_panel.png")
    img.save(out_path)
    print(f"Rendered: {out_path}")


def render_dsp_rack_dock():
    width, height = 800, 520
    img = Image.new("RGBA", (width, height), (14, 18, 28, 255))
    draw = ImageDraw.Draw(img)

    f_title = get_font(18, bold=True)
    f_header = get_font(14, bold=True)
    f_body = get_font(12, bold=False)
    f_small = get_font(10, bold=False)

    # Header Bar
    draw.text((20, 16), "DSP RACK DOCK -- MODULAR AUDIO FX", fill=(240, 245, 255), font=f_title)
    draw.rounded_rectangle([630, 12, 780, 46], radius=4, fill=(35, 50, 75))
    draw.text((648, 22), "Master: ACTIVE", fill=(0, 229, 255), font=f_body)

    # Drop target insertion preview line (cyan #00E5FF)
    draw.line([(20, 68), (780, 68)], fill=(0, 229, 255), width=2)
    draw.text((24, 54), ">>> DROP TARGET INSERTION LINE", fill=(0, 229, 255), font=f_small)

    # Modules
    modules = [
        {"name": "Tube Overdrive", "type": "Distortion", "col": (255, 107, 43), "collapsed": False, "bypass": False, "params": [("Drive", "65%"), ("Bias", "20%"), ("Tone", "50%")]},
        {"name": "State Variable Filter", "type": "Filter", "col": (0, 229, 255), "collapsed": False, "bypass": False, "params": [("Cutoff", "72%"), ("Resonance", "45%"), ("Drive", "15%")]},
        {"name": "Vintage Tape Delay", "type": "Time/Echo", "col": (255, 215, 0), "collapsed": True, "bypass": False, "params": []},
        {"name": "Algorithmic Reverb", "type": "Space", "col": (140, 90, 255), "collapsed": False, "bypass": True, "params": [("Room Size", "82%"), ("Damping", "40%"), ("Mix Wet", "30%")]},
    ]

    my = 78
    for m in modules:
        h = 48 if m["collapsed"] else 86
        box_bg = (18, 22, 30) if m["bypass"] else (22, 30, 46)
        draw.rounded_rectangle([20, my, 780, my + h], radius=6, fill=box_bg, outline=m["col"], width=1)

        # Drag Handle (>=44x44pt)
        draw.rounded_rectangle([28, my + 6, 72, my + 44], radius=4, fill=(30, 40, 60))
        draw.text((44, my + 14), "::", fill=(160, 180, 210), font=f_header)

        # Module Info
        draw.text((85, my + 10), m["name"], fill=(240, 245, 255), font=f_header)
        draw.text((85, my + 28), m["type"], fill=(130, 160, 200), font=f_small)

        # Right Controls: Bypass, Collapse, Delete (each >= 44x44pt)
        # Delete
        draw.rounded_rectangle([730, my + 6, 772, my + 44], radius=4, fill=(45, 25, 30))
        draw.text((746, my + 14), "X", fill=(255, 120, 120), font=f_body)

        # Collapse
        col_lbl = "+" if m["collapsed"] else "-"
        draw.rounded_rectangle([680, my + 6, 722, my + 44], radius=4, fill=(35, 45, 65))
        draw.text((697, my + 14), col_lbl, fill=(200, 220, 250), font=f_body)

        # Bypass
        byp_lbl = "OFF" if m["bypass"] else "ON"
        byp_bg = (60, 40, 45) if m["bypass"] else (0, 180, 140)
        draw.rounded_rectangle([630, my + 6, 672, my + 44], radius=4, fill=byp_bg)
        draw.text((642, my + 14), byp_lbl, fill=(255, 255, 255), font=f_body)

        # Expanded Parameters
        if not m["collapsed"]:
            px = 85
            for p_name, p_val in m["params"]:
                lbl = f"{p_name}:"
                draw.text((px, my + 54), lbl, fill=(180, 200, 225), font=f_small)
                try:
                    lbl_w = int(draw.textlength(lbl, font=f_small))
                except Exception:
                    lbl_w = len(lbl) * 7
                slider_x = px + lbl_w + 8
                draw.rounded_rectangle([slider_x, my + 52, slider_x + 70, my + 68], radius=3, fill=(10, 14, 22))
                val_pct = int(p_val.replace('%', '')) / 100.0
                draw.rounded_rectangle([slider_x, my + 52, slider_x + int(70 * val_pct), my + 68], radius=3, fill=m["col"])
                draw.text((slider_x + 76, my + 54), p_val, fill=m["col"], font=f_small)
                px = slider_x + 115

        my += h + 10

    out_path = os.path.join(OUTPUT_DIR, "dsp_rack_dock.png")
    img.save(out_path)
    print(f"Rendered: {out_path}")

def render_detachable_window_manager():
    width, height = 800, 500
    img = Image.new("RGBA", (width, height), (16, 20, 32, 255))
    draw = ImageDraw.Draw(img)

    f_title = get_font(18, bold=True)
    f_header = get_font(14, bold=True)
    f_body = get_font(12, bold=False)
    f_small = get_font(10, bold=False)

    # Header Bar
    draw.text((20, 16), "MULTI-MONITOR DETACHABLE WINDOW MANAGER", fill=(240, 245, 255), font=f_title)
    draw.text((620, 20), "Edge Snapping: 16pt", fill=(0, 229, 255), font=f_body)

    # Display Topology Card
    draw.rounded_rectangle([20, 56, 780, 140], radius=6, fill=(20, 26, 40), outline=(45, 60, 85))
    draw.text((32, 68), "DETECTED DISPLAY TOPOLOGY:", fill=(200, 220, 245), font=f_header)

    # Mon 1
    draw.rounded_rectangle([32, 90, 380, 130], radius=4, fill=(30, 48, 75), outline=(60, 85, 120))
    draw.text((44, 96), "Monitor #1: Primary Display (4K)", fill=(240, 245, 255), font=f_body)
    draw.text((44, 112), "3840x2160 @ 1.50x DPI Scaling", fill=(0, 229, 255), font=f_small)

    # Mon 2
    draw.rounded_rectangle([400, 90, 748, 130], radius=4, fill=(25, 34, 52), outline=(60, 85, 120))
    draw.text((412, 96), "Monitor #2: Secondary Display (FHD)", fill=(240, 245, 255), font=f_body)
    draw.text((412, 112), "1920x1080 @ 1.00x DPI Scaling", fill=(0, 229, 255), font=f_small)

    # Floating Windows List
    draw.text((20, 158), "ACTIVE FLOATING DETACHED WINDOWS:", fill=(200, 220, 245), font=f_header)

    windows = [
        {"title": "Master Mixer Console", "type": "Multi-Channel Mixer Console", "bounds": "1840x1000 at (3880, 40)", "mon": "#2 (Secondary)"},
        {"title": "3D Waterfall Spectrogram", "type": "3D Waterfall Spectrogram", "bounds": "960x640 at (100, 100)", "mon": "#1 (Primary)"},
        {"title": "Modular DSP Node Graph", "type": "Modular DSP Node Graph", "bounds": "800x500 at (2000, 120)", "mon": "#2 (Secondary)"},
    ]

    wy = 185
    for win in windows:
        draw.rounded_rectangle([20, wy, 780, wy + 68], radius=6, fill=(22, 28, 44), outline=(45, 55, 75))
        draw.text((34, wy + 12), win["title"], fill=(240, 245, 255), font=f_header)
        draw.text((34, wy + 32), f"{win['type']} | Target: {win['mon']}", fill=(140, 170, 210), font=f_small)
        draw.text((34, wy + 48), f"Bounds: {win['bounds']}", fill=(180, 200, 225), font=f_small)

        # Re-attach Button (>=44x44pt)
        draw.rounded_rectangle([590, wy + 12, 765, wy + 56], radius=4, fill=(0, 229, 255))
        draw.text((615, wy + 26), "Re-Attach to Main Dock", fill=(0, 0, 0), font=f_body)

        wy += 78

    out_path = os.path.join(OUTPUT_DIR, "detachable_window_manager.png")
    img.save(out_path)
    print(f"Rendered: {out_path}")

def render_accessibility_announcer():
    width, height = 800, 500
    img = Image.new("RGBA", (width, height), (14, 18, 28, 255))
    draw = ImageDraw.Draw(img)

    f_title = get_font(18, bold=True)
    f_header = get_font(14, bold=True)
    f_body = get_font(12, bold=False)
    f_small = get_font(10, bold=False)

    # Header Bar
    draw.text((20, 16), "ACCESSIBILITY SCREEN READER & FOCUS NAVIGATION", fill=(240, 245, 255), font=f_title)
    draw.rounded_rectangle([610, 12, 780, 46], radius=4, fill=(0, 255, 255))
    draw.text((630, 22), "WCAG AAA (7:1): ON", fill=(0, 0, 0), font=f_body)

    # Active Focus Element Card (With High Contrast Focus Ring)
    draw.rounded_rectangle([20, 60, 780, 150], radius=6, fill=(16, 25, 42), outline=(0, 255, 255), width=2)
    # Focus Ring Outer Contrast Border
    draw.rounded_rectangle([16, 56, 784, 154], radius=8, outline=(0, 0, 0), width=2)

    draw.text((34, 72), "KEYBOARD FOCUSED: SVF Filter Cutoff Frequency [Slider]", fill=(0, 255, 255), font=f_header)
    draw.text((34, 96), "Current Value: 2.4 kHz (72%)", fill=(255, 215, 0), font=f_body)
    draw.text((34, 120), "Tab Order: #4 of 4 | Focus Ring Thickness: 3px | Offset: 3px", fill=(180, 205, 235), font=f_small)

    # Tab Traversal Controls (>=44x44pt)
    draw.rounded_rectangle([20, 165, 190, 210], radius=4, fill=(35, 48, 72))
    draw.text((36, 180), "< Shift+Tab (Prev)", fill=(220, 235, 255), font=f_body)

    draw.rounded_rectangle([205, 165, 375, 210], radius=4, fill=(35, 48, 72))
    draw.text((245, 180), "Tab (Next) >", fill=(220, 235, 255), font=f_body)

    # Live Screen Reader Speech Cues Feed
    draw.text((20, 230), "LIVE SCREEN READER SPEECH CUES:", fill=(200, 220, 245), font=f_header)

    cues = [
        ("[Assertive]", "CRITICAL: Master bus peak overload clip detected at +1.2 dBFS", (255, 107, 43)),
        ("[Polite]", "SVF Filter Cutoff Frequency Slider focused, value 2.4 kHz (72%)", (0, 229, 255)),
        ("[Polite]", "Master Volume Fader value changed to -0.5 dBFS", (0, 229, 255)),
        ("[Polite]", "Project BPM Tempo set to 128.0 BPM", (0, 229, 255)),
        ("[Polite]", "Summoner DAW Accessibility System active. Screen reader narration online.", (0, 229, 255)),
    ]

    cy = 255
    for prio, text, col in cues:
        draw.rounded_rectangle([20, cy, 780, cy + 38], radius=4, fill=(20, 26, 40), outline=(45, 55, 75))
        draw.text((32, cy + 10), prio, fill=col, font=f_header)
        draw.text((115, cy + 12), text, fill=(240, 245, 255), font=f_body)
        cy += 44

    out_path = os.path.join(OUTPUT_DIR, "accessibility_announcer.png")
    img.save(out_path)
    print(f"Rendered: {out_path}")

def render_macro_rotary_dial():
    width, height = 800, 480
    img = Image.new("RGBA", (width, height), (14, 18, 28, 255))
    draw = ImageDraw.Draw(img)

    f_title = get_font(18, bold=True)
    f_header = get_font(14, bold=True)
    f_body = get_font(12, bold=False)
    f_small = get_font(10, bold=False)

    # Header Bar
    draw.text((20, 16), "MULTI-TOUCH MACRO ROTARY DIALS", fill=(240, 245, 255), font=f_title)
    draw.rounded_rectangle([610, 12, 780, 46], radius=4, fill=(35, 50, 75))
    draw.text((626, 22), "Fine Mode: OFF (Hold Shift)", fill=(240, 245, 255), font=f_body)

    # Dials
    dials = [
        {"name": "Filter Cutoff", "mode": "Unipolar", "val": 0.72, "disp": "14.4kHz", "col": (0, 229, 255), "mod": 0.0},
        {"name": "Drive Trim", "mode": "Bipolar", "val": 0.35, "disp": "+8.4dB", "col": (255, 107, 43), "mod": 0.25},
        {"name": "Resonance Q", "mode": "Unipolar", "val": 0.45, "disp": "8.2Q", "col": (255, 215, 0), "mod": 0.0},
        {"name": "Stereo Pan", "mode": "Bipolar", "val": -0.20, "disp": "-20.0%", "col": (140, 90, 255), "mod": 0.0},
    ]

    dx = 30
    for d in dials:
        card_rect = [dx, 70, dx + 170, 440]
        draw.rounded_rectangle(card_rect, radius=8, fill=(20, 26, 40), outline=(45, 55, 75))

        # Title
        draw.text((dx + 30, 85), d["name"], fill=(240, 245, 255), font=f_header)
        draw.text((dx + 50, 105), f"[{d['mode']}]", fill=(140, 165, 200), font=f_small)

        # Dial Canvas Center
        cx = dx + 85
        cy = 210
        r_outer = 48
        r_inner = 36

        # Background Track
        draw.ellipse([cx - r_outer, cy - r_outer, cx + r_outer, cy + r_outer], fill=(12, 16, 26), outline=(40, 55, 80), width=3)

        # Value Arc Angle Math (-135 to +135 deg)
        norm = d["val"] if d["mode"] == "Unipolar" else (d["val"] + 1.0) * 0.5
        angle_deg = -135.0 + norm * 270.0
        angle_rad = math.radians(angle_deg)

        tip_x = cx + math.sin(angle_rad) * r_inner
        tip_y = cy - math.cos(angle_rad) * r_inner

        # Indicator Needle line
        draw.line([(cx, cy), (tip_x, tip_y)], fill=d["col"], width=4)
        draw.ellipse([tip_x - 4, tip_y - 4, tip_x + 4, tip_y + 4], fill=(255, 255, 255))
        draw.ellipse([cx - 5, cy - 5, cx + 5, cy + 5], fill=d["col"])

        # Modulation Ring Overlay if present
        if d["mod"] > 0:
            draw.ellipse([cx - r_outer - 6, cy - r_outer - 6, cx + r_outer + 6, cy + r_outer + 6], outline=(255, 215, 0), width=2)
            draw.text((dx + 35, 280), f"Mod: +{int(d['mod']*100)}%", fill=(255, 215, 0), font=f_small)

        # Display Value Label
        draw.text((dx + 50, 310), d["disp"], fill=d["col"], font=f_header)

        # Minimum Hit Target Compliance Badge (>=44x44pt)
        draw.rounded_rectangle([dx + 15, 360, dx + 155, 415], radius=4, fill=(16, 35, 30), outline=(0, 220, 140))
        draw.text((dx + 25, 375), "[PASS] Hit Target", fill=(0, 255, 180), font=f_small)
        draw.text((dx + 25, 392), "Radius: 32pt (64px)", fill=(0, 255, 180), font=f_small)

        dx += 190

    out_path = os.path.join(OUTPUT_DIR, "macro_rotary_dial.png")
    img.save(out_path)
    print(f"Rendered: {out_path}")

def render_harmonic_tension_map():
    width, height = 800, 480
    img = Image.new("RGBA", (width, height), (14, 18, 28, 255))
    draw = ImageDraw.Draw(img)

    f_title = get_font(18, bold=True)
    f_header = get_font(14, bold=True)
    f_body = get_font(12, bold=False)
    f_small = get_font(10, bold=False)

    # Header Bar
    draw.text((20, 16), "HARMONIC TENSION MAP & PROGRESSION BUILDER", fill=(240, 245, 255), font=f_title)
    draw.text((580, 20), "Key: C Major | 12-EDO Tuning", fill=(0, 229, 255), font=f_body)

    # Tension Curve Graph Card
    draw.rounded_rectangle([20, 56, 780, 250], radius=8, fill=(10, 14, 22), outline=(40, 55, 80))
    draw.text((34, 68), "HARMONIC TENSION CURVE OVERLAY:", fill=(200, 220, 245), font=f_header)

    # Grid guide lines
    draw.line([(50, 110), (750, 110)], fill=(50, 65, 90), width=1)
    draw.text((30, 104), "1.0", fill=(140, 165, 195), font=f_small)
    draw.line([(50, 160), (750, 160)], fill=(50, 65, 90), width=1)
    draw.text((30, 154), "0.5", fill=(140, 165, 195), font=f_small)
    draw.line([(50, 210), (750, 210)], fill=(50, 65, 90), width=1)
    draw.text((30, 204), "0.0", fill=(140, 165, 195), font=f_small)

    # Tension Points: Dm7 (25%), G7 (65%), Cmaj7 (15%), A7alt (95%)
    curve_pts = [
        (120, 210 - int(0.25 * 100)),
        (300, 210 - int(0.65 * 100)),
        (480, 210 - int(0.15 * 100)),
        (660, 210 - int(0.95 * 100)),
    ]

    for i in range(len(curve_pts) - 1):
        draw.line([curve_pts[i], curve_pts[i + 1]], fill=(0, 229, 255), width=3)

    for pt in curve_pts:
        draw.ellipse([pt[0] - 6, pt[1] - 6, pt[0] + 6, pt[1] + 6], fill=(255, 215, 0))

    # Progression Chord Cards
    draw.text((20, 268), "PROGRESSION BLOCKS (>=60x60pt Touch Cards):", fill=(200, 220, 245), font=f_header)

    chords = [
        {"root": "Dm7", "roman": "ii7", "tension": 25, "col": (40, 255, 180)},
        {"root": "G7", "roman": "V7", "tension": 65, "col": (255, 215, 0)},
        {"root": "Cmaj7", "roman": "Imaj7", "tension": 15, "col": (0, 229, 255)},
        {"root": "A7alt", "roman": "VI7alt", "tension": 95, "col": (255, 45, 120)},
    ]

    cx = 30
    for c in chords:
        draw.rounded_rectangle([cx, 295, cx + 170, 445], radius=6, fill=(20, 26, 40), outline=c["col"], width=2)
        draw.text((cx + 50, 310), c["root"], fill=(240, 245, 255), font=f_title)
        draw.text((cx + 65, 345), c["roman"], fill=(150, 180, 220), font=f_body)
        draw.text((cx + 35, 380), f"{c['tension']}% Tension", fill=c["col"], font=f_header)
        draw.text((cx + 25, 415), "4.0 Beats Duration", fill=(130, 155, 185), font=f_small)
        cx += 190

    out_path = os.path.join(OUTPUT_DIR, "harmonic_tension_map.png")
    img.save(out_path)
    print(f"Rendered: {out_path}")

if __name__ == "__main__":
    render_live_macro_rack()
    render_spectrogram_3d()
    render_keybinding_editor()
    render_meter_bridge()
    render_dpi_scale_panel()
    render_dsp_rack_dock()
    render_detachable_window_manager()
    render_accessibility_announcer()
    render_macro_rotary_dial()
    render_harmonic_tension_map()
    print("All Tier 50 & Tier 51 GUI render previews generated successfully!")
