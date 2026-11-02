import init,{evaluate} from "./pkg/muddle_browser_player.js";
let identity;
self.onmessage=async({data})=>{
 if(data.type==="init"){
  identity={generation:data.generation,id:data.id};
  try{await init();self.postMessage({type:"ready",...identity});}catch(error){self.postMessage({type:"fatal",...identity,error:String(error)});}
 }else if(data.type==="run"&&identity&&data.generation===identity.generation&&data.id===identity.id){
  try{const start=performance.now();const raw=evaluate(data.input);self.postMessage({type:"report",...identity,raw,elapsedMs:performance.now()-start});}catch(error){self.postMessage({type:"fatal",...identity,error:String(error)});}
 }
};
