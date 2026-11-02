"""Stage exact corresponding software and complete notices from clean Git."""
import hashlib,html,json,pathlib,shutil,subprocess,zipfile
B=pathlib.Path(__file__).resolve().parent;R=B.parents[1];S=R/'target/player-release/MUDDLE';P=R/'target/player-prototype/MUDDLE'
def run(*args):return subprocess.check_output(args,cwd=R)
def sha(data):return hashlib.sha256(data).hexdigest()
def write(path,value):path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes((json.dumps(value,indent=2)+'\n').encode())
def copy(source,dest):
 assert source.is_file()and not source.is_symlink(),source
 dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(source.read_bytes())
assert not run('git','status','--porcelain').strip(),'Release staging requires clean Git.'
head=run('git','rev-parse','HEAD').decode().strip();run('python',str(B/'verify-foundation.py'))
assert S.resolve()==R/'target/player-release/MUDDLE'
if S.exists():shutil.rmtree(S)
S.mkdir(parents=True);runtime=[]
for name in ['index.html','app.js','style.css','worker.js','example-world.json']:
 data=run('git','show',head+':browser/player/web/'+name);assert data==(P/name).read_bytes(),f'Stale/non-Git web source: {name}'
 copy(P/name,S/name);runtime.append(name)
for name in ['muddle_browser_player.js','muddle_browser_player_bg.wasm']:copy(P/'pkg'/name,S/'pkg'/name);runtime.append('pkg/'+name)
source=[]
with zipfile.ZipFile(S/'source.zip','w',compression=zipfile.ZIP_DEFLATED,compresslevel=9)as archive:
 archive.comment=head.encode()
 for item in run('git','ls-tree','-rz','--full-tree',head).split(b'\0'):
  if not item:continue
  info,path=item.split(b'\t',1);mode,kind,oid=info.decode().split();path=path.decode()
  if path in ['LICENSE','Cargo.toml','Cargo.lock','browser/player/SOURCE-SCOPE.md']or path.startswith(('crates/','browser/player/','docs/rune/')):
   assert kind=='blob'and mode in ['100644','100755'],path
   data=run('git','cat-file','blob',oid);z=zipfile.ZipInfo('MUDDLE-software-source/'+path,date_time=(1980,1,1,0,0,0));z.compress_type=zipfile.ZIP_DEFLATED;z.external_attr=int(mode,8)<<16;archive.writestr(z,data);source.append({'path':path,'mode':mode,'bytes':len(data),'sha256':sha(data)})
write(S/'source-files.json',source)
for name,path in [('LICENSE','LICENSE'),('BUILD.md','browser/player/BUILD.md'),('MODEL.md','browser/player/MODEL.md'),('SOURCE-SCOPE.md','browser/player/SOURCE-SCOPE.md')]: (S/name).write_bytes(run('git','show',head+':'+path))
metadata=json.loads(run('cargo','+1.95.0','metadata','--manifest-path','browser/player/Cargo.toml','--locked','--format-version','1','--filter-platform','wasm32-unknown-unknown'));packages=[]
for package in metadata['packages']:
 folder=pathlib.Path(package['manifest_path']).parent
 if not package.get('source'):folder=R
 elif package['source'].startswith('git+')and not any(p.is_file()and p.name.upper().startswith(('LICENSE','NOTICE','COPYRIGHT','UNLICENSE'))for p in folder.iterdir()):folder=pathlib.Path(subprocess.check_output(['git','rev-parse','--show-toplevel'],cwd=folder).decode().strip())
 notices=[p for p in folder.iterdir()if p.is_file()and p.name.upper().startswith(('LICENSE','NOTICE','COPYRIGHT','UNLICENSE'))];assert notices,package['name'];records=[]
 for notice in sorted(notices):
  dest=S/'licenses/dependencies'/(package['name']+'-'+package['version'])/notice.name;copy(notice,dest);records.append({'path':dest.relative_to(S).as_posix(),'sha256':sha(notice.read_bytes())})
 packages.append({'name':package['name'],'version':package['version'],'source':package.get('source')or'first-party MUDDLE corresponding software source.zip','license':package['license'],'notices':records})
write(S/'dependencies.json',packages)
rustroot=pathlib.Path(run('rustc','+1.95.0','--print','sysroot').decode().strip())/'share/doc/rust';copy(rustroot/'COPYRIGHT-library.html',S/'rust/COPYRIGHT-library.html');notices=[]
for path in sorted((rustroot/'licenses').rglob('*')):
 if path.is_file():
  dest=S/'rust/licenses'/path.relative_to(rustroot/'licenses');copy(path,dest);notices.append({'path':dest.relative_to(S).as_posix(),'sha256':sha(path.read_bytes())})
assert notices;write(S/'rust/provenance.json',{'rustc':run('rustc','+1.95.0','-vV').decode().strip(),'copyright_sha256':sha((S/'rust/COPYRIGHT-library.html').read_bytes()),'notices':notices})
legal=['<h1>MUDDLE browser player notices</h1><p>Original split MIT software/ordinary software documentation and CC BY-NC original non-software content grants remain intact. Demo narratives remain embedded in their original software. Product-owned games are not added. Third-party grants remain their own.</p><p><a href="../LICENSE">Original MUDDLE license</a> · <a href="../SOURCE-SCOPE.md">Corresponding software source scope</a> · <a href="../dependencies.json">Resolved browser dependencies</a> · <a href="../rust/COPYRIGHT-library.html">Rust library copyright</a> · <a href="../rust/provenance.json">Rust notices</a></p>']
for package in packages:
 legal.append('<h2>'+html.escape(package['name']+' '+package['version'])+'</h2><p>'+html.escape(package['license']or'See notices')+'</p>')
 for notice in package['notices']:legal.append('<p><a href="'+html.escape('../'+notice['path'])+'">'+html.escape(pathlib.PurePosixPath(notice['path']).name)+'</a></p>')
(S/'licenses/index.html').parent.mkdir(exist_ok=True);(S/'licenses/index.html').write_bytes(('<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>MUDDLE licenses</title><body>'+''.join(legal)+'</body></html>').encode())
proof={'schema':'muddle.browser-player-release.v1','commit':head,'repository':'https://github.com/giodl73-repo/MUDDLE','native_entry':'e04fd1c0a17ba92ef2ef258a4ebcdb9b9097bc8b','source_scope':'Complete corresponding native software workspace, retained RUNE fixture and browser companion; NOT a full repository archive or authoritative product distribution. See SOURCE-SCOPE.md.','source_files':len(source),'source_zip_sha256':sha((S/'source.zip').read_bytes()),'wasm_sha256':sha((S/'pkg/muddle_browser_player_bg.wasm').read_bytes()),'rustc':run('rustc','+1.95.0','-vV').decode().strip(),'wasm_bindgen':'0.2.127','dependency_groups':len(packages),'rust_notice_files':len(notices),'runtime_files':runtime,'limits':{'request_bytes':524288,'save_bytes':131072,'world_bytes':65536,'report_bytes':2097152,'saved_commands_plus_new_attempts':128,'command_bytes':1024,'rooms':32,'exits':128,'interpreted_string_bytes':4096,'host_checkpoint_bytes':65536,'wasm_linear_memory_bytes':67108864},'budgets':{'runtime_bytes':2097152,'optional_source_zip_bytes':10485760,'site_bytes':26214400},'scope':'Text client over existing mock/spike/static hosts. Native commands/saves/checkpoints/transcripts/snapshots retained. No product-owned adoption, native graphics/filesystem parity, dynamic host loading or native security certification. Saves do not encode host identity.','native_advisory':'Existing GHSA-gg76-hg3v-5q6c macroquad0.4.14 advisory remains open in native workspace; macroquad/miniquad are absent from browser dependency closure.','source_replay':'Build outside Git with first-party paths inside extracted software; repeat both complete target outcomes, native CLI byte pairs and behavior separately. No cross-directory binary identity claim.'}
write(S/'provenance.json',proof);files=[{'path':p.relative_to(S).as_posix(),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())}for p in sorted(S.rglob('*'))if p.is_file()];runtime_bytes=sum((S/p).stat().st_size for p in runtime)
assert runtime_bytes<=2097152 and (S/'source.zip').stat().st_size<=10485760 and sum(p['bytes']for p in files)<=26214400
write(S/'inventory.json',{'commit':head,'runtime_bytes':runtime_bytes,'files':files,'bytes_excluding_inventory':sum(p['bytes']for p in files)})
print(json.dumps({'commit':head,'files':len(files)+1,'runtime_bytes':runtime_bytes,'source_files':len(source),'source_zip_bytes':(S/'source.zip').stat().st_size,'dependency_groups':len(packages),'rust_notices':len(notices)}))
