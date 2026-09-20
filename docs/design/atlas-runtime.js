const root=document.getElementById('jk-atlas');
const el=id=>root.querySelector('#jk-'+id);
const escape=s=>String(s).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const byId=new Map(screens.map(s=>[s.id,s]));
const preferences={appearance:'auto',selection:'subtle',density:'compact'};
let current='log',journey='',lastCommand=null;
let selectionOverride=null,markOverride=null,showBaseline=false;
const saved=window.openai?.widgetState;
function restore(state){const p=state?.privateContent;if(!p)return;if(byId.has(p.current))current=p.current;journey=journeys[p.journey]?p.journey:'';Object.assign(preferences,p.preferences||{});}
restore(saved);
function persist(){window.openai?.setWidgetState?.({modelContent:{screen:byId.get(current).title,command:byId.get(current).command,journey,appearance:preferences.appearance,selection:preferences.selection},privateContent:{current,journey,preferences}})?.catch(()=>{});}
function applyPrefs(){root.style.colorScheme=preferences.appearance==='auto'?'light dark':preferences.appearance;root.dataset.selection=preferences.selection;root.dataset.density=preferences.density;}
function color(text,query=''){
 let out=escape(text);
 // Fixture-only colorization approximates the jj palette in the baseline captures.
 out=out.replace(/\b(pvsouytl|qpvuntsm|mzrqpvnu|vtqsknru|zuktvkvo|ntwquvxs)\b/g,'<span class="id">$1</span>')
 .replace(/\b([a-f0-9]{8})\b/g,'<span class="hash">$1</span>')
 .replace(/\b(joshka)(?= |$)/g,'<span class="author">$1</span>')
 .replace(/(2026-\d\d-\d\d(?: \d\d:\d\d:\d\d)?)/g,'<span class="date">$1</span>')
 .replace(/\b(main|review-ui|default@)\b/g,'<span class="ref">$1</span>');
 if(query){const safe=escape(query).replace(/[.*+?^${}()|[\]\\]/g,'\\$&');out=out.replace(new RegExp(`(${safe})`,'gi'),'<mark>$1</mark>');}
 return out;
}
function rows(s,context=false){return '<div class="rows">'+s.lines.map((line,i)=>{
 const selected=!context&&(selectionOverride??s.selected)===i;
 const marks=markOverride||s.marks||{};
 const gutter=(selected?'›':' ')+(marks[i]||' ')+' ';
 const kind=s.kinds?.[i]||(line.startsWith('+')?'add':line.startsWith('-')?'del':line.startsWith('Error:')?'del':'');
 return `<div class="line ${selected?'selected':''} ${kind}" data-row="${i}"><span class="gutter">${escape(gutter)}</span><span class="line-content">${color(line,s.query)}</span></div>`;
}).join('')+'</div>';}
function goto(id){if(!byId.has(id))throw new Error('Unknown screen '+id);const s=byId.get(current);if(id==='run-options'&&s.id!=='run-options'){const options=byId.get(id);options.command=s.command;options.actions=[btn('Enter','Apply options',s.id),btn('Esc','Cancel',s.id)];}if(s.command&&s.mode!=='result')lastCommand=s;current=id;selectionOverride=null;markOverride=null;render();persist();}
function fillSelect(){const groups=[...new Set(screens.map(s=>s.group))].sort();el('screen').innerHTML=groups.map(g=>`<optgroup label="${escape(g)}">${screens.filter(s=>s.group===g).map(s=>`<option value="${s.id}">${escape(s.title)}</option>`).join('')}</optgroup>`).join('');
 for(const j of Object.keys(journeys)){const opt=document.createElement('option');opt.value=j;opt.textContent=j;el('journey').append(opt);}}
function catalog(){return `<label class="input-label">Find a jj command<input id="jk-command-search" placeholder="rebase, bookmark, sparse…" autocomplete="off"></label><div class="catalog-list" id="jk-catalog"></div>`;}
function filterCatalog(query=''){const items=inventory.commands.filter(i=>(i.command+' '+i.help.split('\n')[0]).toLowerCase().includes(query.toLowerCase()));el('catalog').innerHTML=items.map(i=>`<button type="button" class="cursor-interaction" data-to="${commandTargets[i.command]}"><span>${escape(i.command)}</span><span class="desc">${escape(i.help.split('\n')[0])}</span></button>`).join('')||'<div>No matching commands.</div>';el('status').textContent=`${items.length} commands · installed ${inventory.version} · choose to inspect or preview`;}
function render(){applyPrefs();const s=byId.get(current);showBaseline=false;el('baseline').hidden=true;el('terminal').hidden=false;el('baseline-toggle').hidden=!baselines[current];el('baseline-toggle').textContent='Show existing capture';el('screen').value=current;el('journey').value=journey;el('screen-title').textContent=s.title;el('count').textContent=`${screens.indexOf(s)+1} / ${screens.length} screens · ${inventory.commands.length} command leaves`;
 el('terminal').classList.toggle('compact',!!s.compact);el('scope').textContent='/work/jk · '+(s.scope||'default');const header=s.modal?byId.get(s.base||'log'):s;el('command').textContent=header.command?'jj '+header.command:header.title;
 el('note').textContent=s.note||'Shared focus, marks, command preview, and return behavior.';
 const steps=journeys[journey]||[];el('steps').innerHTML=steps.map((id,i)=>`<button type="button" class="cursor-interaction" data-to="${id}" ${current===id?'aria-current="step"':''}>${i+1} ${escape(byId.get(id).title.split(' · ')[0])}</button>`).join('<span aria-hidden="true">→</span>');
 let body='';
 if(s.mode==='catalog')body=catalog();
 else if(s.mode==='result'){
 const source=lastCommand||byId.get('cmd-version');el('command').textContent='jj '+source.command;
 body=rows({lines:['Command result · illustrative','', 'Command    jj '+source.command, 'Outcome    completed · exit 0','',source.read?'Output remains selectable and can be copied.':'Repository refreshed; inspect operation history for recorded effects.','','Return restores the originating scope and selection.']});
 }else{
  body=rows(s);
  if(s.input)body=`<label class="input-label">${escape(s.input.label)}<input id="jk-field" value="${escape(s.input.value)}" aria-label="${escape(s.input.label)}"></label>`+body;
  if(s.modal){const base=byId.get(s.base||'log');body=`<div class="context-graph" aria-hidden="true">${rows(base,true)}</div><section class="overlay" aria-label="${escape(s.title)}"><h3>${escape(s.title)}</h3>${body}${s.command&&!s.id.startsWith('help')&&!['files','view-options','log-options','template-picker','actions','file-actions','revset','describe-form','commit-form'].includes(s.id)?`<div class="preview-command"><span class="muted">${s.mode==='preview'?'Command to run':'Command context'}</span>jj ${escape(s.command)}</div>`:''}</section>`;}
  if(s.help)body+=`<details class="help-details"><summary>Installed jj help · arguments and all options</summary><pre>${escape(s.help)}</pre></details>`;
 }
 el('body').innerHTML=body;el('body').scrollTop=0;el('body').scrollLeft=0;
 el('status').textContent=s.status||((s.selected!==undefined)?`${s.lines.length} rendered lines · selected object in gutter`:s.modal?'Esc returns to the previous context.':'');
 const actions=s.mode==='result'?[btn('Esc','Return',lastCommand?.id||'log'),btn('C','History','history')]:(s.actions||[btn('Esc','Back','log')]);el('hotbar').innerHTML=actions.map(a=>`<button type="button" class="cursor-interaction" data-to="${a.to}"><kbd>${escape(a.key)}</kbd>${escape(a.label)}</button>`).join('');
 const overlay=el('body').querySelector('.overlay');if(overlay){const status=el('status').textContent;const updateScroll=()=>{el('status').textContent=overlay.scrollHeight>overlay.clientHeight+1?((overlay.scrollTop>0?'↑ more above · ':'')+(overlay.scrollTop+overlay.clientHeight<overlay.scrollHeight-2?'↓ more below · ':'')+'Esc returns'):status;};overlay.addEventListener('scroll',updateScroll);updateScroll();}
 if(s.mode==='catalog'){filterCatalog();el('command-search').addEventListener('input',e=>filterCatalog(e.target.value));}
 if(s.input){el('field').addEventListener('input',e=>{if(current==='search'){const temp={...s,query:e.target.value};const box=el('body').querySelector('.rows');box.outerHTML=rows(temp);}else el('status').textContent='Draft edited · choose Review to inspect the illustrative next step.';});}
}
fillSelect();render();
root.addEventListener('click',e=>{const b=e.target.closest('[data-to]');if(b)goto(b.dataset.to);});
el('screen').addEventListener('change',e=>goto(e.target.value));
el('journey').addEventListener('change',e=>{journey=e.target.value;if(journey)goto(journeys[journey][0]);else{render();persist();}});
function step(delta){const list=journeys[journey]||screens.map(s=>s.id);let index=list.indexOf(current);index=(index+delta+list.length)%list.length;goto(list[index]);}
el('baseline-toggle').addEventListener('click',()=>{const b=baselines[current];if(!b)return;showBaseline=!showBaseline;el('baseline').hidden=!showBaseline;el('terminal').hidden=showBaseline;el('baseline-toggle').textContent=showBaseline?'Show proposed design':'Show existing capture';el('baseline').innerHTML=`<img style="display:block;width:100%;height:auto" src="${b.data}" alt="Existing capture ${escape(b.name)}">`;});
el('prev').addEventListener('click',()=>step(-1));el('next').addEventListener('click',()=>step(1));
el('terminal').addEventListener('keydown',e=>{if(e.target.matches('input,textarea,select'))return;const s=byId.get(current);if(['j','k','ArrowDown','ArrowUp'].includes(e.key)&&s.selected!==undefined){e.preventDefault();const delta=['j','ArrowDown'].includes(e.key)?1:-1;const base=selectionOverride??s.selected;const nodes=s.lines.map((l,i)=>/^[│ ]*[○@◆]/.test(l)?i:-1).filter(i=>i>=0);const choices=nodes.length?nodes:s.lines.map((_,i)=>i);const index=choices.indexOf(base);selectionOverride=choices[Math.max(0,Math.min(choices.length-1,index+delta))];render();return;}
 if(e.key===' '&&s.selected!==undefined){e.preventDefault();const row=selectionOverride??s.selected;markOverride={...(markOverride||s.marks||{})};if(markOverride[row])delete markOverride[row];else markOverride[row]='✓';render();return;}
 const key=e.key==='Escape'?'Esc':e.key;const action=(s.actions||[]).find(a=>a.key===key);if(action){e.preventDefault();goto(action.to);}else if(key===':'||key==='?'){e.preventDefault();if(key===':')goto('command');else{const h=byId.get('help-current');h.title='Help · '+s.title;h.command=s.command;h.base=s.id;h.lines=['Available actions','',...(s.actions||[]).map(a=>'  '+a.key.padEnd(16)+a.label)];h.actions=[btn('Esc','Close help',s.id)];goto('help-current');}}});
window.addEventListener('openai:set_globals',e=>{restore(e.detail?.globals?.widgetState);render();});
if(globalThis.Tweak){const t=new Tweak({container:root,onChange:()=>{applyPrefs();persist();}});t.addSelect(preferences,'appearance',{label:'Terminal appearance',options:['auto','dark','light']});t.addSelect(preferences,'selection',{label:'Selection treatment',options:[{value:'subtle',label:'Quiet row + gutter'},{value:'marker',label:'Gutter only'}]});t.addSelect(preferences,'density',{label:'Row spacing',options:['compact','roomy']});}
// Expose only local design metadata for deterministic visual checks.
root.atlas={screens: screens.map(s=>({id:s.id,title:s.title,command:s.command,group:s.group})),commandTargets,goto,preferences,render};
