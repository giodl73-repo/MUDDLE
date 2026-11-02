"""Build and qualify synthetic native controls, then stage a local-only player."""
import argparse,copy,json,os,pathlib,shutil,subprocess,tempfile,hashlib
import importlib.util
_spec=importlib.util.spec_from_file_location("resource_controls",pathlib.Path(__file__).with_name("resource-controls.py"));_module=importlib.util.module_from_spec(_spec);_spec.loader.exec_module(_module)
p=argparse.ArgumentParser();p.add_argument('--bindgen',required=True,type=pathlib.Path);a=p.parse_args();B=pathlib.Path(__file__).resolve().parent;R=B.parents[1];C=B/'.cache';C.mkdir(exist_ok=True)
env=os.environ.copy();target=pathlib.Path(env.get('CARGO_TARGET_DIR',str(R/'target/cargo'))).resolve();env['CARGO_TARGET_DIR']=str(target);env['CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS']='-C link-arg=--max-memory=67108864'
def run(args,**kw):return subprocess.run([str(x)for x in args],cwd=R,env=env,check=True,**kw)
assert subprocess.check_output([str(a.bindgen.resolve()),'--version']).decode().strip()=='wasm-bindgen 0.2.127'
for args in [[],['--target','wasm32-unknown-unknown']]:run(['cargo','+1.95.0','build','--locked','--manifest-path',B/'Cargo.toml','--release',*args])
run(['cargo','+1.95.0','build','--locked','-p','muddle-cli'])
S=R/'target/player-prototype/MUDDLE';assert S.resolve()==R/'target/player-prototype/MUDDLE'
if S.exists():shutil.rmtree(S)
shutil.copytree(B/'web',S);run([a.bindgen.resolve(),target/'wasm32-unknown-unknown/release/muddle_browser_player.wasm','--target','web','--out-dir',S/'pkg'])
for name in ['muddle_browser_player.d.ts','muddle_browser_player_bg.wasm.d.ts']:(S/'pkg'/name).unlink()
for name in ['BUILD.md','MODEL.md']:shutil.copyfile(B/name,S/name)
suffix='.exe'if os.name=='nt'else'';binary=target/('release/muddle-browser-control'+suffix);cli=target/('debug/muddle-cli'+suffix)
def text(v):return json.dumps(v,ensure_ascii=False,separators=(',',':'))
def evaluate(q):
 raw=text(q)if not isinstance(q,str)else q
 result=run([binary],input=(raw+'\n').encode(),capture_output=True).stdout.decode().splitlines();assert len(result)==1
 return raw,json.loads(result[0])
paths={'mock-labyrinth':['gather ember','go antechamber','inspect glyphs','use ember','go vault'],'banish-pilgrim-loss':['choose resume','inspect plan','inspect manifest','go trail','resolve loss'],'amaze-silverstream':['go receiver','inspect clue','tune signal','unlock hatch','go hatch']}
requests=[];cli_checks=[]
for host,commands in paths.items():
 for n in range(len(commands)+1):
  q={'host':host,'commands':commands[:n]};requests.append(q);_,r=evaluate(q);assert r['kind']=='report'and all(x['kind']=='turn'for x in r['attempts'])
  for field in ['save_text','replay_save_text']:requests.append({'host':host,'save_text':r[field],'commands':['look']})
  with tempfile.TemporaryDirectory(prefix='muddle native cli ')as tmp:
   T=pathlib.Path(tmp);(T/'commands.txt').write_bytes((('\n'.join(commands[:n])+'\n')if n else '').encode());run([cli,'--host',host,'--script',T/'commands.txt','--save',T/'save.muddle','--transcript',T/'transcript.txt'],stdout=subprocess.DEVNULL)
   assert (T/'save.muddle').read_bytes()==r['save_text'].encode();assert (T/'transcript.txt').read_bytes()==r['transcript_text'].encode()
   cli_checks.append({'host':host,'commands':n,'save_sha256':hashlib.sha256((T/'save.muddle').read_bytes()).hexdigest(),'transcript_sha256':hashlib.sha256((T/'transcript.txt').read_bytes()).hexdigest()})
 requests.extend([{'host':host,'commands':['not-a-verb','look','  LOOK  ']},{'host':host,'save_text':'broken'},{'host':host,'save_text':'MUDDLE_SESSION_V1\ncurrent_room=missing'},{'host':host,'commands':['look']*128},{'host':host,'commands':['look']*129}])
world=(B/'web/example-world.json').read_text(encoding='utf-8');requests.extend([{'host':'authored-rooms','world_json':world,'commands':['look','go garden','look','go inside']},{'host':'authored-rooms','world_json':world,'commands':['javascript:alert(1)','look']},{'host':'authored-rooms','world_json':'{'}])
for raw in ['{','null','{}','{"host":"mock-labyrinth","host":"amaze-silverstream"}','{"host":"mock-labyrinth","extra":true}',text({'host':'mock-labyrinth'})+' {}']:requests.append(raw)
requests.extend([{'host':'unknown'},{'host':'authored-rooms'},{'host':'mock-labyrinth','world_json':world},{'host':'mock-labyrinth','commands':['α'*513]},{'host':'mock-labyrinth','save_text':'x'*131073}])
resource=_module.controls()
for case in resource:
 assert evaluate(case["request"])[1]["kind"]==case["expected_kind"],case["name"]
requests.extend(case["request"]for case in resource)
(C/"resource-controls.json").write_text(json.dumps(resource,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
records=[]
for q in requests:
 raw,response=evaluate(q);assert evaluate(q)[1]==response;assert response['request_sha256']==hashlib.sha256(raw.encode()).hexdigest();records.append({'request':raw,'response':response})
(C/'native-cases.json').write_text(json.dumps(records,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');(C/'native-controls.json').write_text(json.dumps({'cases':len(records),'each_repeated':2,'independent_native_cli_save_transcript_pairs':len(cli_checks),'cli_checks':cli_checks},indent=2)+'\n',encoding='utf-8');print(json.dumps({'native_cases':len(records),'direct_cli_save_transcript_pairs':len(cli_checks),'staged':str(S)}))
