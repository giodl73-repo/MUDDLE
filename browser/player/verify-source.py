"""Replay the delivered software outside Git without repository policy/production archives."""
import argparse,hashlib,json,os,pathlib,re,subprocess,tempfile,zipfile
p=argparse.ArgumentParser();p.add_argument('--site',required=True,type=pathlib.Path);p.add_argument('--bindgen',required=True,type=pathlib.Path);a=p.parse_args();B=pathlib.Path(__file__).resolve().parent;C=B/'.cache';S=a.site.resolve();native=json.loads((C/'native-cases.json').read_bytes());wasm=json.loads((C/'wasm-cases.json').read_bytes());assert len(native)==len(wasm)==103
env=os.environ.copy()
for name in ['MUDDLE_BASE_URL','MUDDLE_EXPECTED_COMMIT','MUDDLE_SITE_DIR']:env.pop(name,None)
env['MUDDLE_SITE_DIR']='../../target/player-prototype'
with(C/'source-replay.log').open('wb')as output,tempfile.TemporaryDirectory(prefix='muddle source replay ')as temp:
 T=pathlib.Path(temp).resolve()
 with zipfile.ZipFile(S/'source.zip')as archive:
  for name in archive.namelist():
   path=pathlib.PurePosixPath(name);assert path.parts[0]=='MUDDLE-software-source'and '..'not in path.parts and not path.is_absolute();target=T.joinpath(*path.parts);assert target.resolve().is_relative_to(T);target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(archive.read(name))
 W=T/'MUDDLE-software-source';P=W/'browser/player';assert ' 'in str(W)and not(W/'.roles').exists()and not(W/'.pitfall').exists()and subprocess.run(['git','rev-parse','--show-toplevel'],cwd=W,capture_output=True).returncode!=0
 def run(args,cwd=W):
  result=subprocess.run([str(x)for x in args],cwd=cwd,env=env,capture_output=True)
  output.write(b'\n[stdout]\n'+result.stdout+b'\n[stderr]\n'+result.stderr);output.flush()
  assert result.returncode==0,(result.stdout+result.stderr).decode(errors='replace')[-3000:]
  return result.stdout
 for manifest in [W/'Cargo.toml',P/'Cargo.toml']:
  m=json.loads(run(['cargo','+1.95.0','metadata','--locked','--format-version','1','--manifest-path',manifest]));assert all(pathlib.Path(pkg['manifest_path']).resolve().is_relative_to(W)for pkg in m['packages']if not pkg.get('source'))
 tests=run(['cargo','+1.95.0','test','--locked','--workspace']);assert sum(int(n)for n in re.findall(rb'test result: ok\. (\d+) passed;',tests))==91
 tests=run(['cargo','+1.95.0','test','--locked','--manifest-path',P/'Cargo.toml']);assert sum(int(n)for n in re.findall(rb'test result: ok\. (\d+) passed;',tests))==5
 tests=run(['cargo','+1.95.0','test','--locked','--no-default-features','--manifest-path',P/'Cargo.toml']);assert sum(int(n)for n in re.findall(rb'test result: ok\. (\d+) passed;',tests))==5
 run(['python',P/'verify-embedding.py'])
 run(['python',P/'build-prototype.py','--bindgen',a.bindgen.resolve()]);assert json.loads((P/'.cache/native-cases.json').read_bytes())==native
 controls=json.loads((P/'.cache/native-controls.json').read_bytes());assert controls['independent_native_cli_save_transcript_pairs']==18
 npm='npm.cmd'if os.name=='nt'else'npm';run([npm,'ci'],cwd=P);run([npm,'run','test:pages','--','--grep-invert','packaged delivery|source/legal'],cwd=P)
 stats=json.loads((P/'.cache/browser-results.json').read_bytes())['stats'];assert stats['expected']==17 and all(stats[k]==0 for k in ['skipped','unexpected','flaky']);assert json.loads((P/'.cache/wasm-cases.json').read_bytes())==wasm
 record={'outside_git':True,'path_contains_spaces':True,'policy_production_archives_absent':True,'first_party_manifest_paths_inside_source':True,'native_tests':91,'adapter_tests':5,'no_default_adapter_tests':5,'embedding_demo_entry_export_absent':True,'independent_native_cli_save_transcript_pairs':18,'browser_checks':17,'identical_rebuilt_native_responses':103,'identical_rebuilt_wasm_responses':103,'original_wasm_sha256':hashlib.sha256((S/'pkg/muddle_browser_player_bg.wasm').read_bytes()).hexdigest(),'rebuilt_wasm_sha256':hashlib.sha256((W/'target/player-prototype/MUDDLE/pkg/muddle_browser_player_bg.wasm').read_bytes()).hexdigest(),'binary_identity_claimed':False,'dependencies_vendored':False}
(C/'source-replay.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8');print(json.dumps(record))
