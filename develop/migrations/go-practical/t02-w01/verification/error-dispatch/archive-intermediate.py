"""Retain terminal original outputs; never rebuild an earlier producer."""
from pathlib import Path
import json,subprocess,sys
OUT=Path(__file__).resolve().parent
jobs=[('native-initial-original','native','go-dispatch','/private/tmp/mpk-go-error-dispatch-native-native'),('native-original','native-corrected','go-dispatch','/private/tmp/mpk-go-error-dispatch-native-corrected-native'),('source-closure-original','native-closure','go-closure','/private/tmp/mpk-go-error-dispatch-native-closure-native'),('frontend-original','frontend','frontend','/private/tmp/mpk-go-error-dispatch-frontend-native'),('first-rust-original','first','rust-dispatch','/private/tmp/mpk-go-error-dispatch-first-rust-dispatch-outputs'),('method-original','affected','rust-methods','/private/tmp/mpk-go-error-dispatch-affected-rust-methods-outputs'),('body-original','affected','rust-bodies','/private/tmp/mpk-go-error-dispatch-affected-rust-bodies-outputs')]
for name,epoch,mode,source in jobs:
 r=json.loads((OUT/epoch/mode/'result.json').read_bytes());assert r['status']in {'passed','failed'}
 subprocess.run([sys.executable,str(OUT/'archive-originals.py'),name,source],check=True)
