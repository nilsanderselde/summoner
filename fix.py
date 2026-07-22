import os

def fix_slicer():
    path = r"c:\Users\Nils\Code\Summoner\crates\summoner_dsp\src\slicer.rs"
    with open(path, "r") as f:
        content = f.read()
    content = content.replace("buffer.samples", "buffer.data")
    with open(path, "w") as f:
        f.write(content)

def fix_kbm():
    path = r"c:\Users\Nils\Code\Summoner\crates\summoner_harmony\src\kbm.rs"
    with open(path, "r") as f:
        content = f.read()
    content = content.replace("use std::collections::HashMap;\n", "")
    with open(path, "w") as f:
        f.write(content)

def fix_name(module, struct_name, string_name):
    path = rf"c:\Users\Nils\Code\Summoner\crates\summoner_dsp\src\{module}.rs"
    with open(path, "r") as f:
        content = f.read()
    
    replace_str = f"impl SignalProcessor for {struct_name} {{"
    replacement = f"impl SignalProcessor for {struct_name} {{\n    fn name(&self) -> &str {{ \"{string_name}\" }}"
    
    content = content.replace(replace_str, replacement)
    
    with open(path, "w") as f:
        f.write(content)

def add_timeline():
    path = r"c:\Users\Nils\Code\Summoner\crates\summoner_sequencer\src\timeline.rs"
    if not os.path.exists(path):
        with open(path, "w") as f:
            f.write("// Timeline module\n")

if __name__ == "__main__":
    fix_slicer()
    fix_kbm()
    
    fix_name("biquad", "FilterBiquad", "FilterBiquad")
    fix_name("compressor", "CompressorNode", "CompressorNode")
    fix_name("limiter", "LimiterNode", "LimiterNode")
    fix_name("mod_fx", "EffectChorus", "EffectChorus")
    fix_name("mod_fx", "EffectFlanger", "EffectFlanger")
    fix_name("mod_fx", "EffectPhaser", "EffectPhaser")
    fix_name("ring_mod", "RingModulator", "RingModulator")
    fix_name("ring_mod", "FrequencyShifter", "FrequencyShifter")
    fix_name("meter", "LufsMeterNode", "LufsMeterNode")
    
    add_timeline()

print("Fixes applied")
