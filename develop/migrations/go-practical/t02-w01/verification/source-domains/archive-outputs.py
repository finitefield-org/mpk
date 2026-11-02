"""Keep terminal original observer outputs without rerunning their consumers."""
from pathlib import Path
import json,subprocess,sys
OUT=Path(__file__).resolve().parent
for epoch,mode in [('first','rust-domains'),('complete','rust-domains'),('corrected','rust-domains'),('affected-selection','rust-selection'),('affected-boxings','rust-boxings')]:
 p=OUT/epoch/mode/'result.json';r=json.loads(p.read_bytes());assert r['status']in {'passed','failed'}
 name=epoch+'-'+mode+'-original';source=Path('/private/tmp')/('mpk-go-source-domains-'+epoch+'-'+mode+'-outputs')
 assert source.is_dir()
 subprocess.run([sys.executable,str(OUT/'archive-originals.py'),name,str(source)],check=True)
