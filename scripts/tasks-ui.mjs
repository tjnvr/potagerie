#!/usr/bin/env node
// Local board of docs/tasks/: statuses, dependency order, details, launch.
// No dependency: node scripts/tasks-ui.mjs, then http://localhost:4321 (TASKS_PORT to change).
import { createServer } from "node:http";
import { execFile } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { ROOT, HOME, PORT, fetchOrigin, readTasks, answerTask, publish } from "./tasks-lib.mjs";

const body = (req) => new Promise((ok, ko) => {
  let b = ""; req.on("data", (c) => (b += c)); req.on("end", () => { try { ok(JSON.parse(b || "{}")); } catch (e) { ko(e); } });
});

const send = (res, code, body, type = "application/json") => {
  res.writeHead(code, { "content-type": `${type}; charset=utf-8`, "cache-control": "no-store" });
  res.end(type === "application/json" ? JSON.stringify(body) : body);
};

const server = createServer((req, res) => {
  const url = new URL(req.url, "http://localhost");
  const m = url.pathname.match(/^\/api\/(run|log)\/(T\d+)$/);
  if (req.method === "GET" && url.pathname === "/") return send(res, 200, PAGE, "text/html");
  if (url.pathname === "/favicon.ico") { res.writeHead(204); return res.end(); }
  if (req.method === "GET" && url.pathname === "/api/tasks") {
    fetchOrigin(url.searchParams.has("fetch"));
    return send(res, 200, readTasks());
  }
  const am = url.pathname.match(/^\/api\/answer\/(T\d+)$/);
  if (am && req.method === "POST") {
    body(req).then(({ n, answer, resume, ready }) => {
      if (!Number.isInteger(n) || typeof answer !== "string" || !answer.trim()) throw new Error("question number and answer required");
      let out = answerTask(am[1], n, answer, { ready: !!ready });
      if (!resume) return send(res, 200, { ok: true, out });
      execFile(join(ROOT, "scripts/tasks.sh"), ["run", am[1]], { cwd: ROOT }, (err, o, e) =>
        send(res, err ? 409 : 200, { ok: !err, out: out + "\n" + (o + e).trim() }));
    }).catch((e) => send(res, 400, { ok: false, out: String(e.message || e) }));
    return;
  }
  if (m && m[1] === "log" && req.method === "GET") {
    const p = join(HOME, "logs", `${m[2]}.log`);
    const lines = existsSync(p) ? readFileSync(p, "utf8").split("\n").slice(-200).join("\n") : "";
    return send(res, 200, { log: lines });
  }
  if (m && m[1] === "run" && req.method === "POST") {
    execFile(join(ROOT, "scripts/tasks.sh"), ["run", m[2]], { cwd: ROOT }, (err, out, errOut) =>
      send(res, err ? 409 : 200, { ok: !err, out: (out + errOut).trim() }));
    return;
  }
  if (url.pathname === "/api/publish" && req.method === "POST") {
    try { return send(res, 200, { ok: true, out: publish() }); }
    catch (e) { return send(res, 409, { ok: false, out: String(e.stderr || e.message || e).trim() }); }
  }
  if (url.pathname === "/api/next" && req.method === "POST") {
    execFile(join(ROOT, "scripts/tasks.sh"), ["next"], { cwd: ROOT }, (err, out, errOut) =>
      send(res, err ? 409 : 200, { ok: !err, out: (out + errOut).trim() }));
    return;
  }
  send(res, 404, { error: "not found" });
});

server.listen(PORT, "127.0.0.1", () => {
  fetchOrigin(true);
  console.log(`Potagerie tasks: http://localhost:${PORT}  (${ROOT})`);
});

const PAGE = String.raw`<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Potagerie · Tasks</title>
<style>
:root{--bg:#f4f1e8;--card:#fffdf7;--ink:#26342b;--muted:#4e6858;--line:#ddd6c4;--green:#3f7d4e;
--blocked:#c2553a;--running:#2f6fa8;--ready:#3f7d4e;--draft:#8a8170;--done:#9aa89e;
font-family:ui-sans-serif,system-ui,-apple-system,"Segoe UI",sans-serif;color:var(--ink);background:var(--bg)}
*{box-sizing:border-box}body{margin:0}
header{display:flex;align-items:center;gap:16px;padding:14px 22px;border-bottom:1px solid var(--line);background:var(--card);position:sticky;top:0;z-index:2}
h1{font-size:17px;margin:0;font-weight:650}h1 span{color:var(--muted);font-weight:500}
.tabs{display:flex;gap:4px;margin-left:12px}.tabs button,.btn{border:1px solid var(--line);background:var(--card);border-radius:8px;padding:6px 12px;font:inherit;font-size:13px;cursor:pointer;color:var(--ink)}
.tabs button[aria-pressed=true]{background:var(--ink);color:#fff;border-color:var(--ink)}
.btn.primary{background:var(--green);border-color:var(--green);color:#fff}.btn:disabled{opacity:.45;cursor:default}
.meta{margin-left:auto;font-size:12px;color:var(--muted);display:flex;gap:10px;align-items:center}
main{display:grid;grid-template-columns:1fr 420px;min-height:calc(100vh - 57px)}
#view{padding:18px 22px;overflow:auto}
.board{display:grid;grid-template-columns:repeat(5,minmax(170px,1fr));gap:14px;align-items:start}
.col h2{font-size:12px;text-transform:uppercase;letter-spacing:.06em;margin:0 0 8px;display:flex;gap:6px;align-items:center}
.dot{width:9px;height:9px;border-radius:50%;display:inline-block}
.col h2 .n{color:var(--muted);font-weight:500}
.card{background:var(--card);border:1px solid var(--line);border-left:4px solid var(--c);border-radius:10px;padding:9px 11px;margin-bottom:8px;cursor:pointer}
.card:hover{border-color:var(--muted)}.card.sel{outline:2px solid var(--ink)}
.card .id{font-size:11px;color:var(--muted);font-variant-numeric:tabular-nums;display:flex;gap:6px;align-items:center}
.card .t{font-size:13.5px;line-height:1.3;margin-top:3px}
.chip{font-size:10.5px;border-radius:99px;padding:1px 7px;background:#ece6d6;color:var(--muted)}
.chip.tim{background:#f3dfc9;color:#8a4b1c}.chip.q{background:var(--running);color:#fff}
.sub{font-size:11.5px;color:var(--muted);margin-top:5px}
aside{border-left:1px solid var(--line);background:var(--card);padding:18px 20px;overflow:auto;max-height:calc(100vh - 57px);position:sticky;top:57px}
aside .empty{color:var(--muted);font-size:14px;margin-top:40px;text-align:center}
.doc h1{font-size:18px;margin:0 0 10px}.doc h2{font-size:13px;text-transform:uppercase;letter-spacing:.05em;color:var(--muted);margin:18px 0 6px}
.doc p,.doc li{font-size:13.5px;line-height:1.5}.doc ul{padding-left:18px;margin:4px 0}.doc code{background:#efe9da;padding:1px 4px;border-radius:4px;font-size:12px}
.doc li.task{list-style:none;margin-left:-18px}
.actions{display:flex;gap:8px;align-items:center;margin:4px 0 12px;flex-wrap:wrap}.hint{font-size:12px;color:var(--muted)}
textarea{width:100%;font:inherit;font-size:13px;border:1px solid var(--line);border-radius:8px;padding:8px;background:#fff;resize:vertical}.qa{margin-bottom:12px}.qa p{margin:0 0 6px}
pre.log{background:#1f2a23;color:#d9e6dc;font-size:11.5px;padding:10px;border-radius:8px;max-height:300px;overflow:auto;white-space:pre-wrap}
.queue{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:10px 14px;margin-bottom:16px;font-size:13px;display:flex;gap:10px;align-items:center;flex-wrap:wrap}
svg text{font-family:inherit}.sec{font-size:12px;text-transform:uppercase;letter-spacing:.06em;color:var(--muted);margin:14px 0 6px}.done-toggle{font-size:12px;color:var(--muted);cursor:pointer;background:none;border:none;padding:0}
</style></head><body>
<header><h1>Potagerie <span>· Tasks</span></h1>
<div class="tabs"><button data-v="board" aria-pressed="true">Board</button><button data-v="order" aria-pressed="false">Order</button></div>
<div class="meta"><span id="loop"></span><span id="stamp"></span><button class="btn" id="publish" hidden>Publish</button><button class="btn" id="refresh">Fetch</button></div></header>
<main><section id="view"></section><aside id="detail"><p class="empty">Select a task</p></aside></main>
<script>
const C={Blocked:"var(--blocked)",Running:"var(--running)",Ready:"var(--ready)",Draft:"var(--draft)",Done:"var(--done)"};
let data=null,view="board",sel=null,showDone=false;
const $=s=>document.querySelector(s);
const esc=s=>s.replace(/[&<>"]/g,c=>({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;"}[c]));
function md(src){
  const inline=s=>esc(s).replace(/\x60([^\x60]+)\x60/g,"<code>$1</code>").replace(/\*\*([^*]+)\*\*/g,"<b>$1</b>");
  let out="",inList=false;
  for(const raw of src.replace(/<!--[\s\S]*?-->/g,"").split("\n")){
    const l=raw.trimEnd();let m;
    const close=()=>{if(inList){out+="</ul>";inList=false}};
    if(m=l.match(/^(#{1,3}) (.*)/)){close();out+="<h"+(m[1].length===1?1:2)+">"+inline(m[2])+"</h"+(m[1].length===1?1:2)+">"}
    else if(m=l.match(/^\s*(?:[-*]|\d+\.) (.*)/)){if(!inList){out+="<ul>";inList=true}
      const c=m[1].match(/^\[( |x)\] (.*)/);out+=c?'<li class="task">'+(c[1]==="x"?"☑":"☐")+" "+inline(c[2])+"</li>":"<li>"+inline(m[1])+"</li>"}
    else if(l.trim()===""){close()}
    else{close();out+="<p>"+inline(l)+"</p>"}
  }
  return out+(inList?"</ul>":"");
}
function questions(t){return t.openQuestions||0}
function card(t){
  const q=questions(t);
  return '<div class="card'+(sel===t.id?" sel":"")+'" style="--c:'+C[t.status]+'" data-id="'+t.id+'"><div class="id">'+t.id+
  (t.owner==="Tim"?' <span class="chip tim">Tim</span>':"")+(t.alive?' <span class="chip q">agent running</span>':"")+(t.stalled?' <span class="chip tim">stalled</span>':"")+(t.stuck?' <span class="chip tim">stuck</span>':"")+(t.answered?' <span class="chip">answered, to resume</span>':"")+(!t.pushed?' <span class="chip tim">not pushed</span>':t.edited?' <span class="chip">edited</span>':"")+(t.status!=="Done"&&q?' <span class="chip q">'+q+" ?</span>":"")+
  '</div><div class="t">'+esc(t.title)+"</div>"+
  (t.waitsFor.length&&t.status!=="Done"?'<div class="sub">after '+t.waitsFor.join(", ")+"</div>":"")+
  (t.branch&&(t.status==="Ready")?'<div class="sub">started, see the log</div>':"")+(data.queue.includes(t.id)?'<div class="sub">next #'+(data.queue.indexOf(t.id)+1)+"</div>":"")+"</div>";
}
function board(){
  const cols=data.order.map(s=>{
    let ts=data.tasks.filter(t=>t.status===s);
    if(s==="Ready")ts.sort((a,b)=>(data.queue.indexOf(a.id)+1||99)-(data.queue.indexOf(b.id)+1||99)||a.num-b.num);
    const body=s==="Done"&&!showDone&&ts.length?'<button class="done-toggle" id="dt">Show '+ts.length+"</button>":ts.map(card).join("");
    return '<div class="col"><h2><span class="dot" style="background:'+C[s]+'"></span>'+s+' <span class="n">'+ts.length+"</span></h2>"+body+"</div>"}).join("");
  const need=data.tasks.filter(t=>t.openQuestions>0).length;
  const q='<div class="queue"><b>Running '+data.running+" / "+data.max+"</b>"+(need?'<span class="chip q">'+need+" question"+(need>1?"s":"")+" for you</span>":"")+"<span>Next: "+(data.queue.slice(0,5).join(" → ")||"nothing ready")+
    '</span><button class="btn primary" id="next" '+(data.queue.length&&data.running<data.max?"":"disabled")+">Start next</button></div>";
  const warn=data.unpushed?'<div class="queue">'+data.unpushed+" task(s) only on your disk: an agent can start them once they are pushed to "+data.base+".</div>":"";
  return warn+q+'<div class="board">'+cols+"</div>";
}
function order(){
  const ts=data.tasks.filter(t=>showDone||t.status!=="Done"),by={};ts.forEach(t=>by[t.id]=t);
  const parents=t=>t.deps.filter(d=>by[d]),kids=t=>ts.filter(o=>o.deps.includes(t.id));
  const linked=ts.filter(t=>parents(t).length||kids(t).length),alone=ts.filter(t=>!linked.includes(t));
  const W=210,H=54,GX=70,GY=14,pos={};
  // Columns by depth; each node sits at the mean height of the tasks it waits for.
  const depthOf=t=>parents(t).length?1+Math.max(...parents(t).map(p=>depthOf(by[p]))):0;
  const cols=[];linked.forEach(t=>{const d=depthOf(t);(cols[d]=cols[d]||[]).push(t)});
  // Roots: grouped by the subtree they feed, in id order.
  const rootKey=t=>{const seen=new Set(),st=[t];let m=t.num;while(st.length){const x=st.pop();if(seen.has(x.id))continue;seen.add(x.id);m=Math.min(m,x.num);kids(x).forEach(k=>st.push(k));parents(x).forEach(p=>st.push(by[p]))}return m};
  let maxY=0;
  cols.forEach((col,d)=>{
    if(!col)return;
    const want=t=>d===0?0:parents(t).reduce((a,p)=>a+(pos[p]?pos[p].y:0),0)/parents(t).length;
    col.sort(d===0?(a,b)=>rootKey(a)-rootKey(b)||a.num-b.num:(a,b)=>want(a)-want(b)||a.num-b.num);
    let y=20;
    col.forEach(t=>{const ny=d===0?y:Math.max(want(t),y);pos[t.id]={x:20+d*(W+GX),y:ny};y=ny+H+GY;maxY=Math.max(maxY,ny+H)});
    // Roots spread so their children have room: place each root group by its widest column later.
  });
  // Second pass: pull each parent down to the mean of its children when there is room (keeps edges flat).
  for(let d=cols.length-2;d>=0;d--){const col=cols[d]||[];let y=20;
    col.forEach(t=>{const k=kids(t).filter(c=>pos[c.id]);const ideal=k.length?k.reduce((a,c)=>a+pos[c.id].y,0)/k.length:pos[t.id].y;
      const ny=Math.max(y,Math.min(ideal,Infinity));pos[t.id].y=ny;y=ny+H+GY;maxY=Math.max(maxY,ny+H)});}
  const minY=Math.min(...linked.map(t=>pos[t.id].y),Infinity);
  if(isFinite(minY)){linked.forEach(t=>pos[t.id].y-=minY-12);maxY-=minY-12}
  const w=Math.max(20+cols.length*(W+GX),700),hGraph=linked.length?maxY+16:0;
  let s='<svg width="'+w+'" height="'+hGraph+'" style="display:block"><defs><marker id="a" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" orient="auto"><path d="M0 0L8 4L0 8z" fill="#8a8170"/></marker></defs>';
  linked.forEach(t=>parents(t).forEach(d=>{const a=pos[d],b=pos[t.id];
    s+='<path d="M'+(a.x+W)+" "+(a.y+H/2)+" C"+(a.x+W+GX/2)+" "+(a.y+H/2)+" "+(b.x-GX/2)+" "+(b.y+H/2)+" "+(b.x-2)+" "+(b.y+H/2)+'" fill="none" stroke="#8a8170" stroke-width="1.4" marker-end="url(#a)"/>'}));
  const node=(t,x,y)=>'<g class="node" data-id="'+t.id+'" style="cursor:pointer"><rect x="'+x+'" y="'+y+'" width="'+W+'" height="'+H+'" rx="10" fill="#fffdf7" stroke="'+(sel===t.id?"#26342b":"#ddd6c4")+'" stroke-width="'+(sel===t.id?2:1)+'"/>'+
    '<rect x="'+x+'" y="'+y+'" width="5" height="'+H+'" rx="2" fill="'+C[t.status]+'"/>'+
    '<text x="'+(x+14)+'" y="'+(y+20)+'" font-size="11" fill="#4e6858">'+t.id+" · "+t.status+(t.owner==="Tim"?" · Tim":"")+"</text>"+
    '<text x="'+(x+14)+'" y="'+(y+38)+'" font-size="12.5" fill="#26342b">'+esc(t.title.length>28?t.title.slice(0,27)+"…":t.title)+"</text></g>";
  linked.forEach(t=>s+=node(t,pos[t.id].x,pos[t.id].y));s+="</svg>";
  // Independent tasks: compact grid.
  const per=Math.max(1,Math.floor(((document.getElementById("view").clientWidth||900)-40)/(W+16)));
  const rows=Math.ceil(alone.length/per);
  let g='<svg width="'+(20+per*(W+16))+'" height="'+(rows*(H+GY)+10)+'" style="display:block">';
  alone.forEach((t,i)=>g+=node(t,20+(i%per)*(W+16),4+Math.floor(i/per)*(H+GY)));g+="</svg>";
  const head='<div class="queue">Left to right: a task waits for the ones that point to it. <label><input type="checkbox" id="sd" '+(showDone?"checked":"")+"> Show done</label></div>";
  return head+(linked.length?'<h2 class="sec">Linked</h2>'+s:"")+(alone.length?'<h2 class="sec">Independent</h2>'+g:"");
}
function qparse(text){
  const parts=text.split(/^## Questions\s*$/m);if(parts.length<2)return null;
  const sec=parts[1].split(/^## /m)[0].replace(/<!--[\s\S]*?-->/g,"");const qs=[];let cur=null;
  for(const l of sec.split("\n")){const m=l.match(/^\s*(\d+)\.\s+(.*)/);
    if(m){cur={n:+m[1],q:m[2],a:""};qs.push(cur)}
    else if(cur){const a=l.match(/^\s*Answer:\s*(.*)/);if(a)cur.a=a[1];else if(l.trim())cur.q+=" "+l.trim()}}
  return {before:parts[0],after:parts[1].includes("\n## ")?"## "+parts[1].split(/^## /m).slice(1).join("## "):"",qs};
}
function qform(t,p){
  if(!p.qs.length)return '<h2>Questions</h2><p class="hint">None.</p>';
  const can=t.status==="Blocked"&&t.runnable;
  return '<h2>Questions</h2>'+p.qs.map(q=>'<div class="qa"><p><b>'+q.n+'.</b> '+md(q.q).replace(/^<p>|<\/p>$/g,"")+'</p>'+
    '<textarea data-n="'+q.n+'" rows="3" placeholder="Your answer">'+esc(q.a)+'</textarea><div class="actions">'+
    '<button class="btn" data-save="'+q.n+'">'+(q.a?"Update":"Save")+'</button>'+
    (can?'<button class="btn primary" data-resume="'+q.n+'">Save and resume '+t.id+'</button>':"")+
    (t.status==="Draft"?'<button class="btn primary" data-ready="'+q.n+'">Save and set Ready</button>':"")+
    (q.a?'<span class="hint">answered</span>':"")+'</div></div>').join("");
}
async function answer(t,n,resume,ready){
  const ta=document.querySelector('textarea[data-n="'+n+'"]');if(!ta.value.trim())return alertBox("Write an answer first.");
  const r=await fetch("/api/answer/"+t.id,{method:"POST",headers:{"content-type":"application/json"},body:JSON.stringify({n,answer:ta.value,resume,ready:!!ready})}).then(r=>r.json());
  alertBox(r.out);await load(true);detail();
}
async function detail(){
  const t=data&&data.tasks.find(x=>x.id===sel);if(!t){$("#detail").innerHTML='<p class="empty">Select a task</p>';return}
  let why="";
  if(t.alive)why="An agent is working on it: follow the log below.";
  else if(!t.pushed)why="Not on "+data.base+" yet: commit and push it, the agent starts from there.";
  else if(t.owner==="Tim")why="For Tim: no agent will take it.";
  else if(t.status==="Draft")why="Draft: answer its questions and set it Ready first.";
  else if(t.status==="Done")why="Done.";
  else if(t.waitsFor.length)why="Waits for "+t.waitsFor.join(", ")+".";
  const label=t.stuck?"Retry":t.status==="Blocked"?"Resume":t.stalled?"Restart":"Start";
  let h='<div class="actions"><button class="btn primary" id="run" '+(t.runnable?"":"disabled")+">"+label+" "+t.id+'</button><span class="hint">'+(t.runnable?(t.branch?"branch "+t.branch:"new worktree from "+data.base):why)+"</span></div>";
  const qp=qparse(t.text);
  h+='<div class="doc">'+(qp?md(qp.before)+qform(t,qp)+md(qp.after):md(t.text))+"</div>";
  if(t.unblocks.length)h+='<p class="hint">Unblocks: '+t.unblocks.join(", ")+"</p>";
  if(t.hasLog){const r=await fetch("/api/log/"+t.id).then(r=>r.json());h+='<h2 style="font-size:13px;color:#4e6858">Agent log</h2><pre class="log">'+esc(r.log||"(empty)")+"</pre>"}
  $("#detail").innerHTML=h;
  document.querySelectorAll("[data-save]").forEach(b=>b.onclick=()=>answer(t,+b.dataset.save,false));
  document.querySelectorAll("[data-resume]").forEach(b=>b.onclick=()=>answer(t,+b.dataset.resume,true));
  document.querySelectorAll("[data-ready]").forEach(b=>b.onclick=()=>answer(t,+b.dataset.ready,false,true));
  const b=$("#run");if(b)b.onclick=async()=>{b.disabled=true;const r=await fetch("/api/run/"+t.id,{method:"POST"}).then(r=>r.json());alertBox(r.out);load()};
}
function alertBox(msg){const d=document.createElement("div");d.textContent=msg;Object.assign(d.style,{position:"fixed",bottom:"16px",left:"22px",background:"#26342b",color:"#fff",padding:"10px 14px",borderRadius:"8px",fontSize:"13px",maxWidth:"520px"});document.body.append(d);setTimeout(()=>d.remove(),5000)}
function render(){
  $("#view").innerHTML=view==="board"?board():order();
  document.querySelectorAll("[data-id]").forEach(e=>e.onclick=()=>{if(typing()&&!confirm("Drop the answer you are writing?"))return;sel=e.dataset.id;render();detail()});
  const dt=$("#dt");if(dt)dt.onclick=()=>{showDone=true;render()};
  const sd=$("#sd");if(sd)sd.onchange=()=>{showDone=sd.checked;render()};
  const nx=$("#next");if(nx)nx.onclick=async()=>{nx.disabled=true;const r=await fetch("/api/next",{method:"POST"}).then(r=>r.json());alertBox(r.out);load()};
  $("#stamp").textContent=data.fetchedAt?"fetched "+new Date(data.fetchedAt).toLocaleTimeString():"";
  const edits=data.tasks.filter(t=>t.edited).length,pb=$("#publish");pb.hidden=!edits;pb.textContent="Publish "+edits;
  const L=data.loop||{},lim=L.limit&&L.limit.until>Date.now()?L.limit:null;
  $("#loop").innerHTML=lim?'<span class="chip tim">Claude limit · resumes '+new Date(lim.until).toLocaleTimeString([], {hour:"2-digit",minute:"2-digit"})+"</span>":L.running?'<span class="chip q">loop on</span>':'<span class="chip">loop off</span>';
}
const typing=()=>[...document.querySelectorAll("#detail textarea")].some(x=>x.value!==x.defaultValue||x===document.activeElement);
async function load(force){data=await fetch("/api/tasks"+(force?"?fetch":"")).then(r=>r.json());render();if(sel&&!typing())detail()}
document.querySelectorAll(".tabs button").forEach(b=>b.onclick=()=>{view=b.dataset.v;document.querySelectorAll(".tabs button").forEach(x=>x.setAttribute("aria-pressed",x===b));render()});
$("#refresh").onclick=()=>load(true);
$("#publish").onclick=async()=>{const b=$("#publish");b.disabled=true;const r=await fetch("/api/publish",{method:"POST"}).then(r=>r.json());b.disabled=false;alertBox(r.out);load(true)};
load();setInterval(load,10000);
</script></body></html>`;
