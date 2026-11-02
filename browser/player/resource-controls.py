"""Named static expectations for browser admission, never game rule changes."""
import copy,json
def controls():
 rows=[]
 def text(v):return json.dumps(v,ensure_ascii=False,separators=(',',':'))
 def add(name,q,kind):rows.append({'name':name,'request':q,'expected_kind':kind})
 def request(world,commands=None):return {'host':'authored-rooms','world_json':text(world),'commands':commands or []}
 world={'start_room':'r0','rooms':[{'id':f'r{i}','title':f'Room {i}','description':'Own static fixture','exits':[{'command':f'go {j}','target_room':f'r{(i+j+1)%32}','label':f'Exit {j}'}for j in range(4)]}for i in range(32)]}
 maximum=request(world,['look']*128);maximum['world_json']+=' '*(65536-len(maximum['world_json'].encode()));add('combined maximum:32 rooms/128 exits/128 attempts/64KiB world',maximum,'report')
 tiny={'start_room':'a','rooms':[{'id':'a','title':'A','description':'Own','exits':[]}]}
 for size in [4096,4097]:
  changed=copy.deepcopy(tiny);changed['rooms'][0]['title']='x'*size;add(f'interpreted string {size} bytes',request(changed),'report'if size==4096 else'browser_refusal')
 changed=copy.deepcopy(tiny);changed['rooms'][0]['description']='α'*2048;add('UTF8 string4096 bytes',request(changed),'report');changed['rooms'][0]['description']+='α';add('UTF8 string4098 bytes',request(changed),'browser_refusal')
 changed=copy.deepcopy(world);changed['rooms'].append({'id':'overflow','title':'Extra','description':'Own','exits':[]});add('33 rooms',request(changed),'browser_refusal')
 changed=copy.deepcopy(world);changed['rooms'][0]['exits'].append({'command':'go overflow','target_room':'r1','label':'Extra'});add('129 exits',request(changed),'browser_refusal')
 changed=copy.deepcopy(tiny);changed['rooms'][0]['exits']=[{'command':'go missing','target_room':'missing','label':'Missing'}];add('dangling target',request(changed),'browser_refusal')
 changed=copy.deepcopy(tiny);changed['rooms']*=2;add('duplicate room ID',request(changed),'browser_refusal')
 add('world65537 bytes',dict(maximum,world_json=maximum['world_json']+' '),'browser_refusal')
 for n in [1024,1025]:add(f'command {n} bytes',{'host':'mock-labyrinth','commands':['x'*n]},'report'if n==1024 else'browser_refusal')
 for n in [131072,131073]:add(f'save {n} bytes',{'host':'mock-labyrinth','save_text':'x'*n},'native_error'if n==131072 else'browser_refusal')
 for n in [65536,65537]:add(f'checkpoint {n} decoded bytes',{'host':'mock-labyrinth','save_text':'MUDDLE_SESSION_V1\ncurrent_room=labyrinth-camp\nhost_checkpoint='+'x'*n},'native_error'if n==65536 else'browser_refusal')
 add('saved128 plus new1',{'host':'mock-labyrinth','save_text':'MUDDLE_SESSION_V1\ncurrent_room=labyrinth-camp'+'\ncommand=look'*128,'commands':['look']},'browser_refusal')
 changed=copy.deepcopy(tiny);changed['rooms'][0]['title']='t'*4096;changed['rooms'][0]['description']='d'*4096;add('report2MiB refusal',request(changed,['look']*128),'browser_refusal')
 raw=text({'host':'mock-labyrinth'});raw+=' '*(524288-len(raw.encode()));add('request512KiB',raw,'report');add('request512KiB plus1',raw+' ','browser_refusal')
 return rows
