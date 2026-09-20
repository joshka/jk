// Validate catalog coverage and links without running any proposed jj commands.
const fs = require('node:fs');
const vm = require('node:vm');
const path = require('node:path');
const root = __dirname;
const inventory = JSON.parse(fs.readFileSync(path.join(root, 'command-inventory.json'), 'utf8'));
const source = fs.readFileSync(path.join(root, 'screens.js'), 'utf8');
const data = vm.runInNewContext('const inventory = ' + JSON.stringify(inventory) + ';' + source +
  ';({screens,commandTargets,journeys})');
const ids = new Set(data.screens.map(s => s.id));
if (ids.size !== data.screens.length) throw Error('Duplicate screen IDs');
for (const s of data.screens) {
  for (const a of s.actions || []) if (!ids.has(a.to)) throw Error(`${s.id} -> ${a.to}`);
  if (s.base && !ids.has(s.base)) throw Error(`Missing base for ${s.id}`);
}
for (const sequence of Object.values(data.journeys)) {
  for (const id of sequence) if (!ids.has(id)) throw Error(`Missing journey screen ${id}`);
}
for (const cmd of inventory.commands) {
  if (!ids.has(data.commandTargets[cmd.command])) throw Error(`Unmapped command ${cmd.command}`);
}
const index = {
  version: inventory.version,
  screens: data.screens.map(({id,title,group,command}) => ({id,title,group,command})),
  commandTargets: data.commandTargets,
  journeys: data.journeys,
};
fs.writeFileSync(path.join(root, 'screen-index.json'), JSON.stringify(index, null, 2) + '\n');
console.log(`${ids.size} screens; ${inventory.commands.length} mapped command leaves; ` +
  `${Object.keys(data.journeys).length} journeys; all destinations resolve`);
