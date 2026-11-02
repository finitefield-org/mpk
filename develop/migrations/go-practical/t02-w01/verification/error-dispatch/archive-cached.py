"""Preserve all completed Rust source consumers, including before cache repair."""
from pathlib import Path
import json,subprocess,sys
OUT=Path(__file__).resolve().parent
for epoch,mode,name in [('affected','rust-selection','selection-before-cache'),('affected','rust-domains','domains-before-cache'),('affected','rust-boxings','boxings-before-cache'),('cached','rust-dispatch','rust-original'),('cached','rust-selection','selection-original'),('cached','rust-domains','domains-original'),('cached','rust-boxings','boxings-original')]:
 r=json.loads((OUT/epoch/mode/'result.json').read_bytes());assert r['status']=='passed'
 source=Path('/private/tmp')/('mpk-go-error-dispatch-'+epoch+'-'+mode+'-outputs')
 subprocess.run([sys.executable,str(OUT/'archive-originals.py'),name,str(source)],check=True)
