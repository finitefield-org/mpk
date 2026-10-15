import hashlib,json,pathlib,re,subprocess
b=pathlib.Path(__file__).parent;g=pathlib.Path('/private/tmp/mpk-w09-refreeze-metadata-integration');commit='40469d298f1d69558713d02ea0b553a1dd583bf5'
def body(s,header):
 start=s.index(header)+len(header);depth=1
 for token in re.finditer(r'"(?:\\.|[^"\\])*"|//[^\n]*|[{}]',s[start:]):
  if token.group()=='{':depth+=1
  elif token.group()=='}':
   depth-=1
   if depth==0:return s[start:start+token.start()]
 raise AssertionError('unterminated function')
rows=[]
for file,family in [('integer_format','integer_formats'),('hex_codec','hex_codecs'),('decimal_format','decimal_formats'),('decimal_fixed_format','decimal_fixed_formats')]:
 path=f'crates/mpk-vc/tests/support/csharp_practical_ordinary_{file}_tests.rs';old=subprocess.check_output(['/usr/bin/git','-C',str(g),'show',commit+':'+path],text=True);new=(g/path).read_text();a=body(old,f'fn csharp_03_t06_w09_{family}_original_sources() {{');c=body(new,f'fn {family}_original_sources(observe_values: bool) {{');guard='    if !observe_values {\n        return;\n    }\n';assert c.count(guard)==1;c=c.replace(guard,'',1);assert a==c,(path,'owner body changed');assert f'fn csharp_03_t06_w09_{family}_original_sources() {{\n    {family}_original_sources(true);\n}}' in new;assert f'fn csharp_03_t06_w09_{family}_original_source_certificates() {{\n    {family}_original_sources(false);\n}}' in new;rows.append(dict(path=path,original_owner_body_raw_sha256=hashlib.sha256(a.encode()).hexdigest(),complete_source_generation_import_mutations_and_value_observations_preserved=True))
receipt={'schema':'mpk.csharp_practical.t01_w09.observation_owner_split_audit.v1','status':'passed_exact_original_owner_body_preservation','public_original_source_commit':commit,'owner_functions':rows,'selection_reason':'Only context-bound source/canonical-metadata linkage is affected. Separate certificate replay from exhaustive value observations without removing or changing any original semantic case; the original owners call the unchanged complete body with observations enabled.'};(b/'observation-owner-split-audit.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n');print(json.dumps({'status':receipt['status'],'owner_functions':len(rows)}))
