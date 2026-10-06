import hashlib
import json
import os
import pathlib
import re
import subprocess
import time

base = pathlib.Path(__file__).parent
repo = pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration')
out = base/'ordinary-regeneration-4d'
out.mkdir(exist_ok=False)
goldens = out/'goldens'
goldens.mkdir()
manifest = json.loads((base/'ordinary-source-manifest-4d.json').read_bytes())
digest = lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
state = {'status':'running','stages':[],'selection_reason':manifest['selection_reason'],
    'full_t01_gate':'deferred to renewed T01-W10','full_t06_gate':'deferred to T06-W12'}
def save():
    (out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
def verify():
    for path, sha in manifest['hashes'].items():
        assert digest(repo/path) == sha, path
jobs = [('integer-formats', 'csharp_03_t06_w09_integer_formats_original_source_certificates', 'MPK_W09_INTEGER_FORMATS_OUT', 'integer-formats', 'mpk-vc', 'csharp_practical_vc'), ('hex-codecs', 'csharp_03_t06_w09_hex_codecs_original_source_certificates', 'MPK_W09_HEX_CODECS_OUT', 'hex-codecs', 'mpk-vc', 'csharp_practical_vc'), ('calendar-codecs', 'csharp_03_t06_w09_calendar_codecs_original_sources', 'MPK_W09_CALENDAR_CODECS_OUT', 'calendar-codecs', 'mpk-vc', 'csharp_practical_vc'), ('decimal-parsers', 'csharp_03_t06_w09_decimal_parsers_original_sources', 'MPK_W09_DECIMAL_PARSERS_OUT', 'decimal-parsers', 'mpk-vc', 'csharp_practical_vc'), ('decimal-formats', 'csharp_03_t06_w09_decimal_formats_original_source_certificates', 'MPK_W09_DECIMAL_FORMATS_OUT', 'decimal-formats', 'mpk-vc', 'csharp_practical_vc'), ('decimal-fixed-formats', 'csharp_03_t06_w09_decimal_fixed_formats_original_source_certificates', 'MPK_W09_DECIMAL_FIXED_FORMATS_OUT', 'decimal-fixed-formats', 'mpk-vc', 'csharp_practical_vc'), ('structural-storage', 'csharp_03_t06_w09_structural_source_linkage_rejects_substitution', 'MPK_W09_STRUCTURAL_OUT', 'structural-storage', 'mpk-vc', 'csharp_practical_vc'), ('scalar-domains', 'csharp_03_t06_w09_scalar_domains_original_sources_and_mutations', 'MPK_W09_SCALAR_DOMAINS_OUT', 'scalar-domains', 'mpk-vc', 'csharp_practical_vc'), ('scalar-domain-ranges', 'csharp_03_t06_w09_scalar_domains_ranges_from_original_sources', 'MPK_W09_SCALAR_DOMAIN_RANGES_OUT', 'scalar-domain-ranges', 'mpk-vc', 'csharp_practical_vc')]
save()
try:
    verify()
    for label, selector, variable, destination, package, target in jobs:
        env = os.environ.copy()
        for key in list(env):
            if key.startswith(('MPK_T06_', 'MPK_W09_')) or key == 'MPK_W14_REQUEST_OUTPUT':
                env.pop(key)
        env['CARGO_TARGET_DIR'] = '/private/tmp/mpk-w09-refreeze-metadata-target'
        generated = goldens/destination if destination else None
        if generated:
            generated.parent.mkdir(parents=True,exist_ok=True)
            env[variable] = str(generated)
        command = ['/Users/kazuyoshitoshiya/.cargo/bin/cargo','test','-p',package,
            '--test',target,selector,'--','--nocapture','--test-threads=1']
        state['stage'] = label
        started = time.monotonic()
        with (out/(label+'.log.txt')).open('wb') as log:
            process = subprocess.Popen(command,cwd=repo,env=env,stdout=log,stderr=subprocess.STDOUT)
            state['process_pid'] = process.pid
            save()
            code = process.wait()
        text = (out/(label+'.log.txt')).read_text()
        counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;',text)
        row = {'stage':label,'selector':selector,'destination':destination,'exit_code':code,
            'test_counts':counts,'log_sha256':digest(out/(label+'.log.txt')),
            'elapsed_seconds':time.monotonic()-started}
        state['stages'].append(row)
        save()
        print(label+': '+str(code),flush=True)
        assert code == 0 and counts == ['1'] and (generated is None or generated.is_dir()), label
        if generated:
            row['generated_files'] = {str(p.relative_to(generated)):digest(p) for p in generated.rglob('*') if p.is_file()}
        verify()
    state['status'] = 'passed_selected_source_bound_ordinary_owners'
except BaseException as error:
    state['status'] = 'failed'
    state['error'] = repr(error)
    raise
finally:
    save()
