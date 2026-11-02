const $=id=>document.getElementById(id),enc=new TextEncoder();
let accepted=null,worker=null,generation=0,revision=0,pending=null,timer=null,recall=[],recallAt=0;
const slots={save:{token:0,max:131072,original:null},world:{token:0,max:65536,original:null}};
const paths={"mock-labyrinth":["gather ember","go antechamber","inspect glyphs","use ember","go vault"],"banish-pilgrim-loss":["choose resume","inspect plan","inspect manifest","go trail","resolve loss"],"amaze-silverstream":["go receiver","inspect clue","tune signal","unlock hatch","go hatch"],"authored-rooms":["look","go garden","look","go inside"]};
const status=text=>{$("status").textContent=text;};
const bytes=text=>enc.encode(text).length;
const clone=value=>JSON.parse(JSON.stringify(value));
function download(data,name,type="text/plain;charset=utf-8"){
 const url=URL.createObjectURL(new Blob([data],{type})),a=document.createElement("a");a.href=url;a.download=name;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);
}
function stop(){clearTimeout(timer);timer=null;generation++;if(worker)worker.terminate();worker=null;pending=null;$("cancel").disabled=true;$("send").disabled=!accepted;}
function intent(){revision++;if(pending){stop();status("Pending computation cancelled by newer input. Accepted session and downloads retained.");}}
function fail(message){stop();status(message+" Accepted session and inputs retained. Source and build downloads remain available.");}
function validReport(r,q){
 if(r.kind!=="report")return ["native_error","input_error","browser_refusal"].includes(r.kind)&&typeof r.error==="string";
 const s=r.snapshot;
 const records=(a,keys)=>Array.isArray(a)&&a.every(x=>x&&keys.every(k=>typeof x[k]==="string"));
 if(!s||!s.panels||!records(s.panels.resources,["label","value"])||!records(s.panels.inventory,["label","detail"])||!Array.isArray(s.panels.objectives)||!s.panels.objectives.every(x=>typeof x==="string")||!(s.panels.map===null||typeof s.panels.map==="string")||!records(s.history,["room","command","response"]))return false;
 if(q.host==="authored-rooms"){let w;try{w=JSON.parse(q.world_json);}catch{return false;}const room=w.rooms?.find(x=>x.id===s.room);if(!room||!records(room.exits,["command"]))return false;}
 return s&&s.host===q.host&&typeof s.room==="string"&&typeof s.room_card==="string"&&typeof s.last_response==="string"&&Number.isSafeInteger(s.turns)&&s.turns>=0&&Array.isArray(s.history)&&s.history.length===s.turns&&Array.isArray(s.commands)&&s.commands.every(c=>typeof c.command==="string"&&typeof c.description==="string")&&Array.isArray(s.controls)&&Array.isArray(s.visual_nodes)&&s.panels&&Array.isArray(s.panels.resources)&&Array.isArray(s.panels.inventory)&&Array.isArray(s.panels.objectives)&&Array.isArray(r.attempts)&&r.attempts.length===q.commands.length&&[r.save_text,r.replay_save_text,r.transcript_text].every(x=>typeof x==="string");
}
async function run(q){
 stop();const input=JSON.stringify(q);if(bytes(input)>524288){status("Browser refusal: request exceeds 512 KiB. Accepted session retained.");return;}
 const id=++revision,g=++generation,rev=revision;pending={id,generation:g,revision:rev,q,input};$("cancel").disabled=false;$("send").disabled=true;status("Starting Rust worker…");
 const alive=()=>pending&&pending.id===id&&pending.generation===g&&pending.revision===rev&&revision===rev&&generation===g;
 try{worker=new Worker(new URL("./worker.js",import.meta.url),{type:"module"});}catch(e){fail("Worker unavailable: "+e);return;}
 timer=setTimeout(()=>{if(alive())fail("Engine startup deadline exceeded (12 seconds).");},12000);
 worker.onerror=()=>{if(alive())fail("Engine crashed. Try the action again or restart the active host.");};
 worker.onmessage=async({data})=>{
  if(!alive()||data.id!==id||data.generation!==g)return;
  if(data.type==="ready"){
   if(pending.phase==="running")return;pending.phase="running";clearTimeout(timer);timer=setTimeout(()=>{if(alive())fail("Computation deadline exceeded (10 seconds).");},10000);status("Rust is replaying the session…");worker.postMessage({type:"run",id,generation:g,input});
  }else if(data.type==="fatal"){fail("Engine unavailable: "+data.error);
  }else if(data.type==="report"){
   try{
    if(pending.phase!=="running"||typeof data.raw!=="string"||bytes(data.raw)>2097152)throw Error("invalid report envelope");
    const r=JSON.parse(data.raw),hash=[...new Uint8Array(await crypto.subtle.digest("SHA-256",enc.encode(input)))].map(x=>x.toString(16).padStart(2,"0")).join("");
    if(!alive())return;if(r.request_sha256!==hash||!validReport(r,q))throw Error("report schema or request hash mismatch");
    if(r.kind!=="report"){fail(r.kind+": "+r.error);return;}
    accepted={q:clone(q),raw:data.raw,r,input};stop();render(data.elapsedMs);status("Session ready. Native host results are shown below.");
   }catch(e){if(alive())fail("Malformed engine report: "+e);}
  }
 };
 worker.postMessage({type:"init",id,generation:g});
}
function selected(save=false){const host=$("host").value;return {host,commands:[],...(host==="authored-rooms"?{world_json:$("world").value}:{}),...(save?{save_text:$("save").value}:{})};}
function history(){
 const list=$("history");list.replaceChildren();if(!accepted)return;const all=accepted.r.snapshot.history,filter=$("history-filter").value.toLowerCase(),rows=all.filter(h=>JSON.stringify(h).toLowerCase().includes(filter));$("history-count").textContent=`${rows.length} of ${all.length} successful turns`;
 for(const h of rows){const li=document.createElement("li"),b=document.createElement("strong"),p=document.createElement("p");b.textContent=`${h.turn}. ${h.room} › ${h.command}`;p.textContent=h.response;li.append(b,p);list.append(li);}
}
function render(ms){
 const {r,q}=accepted,s=r.snapshot;$("active").textContent=`Active host: ${q.host}. Host/save/world editors are candidate inputs until started or resumed.`;$("room-title").textContent=s.room;$("room").textContent=s.room_card;$("response").textContent=s.last_response;$("turns").textContent=`${s.turns} successful turns · ${r.attempts.length} attempts after imported save.`;
 $("panels").textContent=[...s.panels.resources.map(x=>`${x.label}: ${x.value}`),...s.panels.inventory.map(x=>`${x.label}: ${x.detail}`),...s.panels.objectives,...(s.panels.map?[s.panels.map]:[])].join("\n");
 $("snapshot").textContent=JSON.stringify(s,null,2);$("attempts").textContent=JSON.stringify(r.attempts,null,2);$("timing").textContent=`Worker replay and report serialization: ${Number(ms).toFixed(1)} ms. No comparative speed claim.`;
 $("hints").replaceChildren();for(const c of s.commands){const b=document.createElement("button");b.textContent=c.command;b.title=c.description;b.addEventListener("click",()=>command(c.command));$("hints").append(b);}
 // StaticHost intentionally has no command-panel override. Expose its exact stored exits.
 if(q.host==="authored-rooms"){
  const room=JSON.parse(q.world_json).rooms.find(x=>x.id===s.room);
  for(const text of ["look",...room.exits.map(e=>e.command)]){const b=document.createElement("button");b.textContent=text;b.addEventListener("click",()=>command(text));$("hints").append(b);}
 }
 document.querySelectorAll("[data-export]").forEach(b=>b.disabled=false);$("export-request").disabled=false;$("export-report").disabled=false;$("send").disabled=false;history();
}
function command(text){if(!accepted)return;const q=clone(accepted.q);q.commands.push(text);recall.push(text);recallAt=recall.length;$("command").value="";run(q);}
$("command-form").addEventListener("submit",e=>{e.preventDefault();command($("command").value);});
$("command").addEventListener("keydown",e=>{if(e.key==="ArrowUp"||e.key==="ArrowDown"){e.preventDefault();recallAt=Math.max(0,Math.min(recall.length,recallAt+(e.key==="ArrowUp"?-1:1)));e.target.value=recall[recallAt]||"";}});
$("start").onclick=()=>run(selected());$("resume").onclick=()=>run(selected(true));
$("restart").onclick=()=>{if(accepted){const q=clone(accepted.q);delete q.save_text;q.commands=[];run(q);}else run(selected());};
$("cancel").onclick=()=>{revision++;stop();status("Computation cancelled. Accepted session and downloads retained.");};
$("host").onchange=intent;$("script").oninput=intent;
$("run-script").onclick=()=>{if(!accepted){status("Start a host before playing a script.");return;}const q=clone(accepted.q);q.commands.push(...$("script").value.split(/\r?\n/).filter(x=>x.trim()!==""));run(q);};
$("example-script").onclick=()=>{intent();$("script").value=(paths[accepted?.q.host||$("host").value]||[]).join("\n");};
$("history-filter").oninput=history;
document.querySelectorAll("[data-export]").forEach(b=>b.onclick=()=>{if(accepted)download(accepted.r[b.dataset.export],b.dataset.export==="transcript_text"?"session-transcript.txt":b.dataset.export==="save_text"?"session-checkpoint.muddle":"session-replay.muddle");});
$("export-request").onclick=()=>{if(accepted)download(accepted.input,"session-request.json","application/json");};$("export-report").onclick=()=>{if(accepted)download(accepted.raw,"session-report.json","application/json");};
async function readFile(name,file){
 const slot=slots[name],token=++slot.token;intent();if(!file)return;
 if(file.size>slot.max){status(`Browser refusal: ${name} file exceeds ${slot.max} bytes. Prior inputs/session retained.`);return;}
 let timeout;
 try{
  const buffer=await Promise.race([file.arrayBuffer(),new Promise((_,reject)=>timeout=setTimeout(()=>reject(Error("Read deadline exceeded (5 seconds)")),5000))]);
  if(token!==slot.token)return;const raw=new Uint8Array(buffer);if(raw.length>slot.max)throw Error("File exceeds byte bound");const text=new TextDecoder("utf-8",{fatal:true,ignoreBOM:true}).decode(raw);slot.original=raw;$(name).value=text;$(name+"-origin").textContent=`Opened ${file.name}: ${raw.length} bytes. Original bytes remain separate from editor text.`;$(name+"-original").disabled=false;status("Input opened. Use Start or Resume to propose it; active session retained.");
 }catch(e){if(token===slot.token)status(`Could not read ${name}: ${e}. Prior inputs/session retained.`);}finally{clearTimeout(timeout);}
}
for(const name of ["save","world"]){
 $(name+"-file").onchange=e=>readFile(name,e.target.files[0]);$(name).oninput=()=>{slots[name].token++;intent();};
 $(name).ondragover=e=>e.preventDefault();$(name).ondrop=e=>{e.preventDefault();readFile(name,e.dataTransfer.files[0]);};
 $(name+"-original").onclick=()=>{if(slots[name].original)download(slots[name].original,`opened-${name}-original`);};$(name+"-current").onclick=()=>download($(name).value,`current-${name}.txt`);
 $(name+"-lf").onclick=()=>{slots[name].token++;intent();$(name).value=$(name).value.replace(/^\uFEFF/,"").replace(/\r\n?/g,"\n");status("Explicit LF editor copy selected. Opened original unchanged.");};
}
async function exampleWorld(){const token=++slots.world.token;intent();try{const r=await fetch(new URL("./example-world.json",import.meta.url));if(!r.ok)throw Error(r.status);const text=await r.text();if(token!==slots.world.token)return;$("world").value=text;}catch(e){if(token===slots.world.token)status("Example unavailable: "+e);}}
$("world-example").onclick=exampleWorld;
exampleWorld();run(selected());
