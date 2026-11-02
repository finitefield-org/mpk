"""Preserve terminal original producer outputs; do not rerun consumers."""
from pathlib import Path
import json,subprocess,sys
OUT=Path(__file__).resolve().parent
for name,epoch,mode,source in [('native-initial-original','native','go-overlays','/private/tmp/mpk-go-source-overlays-native-native'),('native-root-original','native-corrected','go-overlays','/private/tmp/mpk-go-source-overlays-native-corrected-native'),('native-original','prepared-native','go-overlays','/private/tmp/mpk-go-source-overlays-prepared-native-native'),('first-rust-original','first','rust-overlays','/private/tmp/mpk-go-source-overlays-first-rust-overlays-outputs')]:
 r=json.loads((OUT/epoch/mode/'result.json').read_bytes());assert r['status']in {'passed','failed'}
 subprocess.run([sys.executable,str(OUT/'archive-originals.py'),name,source],check=True)
