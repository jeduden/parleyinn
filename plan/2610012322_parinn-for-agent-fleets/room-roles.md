# Room capabilities and roles

The single source for the room's capabilities and roles; the
[concepts](concepts.md) and the [room security note](room-security.md)
include it.

| Capability           | Lets a participant                                                                                         |
| -------------------- | ---------------------------------------------------------------------------------------------------------- |
| read                 | Read the conversation, pins, branches, diffs, results and evidence                                         |
| post, link           | Post messages and links to marked ranges                                                                   |
| pin, unpin           | Pin information to the room, a claim of work among it, or withdraw a pin                                   |
| work                 | Work on a branch of the room it was given: edit, run commands, checkpoint, commit                          |
| branch               | Open a branch in the room for a new attempt, or close one                                                  |
| present              | Control the room's outcome window: what it shows (a dev server, an artifact, a file followed live, a diff) |
| kick, bar, read only | Moderate participants and messages                                                                         |

| Role                         | Always there     | Capabilities                                                                            | Filled by                                                     |
| ---------------------------- | ---------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| Viewer                       | no               | read                                                                                    | A player its person adds, or one an operator set to read only |
| Participant                  | yes              | read, post, link, pin and unpin its own pins; work on the branches it is given; present | A player its person adds                                      |
| Operator                     | yes              | a participant's, plus branch, unpin any pin but the intent, kick, bar, read only        | The owner, unless the owner assigns others                    |
| Etiquette or facilitator bot | no               | an operator's, plus posting findings against the pins                                   | A bot the owner provides                                      |
| Owner                        | beside the roles | everything, plus the intent, roles, successor and handover                              | The person who opened the room, until a handover              |
