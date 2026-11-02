import {test,expect} from "@playwright/test";
import fs from "node:fs";
const resources=JSON.parse(fs.readFileSync(new URL("../.cache/resource-controls.json",import.meta.url),"utf8"));
const ready=async page=>{await page.goto("./");await expect(page.locator("#status")).toContainText("Session ready");};
async function requestBytes(page){const [d]=await Promise.all([page.waitForEvent("download"),page.locator("#export-request").click()]);return fs.readFileSync(await d.path());}
// Fault workers are lifecycle probes only, never game outcomes or native goldens.
async function fault(page,mode){await page.evaluate(mode=>{
 const Real=window.Worker;window.restoreWorker=()=>window.Worker=Real;window.probe={terminated:0};
 window.Worker=class{
  constructor(){if(mode==="constructor")throw Error("synthetic constructor fault");}
  terminate(){window.probe.terminated++;}
  postMessage(data){
   const emit=value=>this.onmessage?.({data:value});
   if(data.type==="init"){
    this.identity={id:data.id,generation:data.generation};
    if(mode!=="startup")queueMicrotask(()=>emit({type:"ready",...this.identity}));
   }else if(data.type==="run"){
    if(mode==="crash"){queueMicrotask(()=>this.onerror?.({message:"synthetic crash"}));return;}
    if(mode==="malformed")queueMicrotask(async()=>{const hash=[...new Uint8Array(await crypto.subtle.digest("SHA-256",new TextEncoder().encode(data.input)))].map(x=>x.toString(16).padStart(2,"0")).join("");emit({type:"report",...this.identity,raw:JSON.stringify({kind:"report",snapshot:null,request_sha256:hash}),elapsedMs:0});});
    if(mode==="stale"){
     // Wrong IDs/generations and queued ready are intentionally synthetic.
     emit({type:"fatal",id:data.id+1,generation:data.generation,error:"wrong ID"});emit({type:"fatal",id:data.id,generation:data.generation-1,error:"wrong generation"});emit({type:"ready",...this.identity});
     window.emitOld=()=>emit({type:"fatal",...this.identity,error:"old terminated worker"});
    }
   }
  }
 };
},mode);}
for(const [mode,expected]of [["constructor","Worker unavailable"],["crash","Engine crashed"],["malformed","Malformed engine report"]])test(`${mode} retains accepted session and exports; real worker restart recovers`,async({page})=>{
 await ready(page);const before=await requestBytes(page);await fault(page,mode);await page.locator("#command").fill("look");await page.locator("#command").press("Enter");await expect(page.locator("#status")).toContainText(expected);expect(await requestBytes(page)).toEqual(before);await page.evaluate(()=>window.restoreWorker());await page.locator("#restart").click();await expect(page.locator("#status")).toContainText("Session ready");
});
for(const [mode,expected,ms]of [["startup","startup deadline",12000],["compute","Computation deadline",10000]])test(`actual ${mode} deadline terminates work and retains accepted session`,async({page})=>{
 await ready(page);const before=await requestBytes(page);await fault(page,mode);const start=Date.now();await page.locator("#command").fill("look");await page.locator("#command").press("Enter");await expect(page.locator("#status")).toContainText(expected,{timeout:ms+5000});expect(Date.now()-start).toBeGreaterThanOrEqual(ms-100);expect(await page.evaluate(()=>window.probe.terminated)).toBe(1);expect(await requestBytes(page)).toEqual(before);
});
test("new candidate input and explicit cancel terminate CPU; old IDs/generations cannot publish",async({page})=>{
 await ready(page);const before=await requestBytes(page);await fault(page,"stale");await page.locator("#command").fill("look");await page.locator("#command").press("Enter");await expect(page.locator("#status")).toContainText("Rust is replaying");await page.locator("#world").fill("new candidate, not active world");await expect(page.locator("#status")).toContainText("cancelled by newer input");await page.evaluate(()=>window.emitOld());await expect(page.locator("#status")).toContainText("cancelled by newer input");expect(await requestBytes(page)).toEqual(before);
 await page.locator("#command").fill("look");await page.locator("#command").press("Enter");await page.locator("#cancel").click();await page.evaluate(()=>window.emitOld());await expect(page.locator("#status")).toContainText("Computation cancelled");expect(await page.evaluate(()=>window.probe.terminated)).toBe(2);expect(await requestBytes(page)).toEqual(before);
});
for(const slot of ["save","world"])test(`actual ${slot} read deadline and invalid UTF-8 preserve prior input/original`,async({page})=>{
 await page.addInitScript(()=>{const read=File.prototype.arrayBuffer;File.prototype.arrayBuffer=function(){return this.name.startsWith("stall")?new Promise(()=>{}):read.call(this);};});await ready(page);
 await page.locator(`#${slot}-file`).setInputFiles({name:"original.txt",mimeType:"text/plain",buffer:Buffer.from("prior editor text")});await expect(page.locator(`#${slot}`)).toHaveValue("prior editor text");
 const start=Date.now();await page.locator(`#${slot}-file`).setInputFiles({name:"stall.txt",mimeType:"text/plain",buffer:Buffer.from("late")});await expect(page.locator("#status")).toContainText("Read deadline exceeded",{timeout:7500});expect(Date.now()-start).toBeGreaterThanOrEqual(4900);await expect(page.locator(`#${slot}`)).toHaveValue("prior editor text");
 await page.locator(`#${slot}-file`).setInputFiles({name:"invalid.txt",mimeType:"text/plain",buffer:Buffer.from([0xff])});await expect(page.locator("#status")).toContainText(/encoded data|UTF-8|utf-8/);await expect(page.locator(`#${slot}`)).toHaveValue("prior editor text");await expect(page.locator(`#${slot}-origin`)).toContainText("original.txt");
});
test("independent slot reads survive each other; editor and example intent fence stale file completion",async({page})=>{
 await page.addInitScript(()=>{const read=File.prototype.arrayBuffer;File.prototype.arrayBuffer=async function(){if(this.name.startsWith("slow"))await new Promise(r=>setTimeout(r,500));return read.call(this);};});await ready(page);
 await page.locator("#world-file").setInputFiles({name:"slow-world.json",mimeType:"application/json",buffer:Buffer.from("world read")});await page.locator("#save-file").setInputFiles({name:"save.txt",mimeType:"text/plain",buffer:Buffer.from("save read")});await expect(page.locator("#world")).toHaveValue("world read");await expect(page.locator("#save")).toHaveValue("save read");
 await page.locator("#world-file").setInputFiles({name:"slow-old.json",mimeType:"application/json",buffer:Buffer.from("must not replace edit")});await page.locator("#world").fill("new editor text");await page.waitForTimeout(650);await expect(page.locator("#world")).toHaveValue("new editor text");
 await page.locator("#world-file").setInputFiles({name:"slow-old2.json",mimeType:"application/json",buffer:Buffer.from("must not replace example")});await page.locator("#world-example").click();await page.waitForTimeout(650);await expect(page.locator("#world")).toHaveValue(/reading-room/);
});
test("whole-file and command bounds refuse requests while retaining accepted exports",async({page})=>{
 await ready(page);const before=await requestBytes(page);
 for(const [slot,max]of [["save",131072],["world",65536]]){await page.locator(`#${slot}`).fill("retained");await page.locator(`#${slot}-file`).setInputFiles({name:"oversized",mimeType:"text/plain",buffer:Buffer.alloc(max+1,120)});await expect(page.locator("#status")).toContainText("file exceeds");await expect(page.locator(`#${slot}`)).toHaveValue("retained");}
 await page.locator("#script").fill(Array(129).fill("look").join("\n"));await page.locator("#run-script").click();await expect(page.locator("#status")).toContainText("browser_refusal");expect(await requestBytes(page)).toEqual(before);await expect(page.locator("#turns")).toContainText("0 successful");
});
test("combined admitted maximum remains playable, inspectable and within mobile viewport",async({page})=>{
 await page.setViewportSize({width:390,height:844});await ready(page);const q=resources.find(x=>x.name.startsWith("combined maximum")).request;
 await page.locator("#world").fill(q.world_json);await page.locator("#host").selectOption("authored-rooms");await page.locator("#start").click();await expect(page.locator("#room-title")).toHaveText("r0");await page.locator("#script").fill(q.commands.join("\n"));await page.locator("#run-script").click();await expect(page.locator("#turns")).toContainText("128 successful turns");expect(await page.locator("#history li").count()).toBe(128);expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);const request=JSON.parse(await requestBytes(page));expect(request).toEqual(q);await page.locator("#command").fill("look");await page.locator("#command").press("Enter");await expect(page.locator("#status")).toContainText("browser_refusal");await expect(page.locator("#turns")).toContainText("128 successful turns");
});
