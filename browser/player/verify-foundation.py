"""Protect existing native consumers and source contracts against this candidate."""
import json,pathlib,subprocess
R=pathlib.Path(__file__).resolve().parents[2];ENTRY='e04fd1c0a17ba92ef2ef258a4ebcdb9b9097bc8b'
paths=['Cargo.toml','Cargo.lock','crates','docs/compatibility.md','docs/rune','.pitfall','.roles','PRODUCT_PLAN.md','context/waves/2026-05-17-muddle-mud-engine']
for args in [['diff','--name-only',ENTRY,'--',*paths],['diff','--cached','--name-only',ENTRY,'--',*paths]]:
 assert not subprocess.check_output(['git',*args],cwd=R).strip(),'Protected native/policy/content changes'
old=subprocess.check_output(['git','show',ENTRY+':README.md'],cwd=R).decode();assert (R/'README.md').read_text(encoding='utf-8').startswith(old),'README front door is append-only'
print(json.dumps({'entry':ENTRY,'foundation_core_cli_hosts_native_renderers_workspace_locks_rune_policy':'unchanged','inherited_native_cleanup_requires_banish_rehearsal':True}))
