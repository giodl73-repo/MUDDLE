"""Prove default-disabled Rust embedding does not export the demo player."""
import json,os,pathlib,shutil,subprocess
P=pathlib.Path(__file__).resolve().parent;R=P.parents[1]
target=pathlib.Path(os.environ['CARGO_TARGET_DIR']);assert target.is_absolute()
subprocess.run(['cargo','+1.95.0','build','--locked','--manifest-path','browser/player/Cargo.toml','--no-default-features','--release','--target','wasm32-unknown-unknown','--lib'],cwd=R,check=True)
path=target/'wasm32-unknown-unknown/release/muddle_browser_player.wasm'
script="const fs=require('node:fs');const module=new WebAssembly.Module(fs.readFileSync(process.argv[1]));const names=WebAssembly.Module.exports(module).map(e=>e.name);if(names.includes('evaluate'))throw Error('Standalone demo entry leaked into embedding build');process.stdout.write(JSON.stringify({no_default_features:true,evaluate_export_absent:true,wasm_exports:names}));"
proof=json.loads(subprocess.check_output([shutil.which('node'),'-e',script,str(path)],cwd=R))
(P/'.cache').mkdir(exist_ok=True);(P/'.cache/embedding-exports.json').write_text(json.dumps(proof,indent=2)+'\n',encoding='utf-8');print(json.dumps(proof))
