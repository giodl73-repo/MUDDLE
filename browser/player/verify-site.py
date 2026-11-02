"""Verify exact artifact/source scope and original grants; no product adoption claims."""
import argparse,hashlib,json,pathlib,subprocess,zipfile
p=argparse.ArgumentParser();p.add_argument('site',type=pathlib.Path);p.add_argument('--commit',required=True);a=p.parse_args();S=a.site.resolve();R=pathlib.Path(__file__).resolve().parents[2]
def read(path):return json.loads(path.read_bytes())
def sha(data):return hashlib.sha256(data).hexdigest()
def safe(path):
 value=pathlib.PurePosixPath(path);assert not value.is_absolute()and '..'not in value.parts and not any(x.startswith('.')for x in value.parts),path
 return path
proof=read(S/'provenance.json');inventory=read(S/'inventory.json');assert proof['commit']==inventory['commit']==a.commit
assert proof['runtime_files']==['index.html','app.js','style.css','worker.js','example-world.json','pkg/muddle_browser_player.js','pkg/muddle_browser_player_bg.wasm']
for name in proof['runtime_files'][:5]:assert (S/name).read_bytes()==subprocess.check_output(['git','show',a.commit+':browser/player/web/'+name],cwd=R),name
for name,path in [('LICENSE','LICENSE'),('BUILD.md','browser/player/BUILD.md'),('MODEL.md','browser/player/MODEL.md'),('SOURCE-SCOPE.md','browser/player/SOURCE-SCOPE.md')]:assert (S/name).read_bytes()==subprocess.check_output(['git','show',a.commit+':'+path],cwd=R),name
expected={safe(r['path'])for r in inventory['files']};assert len(expected)==len(inventory['files']);actual=set()
for path in S.rglob('*'):
 assert not path.is_symlink(),path
 if path.is_file():actual.add(path.relative_to(S).as_posix())
assert actual==expected|{'inventory.json'},actual^expected^{'inventory.json'}
for record in inventory['files']:
 data=(S/record['path']).read_bytes();assert len(data)==record['bytes']and sha(data)==record['sha256'],record['path']
assert inventory['bytes_excluding_inventory']==sum(r['bytes']for r in inventory['files'])
assert inventory['runtime_bytes']==sum((S/path).stat().st_size for path in proof['runtime_files'])<=2097152
assert(S/'source.zip').stat().st_size<=10485760 and sum(p.stat().st_size for p in S.rglob('*')if p.is_file())<=26214400
assert sha((S/'source.zip').read_bytes())==proof['source_zip_sha256']and sha((S/'pkg/muddle_browser_player_bg.wasm').read_bytes())==proof['wasm_sha256']
assert(S/'pkg/muddle_browser_player_bg.wasm').read_bytes().startswith(b'\0asm')
git_source={}
for item in subprocess.check_output(['git','ls-tree','-rz','--full-tree',a.commit],cwd=R).split(b'\0'):
 if not item:continue
 info,path=item.split(b'\t',1);mode,kind,oid=info.decode().split();path=path.decode()
 if path in ['LICENSE','Cargo.toml','Cargo.lock','browser/player/SOURCE-SCOPE.md']or path.startswith(('crates/','browser/player/','docs/rune/')):
  assert kind=='blob'and mode in ['100644','100755'];git_source[path]=(mode,oid)
source=read(S/'source-files.json');assert len(source)==proof['source_files']and{r['path']for r in source}==set(git_source)
with zipfile.ZipFile(S/'source.zip')as archive:
 assert archive.comment.decode()==a.commit
 assert len(archive.namelist())==len(source)==len(set(archive.namelist()))
 assert set(archive.namelist())=={'MUDDLE-software-source/'+r['path']for r in source}
 for record in source:
  name='MUDDLE-software-source/'+record['path'];data=archive.read(name);mode,oid=git_source[record['path']]
  assert len(data)==record['bytes']and sha(data)==record['sha256']and record['mode']==mode and archive.getinfo(name).external_attr>>16==int(mode,8)
  assert data==subprocess.check_output(['git','cat-file','blob',oid],cwd=R),name
packages=read(S/'dependencies.json');rust=read(S/'rust/provenance.json');assert len(packages)==proof['dependency_groups']and len(rust['notices'])==proof['rust_notice_files']>0
allowed={'inventory.json','provenance.json','source.zip','source-files.json','dependencies.json','LICENSE','BUILD.md','MODEL.md','SOURCE-SCOPE.md','licenses/index.html','rust/provenance.json','rust/COPYRIGHT-library.html',*proof['runtime_files']}
for package in packages:
 assert package['notices']
 for notice in package['notices']:
  path=safe(notice['path']);assert path.startswith('licenses/dependencies/'+package['name']+'-'+package['version']+'/')and pathlib.PurePosixPath(path).name.upper().startswith(('LICENSE','NOTICE','COPYRIGHT','UNLICENSE'))
  assert sha((S/path).read_bytes())==notice['sha256'];allowed.add(path)
for notice in rust['notices']:
 path=safe(notice['path']);assert path.startswith('rust/licenses/')and sha((S/path).read_bytes())==notice['sha256'];allowed.add(path)
assert sha((S/'rust/COPYRIGHT-library.html').read_bytes())==rust['copyright_sha256']and actual==allowed
assert read(S/'example-world.json')['start_room']=='reading-room'
print(json.dumps({'commit':a.commit,'files':len(actual),'runtime_bytes':inventory['runtime_bytes'],'source_files':len(source),'dependency_groups':len(packages),'rust_notices':len(rust['notices']),'authoritative_product_distribution_claimed':False,'verified':True}))
