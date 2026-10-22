"""Retain terminal current outputs without regenerating any original result."""
from pathlib import Path
import json,subprocess,sys
OUT=Path(__file__).resolve().parent
jobs=[('rust-original','complete','rust-overlays','/private/tmp/mpk-go-source-overlays-complete-rust-overlays-outputs'),('selection-original','affected','rust-selection','/private/tmp/mpk-go-source-overlays-affected-rust-selection-outputs'),('dispatch-original','affected','rust-dispatch','/private/tmp/mpk-go-source-overlays-affected-rust-dispatch-outputs'),('source-closure-original','native-closure','go-closure','/private/tmp/mpk-go-source-overlays-native-closure-native'),('frontend-original','frontend','frontend','/private/tmp/mpk-go-source-overlays-frontend-native')]
for name,epoch,mode,source in jobs:
 r=json.loads((OUT/epoch/mode/'result.json').read_bytes());assert r['status']=='passed'
 subprocess.run([sys.executable,str(OUT/'archive-originals.py'),name,source],check=True)
