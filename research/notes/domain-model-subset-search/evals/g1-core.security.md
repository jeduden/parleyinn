```json
[
  {
    "type": "security",
    "quote": "or any CLI or TUI read from inside a run, whatever its output",
    "problem": "Only reads count as recall. The room-act MCP tools (room_pin, room_edit, room_unpin, room_link, room_create) and CLI calls from inside a run that are not reads (parinn ingest, rebuild, migrate, a pin command) also return output that the harness sends to the model. That output is neither enveloped recall nor a closed path, and nothing bounds it: ingest could echo an attacker's unparsed line, or room_edit could echo pin text. This breaks I2's 'No other write to an agent exists' and I4's list of what leaves the machine.",
    "fix": "Write 'or any CLI or TUI call from inside a run, and any MCP tool's result, whatever its output', or say that every other agent-facing output carries only fixed text ParleyInn ships, structural fields and ids.",
    "severity": "needs-fix"
  },
  {
    "type": "gap",
    "quote": "ends it: the seat's later acts and seals count for nothing, and a device",
    "problem": "Revoking a device seat's key is an allowed widening act, but a new seat is minted only on a node identity change, a node clone or a lost run-seat key. Once the node's own current device seat is revoked, no device seat remains whose seal makes a principal act count. No cut act (quarantine, turning capture on) and no neutral act (acknowledging counters) can follow on that node, so I5 and I6 stop holding there.",
    "fix": "Add: 'Revoking this node's own device seat's key mints a new device seat, certified by its device key, that names the revoked seat but does not take it as a predecessor.'",
    "severity": "needs-fix"
  },
  {
    "type": "security",
    "quote": "a **qualifying pin**, its creating event trusted on this node: a pin",
    "problem": "Qualification checks only the creating event, and revocation stops only 'a device seat's pins', meaning those it authored first. Yet the principal can edit any device-seat pin from any of its device seats. Example: on a node clone, a leaked device-seat key is used to edit a constraint pin carried over from the predecessor. After the principal revokes that key, the pin's author is still unrevoked, so the attacker's version keeps restoring as trusted text.",
    "fix": "Write: 'a pin qualifies only while neither its creating event nor the version that restores was written by a seat whose key is revoked', and in Seat key: 'a device seat's pins and the pin versions it wrote stop qualifying'.",
    "severity": "needs-fix"
  },
  {
    "type": "contradiction",
    "quote": "A seat certified by this node's device key, or by the device key of the node",
    "problem": "This rule gives this node's principal only the seats certified by its own device key or its direct parent's. Pin and Trusted sources, though, trust what a device seat's predecessors wrote 'however far back'. On a clone of a clone, the grandparent's device-seat pins qualify, but their author belongs to no principal of this node. I8 then treats them as another principal's untrusted content, and this principal cannot edit or unpin them.",
    "fix": "Write 'or by the device key of any node in the chain its home was cloned from'.",
    "severity": "minor"
  }
]
```

FINDINGS: 3
