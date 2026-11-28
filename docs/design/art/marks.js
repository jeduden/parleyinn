'use strict';
// Where each concept's number sits in the picture, and the part of the picture its entry zooms to: [centre x, centre y, width].
const order = ['node', 'boundary', 'peer', 'principal', 'keys', 'held', 'counter', 'home', 'harness', 'hook', 'agent', 'run', 'room', 'seat', 'writer', 'record', 'seal', 'landmark', 'pin', 'compaction', 'restore', 'recall', 'envelope', 'quarantine', 'purge', 'canary', 'forge', 'stake'];
const pos = {
  node: [231, 56], boundary: [458, 210], peer: [441, 240], principal: [348, 221], keys: [343, 117], held: [321, 172], counter: [329, 219], home: [361, 156],
  harness: [204, 262], hook: [86, 262], agent: [212, 193], run: [271, 225], room: [236, 224], seat: [199, 202], writer: [225, 204], record: [142, 158],
  seal: [390, 216], landmark: [166, 136], pin: [156, 211], compaction: [280, 247], restore: [233, 124], recall: [304, 219], envelope: [294, 133],
  quarantine: [46, 267], purge: [421, 282], canary: [189, 191], forge: [62, 166], stake: [345, 249],
};
const box = {
  node: [230, 150, 320], boundary: [452, 196, 120], peer: [414, 248, 120], principal: [346, 200, 96], keys: [332, 130, 96], held: [343, 182, 96], counter: [330, 204, 96], home: [352, 140, 96],
  harness: [226, 264, 160], hook: [128, 260, 120], agent: [214, 208, 96], run: [266, 212, 96], room: [236, 204, 160], seat: [212, 214, 96], writer: [232, 210, 96], record: [165, 137, 120],
  seal: [404, 214, 96], landmark: [166, 136, 96], pin: [156, 190, 96], compaction: [270, 262, 120], restore: [232, 158, 160], recall: [300, 196, 96], envelope: [290, 138, 96],
  quarantine: [46, 292, 120], purge: [430, 302, 120], canary: [189, 182, 96], forge: [42, 204, 120], stake: [340, 272, 96],
};
module.exports = { order, pos, box };
