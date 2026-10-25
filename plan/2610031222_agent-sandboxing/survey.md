# Sandbox survey, initial

What is known on 3 October 2026, before task 1 extends it. Each entry
says what the technology confines, which residual risks of the
[plan](plan.md) it can address, and where ParleyInn's record writer would
sit.

## Named by the stakeholder

- **NVIDIA OpenShell** (Apache-2.0, open-sourced February 2026): a
  runtime that runs coding agents (Claude Code, Codex, OpenCode,
  Copilot) inside a sandbox governed by declarative YAML policy over
  four domains: filesystem, network, process and inference. Sources
  describe both kernel enforcement (Landlock for files, seccomp for
  syscalls) and per-sandbox containers with policy-enforced egress,
  run from a gateway; credentials are injected without reaching the
  sandbox's filesystem. A whole-harness sandbox: the harness and its
  hooks run inside, so the record writer's place is open. Candidate
  coverage: R2, R3, R5, R6 through file and process policy; R1 and R4
  when the owner's terminal and display stay outside.
- **Deno Claw Patrol** (MIT, May 2026): an egress firewall between an
  agent and the systems it reaches. It parses traffic at the wire level
  (SQL verbs, Kubernetes verbs, HTTP method and path), evaluates each
  action against HCL rules, can inject credentials, and can escalate
  to a human or an LLM approver; it deploys over WireGuard or
  Tailscale. It confines network actions, not files or processes, so
  it complements a sandbox rather than replacing one. Its decisions
  and approvals are candidate untrusted events for the lane (task 7).

## Also in scope for task 1

- Tool sandboxes: Claude Code's own sandbox, Codex's sandbox.
- Containers and microVMs: devcontainers, Docker, Podman, gVisor,
  Firecracker, Kata.
- Kernel primitives: Landlock, seccomp, bubblewrap, nsjail, macOS
  sandbox-exec.
- A separate OS user for the agent.
- Cloud sandboxes, where the whole machine is the sandbox.

## Sources

- NVIDIA OpenShell:
  [project page](https://landscape.jimmysong.io/projects/openshell/),
  [NVIDIA perspectives](https://perspectives.nvidia.com/nvidia-openshell/)
- [Deno: Claw Patrol](https://deno.com/blog/clawpatrol)
- [Claw Patrol on GitHub](https://github.com/denoland/clawpatrol)
