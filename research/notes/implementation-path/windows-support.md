# Windows support: cost per language and slot

Scope: what it costs, as of 4 October 2026, to make a Windows desktop
a full ParleyInn node in Go and in Rust. Such a node holds the record,
receives Claude Code's hooks and MCP calls, hosts harness terminals in
the run component and serves the room UI. It answers the stakeholder's
new requirement against [CON-02][ctx] and [NFR-10][nfr], which name
linux/amd64, linux/arm64 and darwin/arm64 only, and against the hard
constraints in [the evaluation frame][frame], above all HC-1, HC-3,
HC-11, HC-15, HC-16 and HC-18. Sources are vendor documentation,
release notes, issue trackers, and module and crate sources fetched on
the date above (Go module proxy, crates.io, docs.rs). A claim no
primary source confirmed is marked "unverified"; an inference drawn
from sources is marked "inference"; effort figures are estimates. The
phones (iOS, Android) are clients and get one short section. The note
recommends no option.

## What a Windows node must do

- Run the core (B0) as the hook and MCP binary Claude Code starts, the
  CLI and the TUI, with no socket and no program start but its kernel
  worker ([SEC-01, SEC-10][sec], [CON-04][ctx]).
- Run the lane-view or room server (B1) on loopback, and the run
  component (B1) hosting harness terminals ([SEC-20, SEC-29][sec],
  [OWN-19][own]).
- Keep the store's guarantees: `0700` and `0600`, refusing a home
  another user owns, atomic payload writes and crash consistency
  ([SEC-02][sec], [REC-09][fr], [NFR-07][nfr]).
- Ship as a reproducible, signed artifact through the Claude Code
  plugin ([ENG-19, ENG-20][eng], [ADM-01][fr]).

## 1. Claude Code on Windows

Facts:

- Claude Code runs natively on Windows 10 1809+ or Server 2019+, x64 or
  ARM64, or inside WSL. "Sandboxing" is "Not supported" natively and
  on WSL 1, "Supported" on WSL 2 ([setup][cc-setup]). The sandbox
  page says it plainly: "On native Windows, Claude Code runs commands
  unsandboxed" ([sandboxing][cc-sandbox]).
- Git for Windows is optional. Without it, Claude Code uses PowerShell
  for shell commands; with it, Git Bash ([setup][cc-setup]).
- Hooks have two forms. Shell form (no `args`) runs the string through
  "`sh -c` on macOS and Linux, Git Bash on Windows, or PowerShell when
  Git Bash isn't installed". Exec form (`args` present) "resolves
  `command` as an executable on `PATH` and spawns it directly", with
  `${CLAUDE_PLUGIN_ROOT}` substituted as a plain string. "On Windows,
  exec form requires `command` to resolve to a real executable such as
  a `.exe`"; `.cmd` and `.bat` shims fail ([hooks][cc-hooks]).
- On Windows, `${CLAUDE_PLUGIN_ROOT}` arrives with forward slashes
  ([plugin reference][cc-plugref]), while `tool_input.file_path`
  arrives with backslashes ([hooks][cc-hooks]). `~/.claude` means
  `%USERPROFILE%\.claude` ([settings][cc-settings]).
- The spawn error in [#73971][cc-73971] reads `EFTYPE … uv_spawn`, so
  hook processes go through libuv (inference). libuv's Windows search
  appends `.com`, then `.exe`, to a name without an extension
  ([libuv process.c][libuv-search]). Whether exec form therefore finds
  `parinn.exe` from `${CLAUDE_PLUGIN_ROOT}/bin/parinn` is unverified.
- MCP stdio servers: the MCP page has no Windows section today
  ([MCP][cc-mcp]). Community guides wrap `npx` in `cmd /c`
  ([guide][mcp-win-guide], secondary); a native `.exe` needs no
  wrapper (inference from the hooks rule above).
- Plugin distribution: a marketplace entry has no platform or OS field
  (none in the [marketplace reference][cc-mkt]), so one plugin must
  carry every target's binary or each target gets its own plugin.
  Files in `bin/` join the Bash tool's `PATH`, and "claude.ai and
  Cowork don't install a plugin that has this directory"
  ([plugin reference][cc-plugref]). Adding a git marketplace on Windows
  needs Git for Windows; Claude Code "refuses to install a link-mode
  plugin on Windows"; and while another program holds the installed
  copy, an update reports that it "could not be replaced"
  ([troubleshooting][cc-plugts], [marketplace reference][cc-mkt]).
- Managed settings on Windows live in `C:\Program Files\ClaudeCode\`
  and under `HKLM\SOFTWARE\Policies\ClaudeCode`; HKCU is user-writable
  and not an admin source; the legacy `C:\ProgramData` path is no
  longer read ([managed settings][cc-managed]).
- Known issues, each now closed as a duplicate or as not planned:
  - [#16602][cc-16602] (January 2026): hooks ran through `cmd.exe`
    unless `CLAUDE_CODE_GIT_BASH_PATH` was set; closed as duplicate.
  - [#24097][cc-24097] (February 2026): plugin `.sh` hooks opened a
    file-association dialog instead of running; closed as duplicate.
  - [#25577][cc-25577] (February 2026): user hooks did not fire with
    the native `claude.exe`; closed as not planned.
  - [#73971][cc-73971] (July 2026, v2.1.197): every command hook
    failed because shell resolution spawned the Git `bin` directory
    as an executable; closed as not planned.
  - [#32951][cc-32951] (March 2026, v2.1.72): local stdio MCP servers
    were silently absent on native Windows; closed as not planned.
- Microsoft Defender scans synchronously on open outside a trusted Dev
  Drive: "Open now, scan now" ([Defender performance mode][def-perf]).
  How much that adds to a hook's process start and store opens is
  unmeasured.
- WSL 2 is the cheap path: a Linux `parinn` inside WSL, and a Windows
  browser reaching its loopback server, since "you can access it from
  a Windows app … using `localhost`" ([WSL networking][wsl-net]). It
  does not serve native-Windows Claude Code.

**Go.** `GOOS=windows CGO_ENABLED=0` yields a `.exe` from the Linux
builder, and exec-form hooks need no shell. No language-specific risk.

**Rust.** The same `.exe` from the MSVC target, built on Windows or
cross-built with cargo-xwin (item 8). No language-specific risk.

**Maturity.** Native Windows hooks have a record of regressions in
2026, most in the shell path; exec form avoids the shell, though
whether it avoided each bug is unverified. The open questions are the
same in both languages:

- How one plugin picks the binary for linux, darwin and windows
  without a shell or a second process start in the hook path (an
  existing linux-versus-darwin question that Windows widens).
- The hook budgets with Defender scanning, measured on Windows.
- Whether a hook process shares Claude Code's console on Windows. The
  hooks page says only that "Windows has no `/dev/tty`"
  ([hooks][cc-hooks]); this decides OWN-12's terminal test there.

**SRS rows.** [ADM-01, ADM-02, ADM-03][fr], [NFR-01, NFR-12][nfr],
[ASM-03, ASM-07, ASM-16][ctx], [OWN-12, OWN-22][own],
[ENG-14][eng]. Every native-Windows session has no Claude Code
sandbox, so under OWN-22 it counts as unsandboxed and every widening
act on that node needs the owner's recorded risk acceptance.

## 2. Pty hosting

Facts:

- ConPTY (`CreatePseudoConsole` in `kernel32.dll`) needs Windows 10
  1809 or Server 2019; streams are "UTF-8 … interleaved with Virtual
  Terminal Sequences" and the only documented flag is
  `PSEUDOCONSOLE_INHERIT_CURSOR` ([CreatePseudoConsole][ms-cpc]).
- ConPTY is not a byte pipe. ConHost "renders changes in its Output
  Buffer as UTF-8 encoded text/VT", through a "VT Renderer"
  ([ConPTY announcement][ms-conpty-blog]). The run component's VT model
  sees ConHost's re-rendering, not the harness's own bytes.
- portable-pty prefers "a sideloaded conpty.dll and openconsole.exe
  host deployed alongside the application", falls back to the kernel,
  and passes `INHERIT_CURSOR | RESIZE_QUIRK | WIN32_INPUT_MODE`; a
  passthrough flag is defined but unused ([portable-pty source][ppty]).
  Sideloading would add a Microsoft binary and a program start to the
  run component's register row (inference).
- No-echo detection: on Unix the host reads the pty's echo flag; WezTerm
  says its password detection "only works for local processes on unix
  systems" ([WezTerm][wez-pw]). A ConPTY host query for the child's
  console echo mode was not found (unverified). OWN-19's "input typed
  at a no-echo prompt MUST NOT be stored" may need a "cannot tell"
  state on Windows.

**Go.** `os/exec` cannot attach a pseudoconsole: the proposal to add
it has been open since September 2023 ([golang/go#62708][go-62708]).
So every Go library calls `windows.CreateProcess` itself:

- charmbracelet/x/conpty v0.2.0 (17 Nov 2025), MIT, depends on
  `golang.org/x/sys` only ([x/conpty][xconpty]).
- aymanbagabas/go-pty v0.2.3 (17 May 2026), MIT, Unix and ConPTY behind
  one API, but pulls creack/pty, u-root and `x/crypto`
  ([go-pty][gopty], [go.mod][gopty-mod]).
- UserExistsError/conpty v0.1.4 (9 Jul 2024), MIT, `x/sys` only
  ([conpty.go][uec-conpty]).
- `x/sys/windows` v0.48.0 exposes `CreatePseudoConsole` directly
  ([syscall_windows.go][xsys-cpc]), so an in-house host is feasible.

The cost lands on HC-4: program start on Windows is
`windows.CreateProcess` in `x/sys`, not `os/exec`, so the evidence must
ban symbols, not packages (item "What Windows changes" below).

**Rust.** portable-pty 0.9.0 (11 Feb 2025), MIT, used and patched by
herdr; `conpty` 0.7.0 (23 Sep 2024), MIT ([crates.io][conpty-crate]);
alacritty_terminal has a ConPTY back end ([terminal note][term]). Rust's
`std::process::Command` cannot attach a pseudoconsole either, so these
crates call `CreateProcessW` (inference from portable-pty's design).

**libghostty-vt on Windows.** "Windows is already a supported target"
for libghostty, while for the Ghostty app "the earliest Windows is
planned to be looked at is late this year", with no commitment
([discussion #12290][ghostty-12290], April 2026). herdr maps
`x86_64-` and `aarch64-pc-windows-msvc` ([herdr build.rs][herdr-build]).
A native Windows build needs an installed MSVC and Windows SDK on a
Windows host, and "embeds the building machine's absolute paths in the
archive's member table", so ghosttea-vt-sys declares its Windows
releases unverifiable for reproducibility ([ghosttea-vt-sys][ghosttea]).
The WASM build is one file for every OS, so the Go path through
wasm2go or wazero ([terminal note][term]) is unaffected by Windows.

**Maturity.** ConPTY is stable API since 2018. The Go libraries are
small and young; portable-pty is the most used. The no-echo gap and
ConPTY's re-rendering apply to both languages.

**SRS rows.** [OWN-03, OWN-19][own], [SEC-29][sec], [NFR-09][nfr]
(keystroke-to-echo ≤ 10 ms through one more hop, unmeasured),
[ENG-16, ENG-19][eng], T21.

## 3. SQLite with FTS5

**Go.** ncruces/go-sqlite3 v0.35.6 (23 Sep 2026) is pure Go through
wasm2go and "every commit is tested on … Windows: amd64, arm64"
([README][gosq-readme]). On Windows its VFS "uses `LockFileEx` and
`UnlockFileEx`, like SQLite" and maps the WAL index with
`MapViewOfFile` ([VFS README][gosq-vfs]); the support matrix shows
full locking and shared-memory WAL on windows/amd64 and arm64
([support matrix][gosq-matrix]). It cross-builds from Linux with no C
toolchain. FTS5 and the ADR-07 measurements carry over unchanged
(ADR-2609302341).

**Rust.** rusqlite 0.40.2 (8 Aug 2026), MIT, with `bundled` compiles
SQLite through the `cc` crate and always sets `-DSQLITE_ENABLE_FTS5`;
it honours `crt-static` on MSVC ([libsqlite3-sys build.rs][rsq-build]).
On Windows that is a C compile with MSVC `cl.exe`, or clang-cl under
cargo-xwin, inside the reproducible pipeline (item 8).

**Store semantics that change on Windows.**

- Go: "Even within the same directory, on non-Unix platforms Rename is
  not an atomic operation" ([os.Rename][go-rename]). REC-09's "write,
  fsync, rename" needs a Windows-specific replace (inference).
- Rust: `fs::rename` "on Windows 10 version 1607 and above" behaves as
  on Unix "if the filesystem supports `FileRenameInfoEx`"
  ([fs::rename][rs-rename]).
- Both: Defender's synchronous scan on open applies to store and
  payload files ([Defender performance mode][def-perf]); NFR-04's
  5,000 events/s and NFR-03's latencies need a Windows measurement.

**Maturity.** High in both; the driver question is unchanged.

**SRS rows.** [REC-09, REC-11][fr], [NFR-03, NFR-04, NFR-07][nfr],
[SEC-14][sec], ADR-07, OQ-03.

## 4. File permissions and ownership

Facts:

- Mode bits do not exist. In Go, "On Windows, only the 0o200 bit (owner
  writable) of mode is used; it controls whether the file's read-only
  attribute is set" ([os.Chmod][go-chmod]). In Rust, `Permissions` on
  Windows is `FILE_ATTRIBUTE_READONLY` and "does not take Access
  Control Lists (ACLs) … into account" ([Permissions][rs-perm]).
- SEC-02's `0700`, `0600` and "owned by the running UID" therefore
  become: the owner SID equals the process token's user SID; the DACL
  is protected from inheritance; and it allows only that SID, plus
  whichever of SYSTEM and Administrators the SRS accepts as Windows'
  root equivalent (a ruling, inference).
- Owner of objects an administrator creates: Microsoft says UAC
  "will ensure the user account is being used as owner for all objects
  created locally" ([KB 947721][kb947721]). Whether an elevated
  process records the Administrators group instead is unverified, so
  a spike tests both cases.
- Windows has no umask; a new file inherits its parent's ACL unless it
  is created with an explicit security descriptor (inference). Go's
  `os.OpenFile` takes no descriptor, so store files are created through
  `windows.CreateFile` (inference).
- Path confinement: since Go 1.23, "On Windows, EvalSymlinks no longer
  evaluates mount points" ([Go 1.23][go123]), so a junction inside a
  transcript root is not resolved by it; SEC-18's check must treat
  reparse points itself (inference). `os.Root` refuses reserved device
  names such as `NUL` on Windows ([os.Root][go-root]).
- Test isolation: on Windows `os.UserHomeDir` "returns %USERPROFILE%"
  ([os.UserHomeDir][go-home]), and Claude Code's `~/.claude` is under
  it ([settings][cc-settings]). ENG-14's guard must redirect
  `USERPROFILE`, not only `HOME`.
- Managed policy path: `C:\ProgramData` has permissive DACLs "so that
  every user can access directories there freely"
  ([CyberArk][cyberark-pd], secondary); Claude Code's choice of
  `C:\Program Files\ClaudeCode\` and HKLM ([managed settings][cc-managed])
  is the model for ADM-04's "system path … the tenant cannot write",
  checked by ACL rather than mode bits (inference).

**Go.** `x/sys/windows` has what the check needs without cgo:
`GetNamedSecurityInfo`, `SECURITY_DESCRIPTOR.Owner`,
`Token.GetTokenUser`, `ACLFromEntries` and `SecurityDescriptorFromString`
for SDDL ([security_windows.go][xsys-sec]). Registry reads for HKLM
policy come from `x/sys/windows/registry` (same module).

**Rust.** `windows-sys` 0.61.2 (MIT or Apache-2.0, Microsoft) has the
same calls under `Win32_Security_Authorization`, as `unsafe` FFI
([Cargo.toml][ws-cargo]). The convenience crate windows-acl 0.3.0 was
last released in January 2021 ([crates.io][winacl]).

**Maturity.** The Win32 security API is old and stable; the ParleyInn code
is new in both languages, and the SRS wording is POSIX-only.

**SRS rows.** [SEC-02, SEC-03, SEC-18][sec], [§8.1 home layout][data],
[ADM-04][fr], [ENG-14][eng].

## 5. Key store

Facts:

- DPAPI: `CryptProtectData` in `Crypt32.dll`; "only a user with the
  same logon credential as the user who encrypted the data can decrypt
  the data" ([CryptProtectData][dpapi]). The calls "make a local RPC
  call to the Local Security Authority", and "all applications running
  under the same user can access any protected data that they know
  about" ([Windows Data Protection][dpapi-2001]).
- Credential Manager: `CredWriteW` in `Advapi32.dll` writes to "the
  user's credential set", tied to the logon session
  ([CredWriteW][credwrite]). Any process of the same user can read it
  back (inference; wincred below does exactly that).
- CNG: the Microsoft Software Key Storage Provider signs with ECDSA on
  P-256, P-384, P-521, DSA and RSA; the Platform Crypto Provider keeps
  TPM keys that "cannot be extracted" ([key storage providers][ksp]).
  Ed25519 is in neither list. `curve25519` appears among named curves
  for the ECDSA or ECDH algorithm ids ([named curves][curves]), not as
  EdDSA.
- So on Windows an Ed25519 writer key (REC-18) can be wrapped by DPAPI
  but not held non-exportably; a TPM-held key means P-256, the same
  split [the data note][pdc] found on iPhones (inference).
- "No socket and no program start" (SEC-10): DPAPI and Credential
  Manager reach LSA by local RPC, not a Winsock socket. Whether that
  passes SEC-10 needs a ruling, like the Keychain's daemon on macOS
  (inference).
- Against a same-user agent, a DPAPI blob is no stronger than a `0600`
  file, so `parinn status` must say so on Windows too (SEC-10).

**Go.** `x/sys/windows` binds `CryptProtectData` and
`CryptUnprotectData` ([syscall_windows.go][xsys-dpapi]). Credential
Manager and NCrypt are not in `x/sys`, but `windows.NewLazySystemDLL`
reaches them without cgo, as danieljoos/wincred v1.2.3 (2 Oct 2025,
MIT) does for `CredReadW` and `CredWriteW` ([wincred][wincred]). Unlike
the macOS Keychain, which needs cgo ([frame, HC-16][frame]), Windows
costs Go no foreign-function bridge.

**Rust.** `windows-sys` has DPAPI and NCrypt under
`Win32_Security_Cryptography` and `CredWriteW` under
`Win32_Security_Credentials` ([Cargo.toml][ws-cargo]); keyring 4.2.0
(29 Aug 2026, MIT or Apache-2.0) reaches Windows through an optional
`windows-native-keyring-store` ([keyring][keyring]).

**Maturity.** DPAPI dates from Windows 2000 and is stable; the
`CRYPTPROTECT_PROMPTSTRUCT` prompt flow "will be removed in February
2027" ([CryptProtectData][dpapi]), which ParleyInn would not use.

**SRS rows.** [SEC-10, SEC-09, SEC-27][sec], [REC-18, REC-24][fr],
PRV-10, VIEW-10.

## 6. Local endpoints

Facts:

- AF_UNIX arrived in Windows 10 build 17063: `SOCK_STREAM` only, no
  `SCM_RIGHTS` or `SCM_CREDENTIALS`, socket files as NTFS reparse
  points, access governed by the socket path's permissions
  ([AF_UNIX comes to Windows][afunix]). The peer's PID comes from the
  `SIO_AF_UNIX_GETPEERPID` ioctl, defined in `afunix.h`
  ([mingw-w64 header][mingw-afunix]); no Microsoft page for it was
  found (unverified). PID to process to token to user SID is a
  peer-UID check with a PID-reuse window (inference).
- Named pipes: a `NULL` descriptor grants "read access to members of
  the Everyone group and the anonymous account", and the default,
  `PIPE_ACCEPT_REMOTE_CLIENTS`, accepts "connections from remote
  clients" ([CreateNamedPipe][cnp]). A ParleyInn pipe must set
  `PIPE_REJECT_REMOTE_CLIENTS` and an explicit descriptor, or B1 leaks
  onto the network (inference against I4).
- `GetNamedPipeClientProcessId` gives the client PID (Vista+)
  ([GetNamedPipeClientProcessId][gnpcpid]); impersonating the client
  yields its token directly, without the PID race (inference).
- Loopback TCP for the lane view works as on Unix. Whether binding to
  `127.0.0.1` raises a Windows Firewall prompt is unverified.

**Go.** `net` has supported AF_UNIX "for compatible versions of
Windows" since Go 1.12 ([Go 1.12][go112]), and since Go 1.23 `Stat`
sets `ModeSocket` for such files ([Go 1.23][go123]). The peer PID needs
`WSAIoctl` from `x/sys/windows` on the raw socket (in-house). For
pipes, `x/sys/windows` has `CreateNamedPipe`,
`PIPE_REJECT_REMOTE_CLIENTS` and `GetNamedPipeClientProcessId`
([syscall_windows.go][xsys-pipe]); microsoft/go-winio v0.6.2 (April
2024, MIT) always sets `FILE_PIPE_REJECT_REMOTE_CLIENTS` and takes an
SDDL descriptor ([pipe.go][winio-pipe]), as one more dependency.

**Rust.** `std::os::windows::net` with `UnixStream` and `UnixListener`
exists behind the unstable `windows_unix_domain_sockets` feature
([rust#147335][rs-147335]); uds_windows 1.2.1 (MIT) fills the gap on
stable ([crates.io][uds-win]). tokio's named-pipe server rejects remote
clients by default and takes a raw `SECURITY_ATTRIBUTES` pointer, with
no client-PID call ([tokio ServerOptions][tokio-np]);
`GetNamedPipeClientProcessId` and `ImpersonateNamedPipeClient` are in
`windows-sys` under `Win32_System_Pipes` ([Cargo.toml][ws-cargo]).

**Maturity.** Named pipes are the mature Windows IPC; AF_UNIX on
Windows is younger, with a peer-PID ioctl documented only in headers.

**SEC-20 note.** Its "environment another UID can read" holds if other
users cannot open the process; that default is unverified here.

**SRS rows.** [SEC-20, SEC-29, SEC-22][sec], [ENG-12][eng], I4, HC-5.

## 7. Sandboxing for ENG-12

Facts:

- Claude Code's own sandbox does not exist on native Windows (item 1).
- AppContainer: "Granular access can be granted for Internet access,
  Intranet access, and acting as a server"; file access is limited to
  what is granted ([AppContainer isolation][appc-iso]). Windows
  Filtering Platform default block filters drop traffic a container
  lacks the capability for, and loopback is blocked until
  `CheckNetIsolation.exe LoopbackExempt -a` (outbound) or `-is`
  (inbound) is run ([UWP firewall troubleshooting][uwp-fw]).
  `CreateAppContainerProfile` is a desktop API since Windows 8
  ([CreateAppContainerProfile][cacp]).
- Fit per boundary (inference): the core under an AppContainer with no
  capabilities has no network, loopback included, which is ENG-12's
  core case. B1 needs a loopback exemption, an administrator action;
  that hosted Windows runners grant it is unverified. The container's
  identity must be granted access to the test home, since AppContainer
  file isolation is deny-by-default.
- Job objects: with `JOB_OBJECT_LIMIT_ACTIVE_PROCESS`, a process that
  would exceed the limit "is terminated and the association fails";
  `JOB_OBJECT_LIMIT_PROCESS_MEMORY` makes over-limit commits fail
  ([job limits][job-limits]). An active-process limit of one blocks
  program starts at run time (HC-4), and the memory limit is CMP-05's
  "OS resource limits"; the kernel worker can join its own job before
  reading input (inference).
- Windows Sandbox is a Hyper-V VM; GitHub's hosted Windows runners are
  "not enabled for nested virtualization"
  ([community discussion][gh-nested], secondary). It is out of hosted
  CI.
- Witness runs (OWN-18): an AppContainer can deny network and the
  user's home, so Windows can enforce the denial; without it, the view
  says the platform cannot (inference).

**Go.** `x/sys/windows` has job objects, `JOB_OBJECT_LIMIT_ACTIVE_PROCESS`
and the process-thread attribute list ([syscall_windows.go][xsys-job]),
but no AppContainer calls or `SECURITY_CAPABILITIES`; those come from
`userenv.dll` through `NewLazySystemDLL`, in-house.

**Rust.** `windows-sys` has `CreateAppContainerProfile` under
`Win32_Security_Isolation`; win32job 2.0.3 (MIT or Apache-2.0) wraps
job objects ([Cargo.toml][ws-cargo], [crates.io][win32job]).

**Maturity.** The OS mechanisms are mature; no maintained library in
either language offers a "run this entry under boundary B" harness, so
it is in-house work in both, as Seatbelt and namespaces are today.

**SRS rows.** [ENG-12, ENG-16][eng], [SEC-19, SEC-29][sec],
[CMP-05][fr], [OWN-18, OWN-22][own], HC-4, HC-18, HC-20.

## 8. Reproducible, signed builds

**Go.** For a cgo-free program "a reproducible build is as simple as
compiling with `CGO_ENABLED=0 go build -trimpath`", and "on Windows,
package net already made direct use of DLLs without C code"; the Go
team builds every distribution on Linux and on Windows and requires
"bit-for-bit identical archives" ([Perfectly Reproducible][go-rebuild]).
A windows/amd64 multi-call binary cross-built from Linux was 6.4 MB
([UI and packaging note][ui]). Adding two Windows rows to
[release.yml][release]'s matrix changes no toolchain.

**Rust.**

- MSVC links the C runtime dynamically by default; `crt-static` links
  it statically ([linkage][rs-crt]). Without it the binary needs the
  Visual C++ runtime, against HC-11 (inference).
- rustc has passed `/PDBALTPATH:%_PDB%` to MSVC linkers since March
  2024, so only the PDB's file name is embedded
  ([rust#121297][rs-121297]). `/Brepro` for timestamps is the
  project's to add; one team found the linker version in the COFF
  header still varies with the toolchain ([Threema][rb-threema]).
- Cross-building from Linux uses cargo-xwin, which downloads the
  Microsoft CRT and Windows SDK under a Microsoft license the user
  accepts, and links with clang-cl and lld-link ([cargo-xwin][xwin]).
  That puts a Microsoft-licensed sysroot in the build, which ENG-18,
  ENG-20 and the SBOM must account for (inference). The
  `*-windows-gnullvm` targets are Tier 2 ([platform support][rs-plat]).

**Authenticode against bit-identical builds.**

- A PE signature is embedded; the hash "omits the file's checksum …
  and the Certificate Table directory" ([PE signatures][pe-sign]). So
  two builders can match on the unsigned file, which is then signed,
  and a verifier strips the signature to compare, as Go's verifier
  does for macOS ([Perfectly Reproducible][go-rebuild]). ENG-19 then
  reads "identical except the embedded signature", the same wording
  question as macOS in [the frame's open point 19][frame].
- Smart App Control blocks "unknown, unsigned code" and its "signature
  checks apply to all executable files, not just those downloaded from
  the Internet" ([Smart App Control][sac], [SmartScreen][smartscreen]).
  An unsigned `parinn.exe` installed by the plugin can be blocked on
  such machines; hooks then fail open and record nothing (inference).
- SmartScreen: EV certificates no longer bypass it since 2024; Azure
  Artifact Signing costs about $9.99 a month and serves organizations
  in the USA, Canada, the EU and the UK and individuals in the USA and
  Canada; OV certificates cost $150–300 a year with keys on an HSM;
  SignPath Foundation signs qualifying open-source projects for free
  ([code signing options][signopts]).
- Sigstore (ENG-20) is unaffected; Authenticode is added beside it, with
  a signing credential in the release job.
- Antivirus false positives on Go binaries are "a common occurrence,
  especially on Windows" ([Go FAQ][go-faq]).

**Maturity.** Go's Windows reproducibility is proven by the Go project
itself. Rust's MSVC path is reproducible with care, and less proven
cross-built from Linux (unverified end to end).

**SRS rows.** [ENG-01, ENG-19, ENG-20, ENG-29][eng], [CON-02,
CON-03][ctx], [NFR-10][nfr], HC-11, HC-12, T11.

## 9. Windows on ARM64

- **Go.** "Go 1.17 adds support of 64-bit ARM architecture on Windows
  (the windows/arm64 port)" ([Go 1.17][go117]); it cross-builds with
  `CGO_ENABLED=0`; Go needs Windows 10 or Server 2016 since 1.21
  ([Go 1.21][go121]). ncruces/go-sqlite3 tests windows/arm64 on every
  commit ([README][gosq-readme]).
- **Rust.** `aarch64-pc-windows-msvc` became Tier 1 with host tools in
  Rust 1.91.0 (30 Oct 2025) ([Rust 1.91.0][rs191],
  [platform support][rs-plat]). C dependencies (bundled SQLite, native
  libghostty-vt) need an ARM64 MSVC or clang-cl build.
- **Claude Code** ships `win32-arm64` natively ([setup][cc-setup]).
- **CI.** Windows ARM64 hosted runners: public preview in April 2025
  ([changelog][gh-arm-1]), generally available for public repositories
  in August 2025 ([changelog][gh-arm-2]), in private repositories since
  January 2026 with two vCPUs ([changelog][gh-arm-3]).
- **Fallback.** Windows 11 24H2 runs x64 apps under Prism emulation,
  user mode only ([Microsoft][prism]); an emulated hook pays
  translation cost against NFR-01 (unmeasured).
- **libghostty-vt** maps `aarch64-pc-windows-msvc` in herdr
  ([herdr build.rs][herdr-build]); the WASM path needs nothing.

**SRS rows.** [CON-02][ctx], [NFR-01, NFR-10][nfr], [ENG-19][eng].

## Phones as clients

Phones stay clients: stage one is a browser through an owner-run
tunnel (row 11), so it adds no build target; stage two writes signed
segments (row 17, PRV-10). For that stage, Go has `GOOS=ios` and
`GOOS=android` ([Go 1.16][go116]) with gomobile from `x/mobile`, which
publishes pseudo-versions only ([x/mobile][xmobile]); Rust has
`aarch64-apple-ios` and `aarch64-linux-android` at Tier 2
([platform support][rs-plat]). Phone keys are covered in
[the data note][pdc]. The cost beyond that is out of this note's
scope.

## Summary

Effort is an estimate per language for a Windows desktop node at P1:
S under a week, M one to three weeks, L more than three weeks.

| Item                    | Go                                                                                 | Rust                                                                                        | SRS rows                                                  | Effort (Go / Rust) |
| ----------------------- | ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- | --------------------------------------------------------- | ------------------ |
| 1. Claude Code          | Cross-built `.exe`, exec-form hooks; binary selection and hook latency open        | Same                                                                                        | ADM-01–03, NFR-01, NFR-12, ASM-16, OWN-12, OWN-22, ENG-14 | M / M              |
| 2. Pty hosting          | x/conpty, go-pty or in-house on `x/sys`; `os/exec` cannot host ConPTY; no-echo gap | portable-pty 0.9.0, mature; same no-echo gap                                                | OWN-03, OWN-19, SEC-29, NFR-09, ENG-16                    | M / S              |
| 3. SQLite FTS5          | ncruces tested on windows/amd64, arm64; rename not atomic                          | rusqlite bundled, FTS5 on; MSVC C build; rename POSIX on 1607+                              | REC-09, REC-11, NFR-03, NFR-04, NFR-07, SEC-14            | S / M              |
| 4. Permissions          | ACL check via `x/sys/windows`, no cgo                                              | ACL check via `windows-sys`, `unsafe`; helper crate stale                                   | SEC-02, SEC-03, SEC-18, ADM-04, ENG-14, §8.1              | M / M              |
| 5. Key store            | DPAPI in `x/sys`; CredMan and NCrypt by lazy DLL; no cgo                           | `windows-sys`, keyring 4.2.0                                                                | SEC-10, SEC-09, REC-18, REC-24, PRV-10                    | S / S              |
| 6. Local endpoints      | AF_UNIX in `net`; pipes via `x/sys` or go-winio; peer check in-house               | AF_UNIX unstable in std, uds_windows; tokio pipes; peer check via `windows-sys`             | SEC-20, SEC-29, SEC-22, ENG-12                            | M / M              |
| 7. ENG-12 sandbox       | AppContainer by lazy DLL, job objects in `x/sys`; in-house harness                 | AppContainer and jobs in `windows-sys`, win32job; in-house harness                          | ENG-12, ENG-16, SEC-19, CMP-05, OWN-18, OWN-22            | L / L              |
| 8. Reproducible, signed | Cross-built, proven reproducible; add Authenticode after the compare               | MSVC sysroot, `crt-static`, `/Brepro`; cross-build via cargo-xwin less proven; Authenticode | ENG-01, ENG-19, ENG-20, CON-02, NFR-10                    | M / L              |
| 9. Windows ARM64        | windows/arm64 since 1.17, cross-built                                              | Tier 1 since 1.91; C deps need ARM64 MSVC                                                   | CON-02, NFR-10, NFR-01                                    | S / S              |

## What Windows changes in the options

- **The SRS rows change first.** CON-02 and NFR-10 gain windows/amd64
  and windows/arm64. "Statically linked" already fails on darwin
  ([UI and packaging note][ui]); a Windows binary also loads system
  DLLs, so the neutral wording "no dynamic dependency but the OS's
  system libraries" in [the frame][frame] fits all three OSes. SEC-02,
  ENG-14 and ADM-04 need Windows wordings (SID, ACL, `USERPROFILE`,
  HKLM or Program Files). ENG-19 needs "identical except the embedded
  signature". These are stakeholder-approved changes (ENG-21).
- **OWN-22 bites on every native-Windows node.** Claude Code has no
  sandbox there, so widening owner acts on such a node always need a
  recorded risk acceptance, until Claude Code adds one or the session
  runs in WSL 2.
- **Go keeps its cross-build advantage and gains a new evidence cost.**
  Every Windows API ParleyInn needs (ACLs, DPAPI, Credential Manager,
  ConPTY, pipes, job objects, AppContainer) is reachable without cgo;
  Windows is cheaper for Go than macOS's Keychain. But `x/sys/windows`
  holds `CreateProcess`, `Socket`, `Connect` and `WSAIoctl` in the same
  package as the ACL and DPAPI calls the core needs
  ([syscall_windows.go][xsys-sock]), and `os/exec` cannot host ConPTY.
  HC-1 and HC-4 evidence on Windows must therefore work per symbol,
  through a call graph or a wrapper package with an allow-list, not
  per imported package (inference).
- **Rust gains a little evidence and loses on the build.** windows-sys
  gates modules by Cargo feature, so a core crate that does not enable
  `Win32_Networking_WinSock` cannot name `connect` (inference from
  [Cargo.toml][ws-cargo]). The gate is coarse: `CreateProcessW` shares
  `Win32_System_Threading` with `WaitForSingleObject`, `CreateEventW`
  and `Sleep`, and features unify across one binary's graph, so the
  check must build each component crate alone (inference). The MSVC
  runtime, the Windows SDK and C compiles for SQLite enter the
  reproducible pipeline, and the cross-build is less proven than Go's.
- **Native libghostty-vt on Windows is not reproducible today**
  ([ghosttea-vt-sys][ghosttea]). The WASM build stays one OS-neutral
  file, which favours the WASM path in either language.
- **Both languages owe the same in-house work.** An AppContainer and
  job-object harness for ENG-12; an ACL module for SEC-02; a pipe or
  AF_UNIX endpoint with a token check for SEC-29; Windows CI rows on
  hosted x64 and ARM64 runners; an Authenticode signing service in the
  release job, since Smart App Control blocks unsigned code; and a
  Windows measurement of NFR-01 under Defender.
- **A cheaper first stage exists.** Claude Code in WSL 2 runs the
  existing linux binary with Claude Code's sandbox, and a Windows
  browser reaches its loopback lane view. It covers developers who
  already use WSL, defers every item above but 9, and leaves
  native-Windows Claude Code users without ParleyInn (inference).

[ctx]: ../../../docs/srs/02-context.md
[fr]: ../../../docs/srs/05-functional-requirements.md
[own]: ../../../docs/srs/05c-owner-and-peer-requirements.md
[sec]: ../../../docs/srs/06-security.md
[nfr]: ../../../docs/srs/07-non-functional-requirements.md
[data]: ../../../docs/srs/08-data-and-storage.md
[eng]: ../../../docs/srs/10-engineering-quality.md
[frame]: constraints.md
[term]: terminal-components.md
[ui]: ui-and-packaging-components.md
[pdc]: protocol-and-data-components.md
[release]: ../../../.github/workflows/release.yml
[cc-setup]: https://code.claude.com/docs/en/setup
[cc-hooks]: https://code.claude.com/docs/en/hooks
[cc-mcp]: https://code.claude.com/docs/en/mcp
[cc-plugref]: https://code.claude.com/docs/en/plugins-reference
[cc-mkt]: https://code.claude.com/docs/en/plugins/marketplace-reference
[cc-plugts]: https://code.claude.com/docs/en/plugins/troubleshooting
[cc-sandbox]: https://code.claude.com/docs/en/sandboxing
[cc-settings]: https://code.claude.com/docs/en/settings
[cc-managed]: https://code.claude.com/docs/en/managed-settings
[cc-16602]: https://github.com/anthropics/claude-code/issues/16602
[cc-24097]: https://github.com/anthropics/claude-code/issues/24097
[cc-25577]: https://github.com/anthropics/claude-code/issues/25577
[cc-32951]: https://github.com/anthropics/claude-code/issues/32951
[cc-73971]: https://github.com/anthropics/claude-code/issues/73971
[mcp-win-guide]: https://mcp.directory/blog/claude-code-mcp-on-windows-native-wsl-2026-complete-fix-guide
[libuv-search]: https://github.com/libuv/libuv/blob/v1.x/src/win/process.c#L288-L320
[wsl-net]: https://learn.microsoft.com/en-us/windows/wsl/networking
[def-perf]: https://learn.microsoft.com/en-us/defender-endpoint/microsoft-defender-endpoint-antivirus-performance-mode
[ms-cpc]: https://learn.microsoft.com/en-us/windows/console/createpseudoconsole
[ms-conpty-blog]: https://devblogs.microsoft.com/commandline/windows-command-line-introducing-the-windows-pseudo-console-conpty/
[ppty]: https://docs.rs/crate/portable-pty/0.9.0/source/src/win/psuedocon.rs
[wez-pw]: https://wezterm.org/config/lua/config/detect_password_input.html
[go-62708]: https://github.com/golang/go/issues/62708
[xconpty]: https://pkg.go.dev/github.com/charmbracelet/x/conpty@v0.2.0
[gopty]: https://github.com/aymanbagabas/go-pty/blob/v0.2.3/README.md
[gopty-mod]: https://github.com/aymanbagabas/go-pty/blob/v0.2.3/go.mod
[uec-conpty]: https://github.com/UserExistsError/conpty/blob/v0.1.4/conpty.go
[xsys-cpc]: https://github.com/golang/sys/blob/v0.48.0/windows/syscall_windows.go#L1870-L1871
[conpty-crate]: https://crates.io/crates/conpty
[ghostty-12290]: https://github.com/ghostty-org/ghostty/discussions/12290
[herdr-build]: https://github.com/herdrdev/herdr/blob/e35f3937b0ef/crates/ghostty-vt/build.rs
[ghosttea]: https://docs.rs/crate/ghosttea-vt-sys/0.8.0
[gosq-readme]: https://github.com/ncruces/go-sqlite3/blob/v0.35.6/README.md
[gosq-vfs]: https://github.com/ncruces/go-sqlite3/blob/v0.35.6/vfs/README.md
[gosq-matrix]: https://github.com/ncruces/go-sqlite3/wiki/Support-matrix
[rsq-build]: https://github.com/rusqlite/rusqlite/blob/master/libsqlite3-sys/build.rs
[go-rename]: https://github.com/golang/go/blob/go1.26.0/src/os/file.go#L434-L440
[rs-rename]: https://doc.rust-lang.org/std/fs/fn.rename.html
[go-chmod]: https://github.com/golang/go/blob/go1.26.0/src/os/file.go#L629-L647
[go-home]: https://github.com/golang/go/blob/go1.26.0/src/os/file.go#L600-L608
[go-root]: https://github.com/golang/go/blob/go1.26.0/src/os/root.go#L54-L67
[rs-perm]: https://doc.rust-lang.org/std/fs/struct.Permissions.html
[kb947721]: https://learn.microsoft.com/en-US/troubleshoot/windows-server/group-policy/default-owner-objects-created-members-administrators-group-not-available
[go123]: https://go.dev/doc/go1.23
[cyberark-pd]: https://www.cyberark.com/resources/threat-research/anti-virus-vulnerabilities-who-s-guarding-the-watch-tower
[xsys-sec]: https://github.com/golang/sys/blob/v0.48.0/windows/security_windows.go#L1454-L1523
[ws-cargo]: https://docs.rs/crate/windows-sys/0.61.2/source/Cargo.toml
[winacl]: https://crates.io/crates/windows-acl
[dpapi]: https://learn.microsoft.com/en-us/windows/win32/api/dpapi/nf-dpapi-cryptprotectdata
[dpapi-2001]: https://learn.microsoft.com/en-us/previous-versions/ms995355(v=msdn.10)
[credwrite]: https://learn.microsoft.com/en-us/windows/win32/api/wincred/nf-wincred-credwritew
[ksp]: https://learn.microsoft.com/en-us/windows/win32/seccertenroll/cng-key-storage-providers
[curves]: https://learn.microsoft.com/en-us/windows/win32/seccng/cng-named-elliptic-curves
[xsys-dpapi]: https://github.com/golang/sys/blob/v0.48.0/windows/syscall_windows.go#L301
[wincred]: https://github.com/danieljoos/wincred/blob/v1.2.3/sys.go
[keyring]: https://docs.rs/keyring/4.2.0/keyring/
[afunix]: https://devblogs.microsoft.com/commandline/af_unix-comes-to-windows/
[mingw-afunix]: https://mingw.googlesource.com/mingw-w64/+/refs/heads/v14.x/mingw-w64-headers/include/afunix.h
[cnp]: https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createnamedpipea
[gnpcpid]: https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getnamedpipeclientprocessid
[go112]: https://go.dev/doc/go1.12
[xsys-pipe]: https://github.com/golang/sys/blob/v0.48.0/windows/syscall_windows.go#L171
[winio-pipe]: https://github.com/microsoft/go-winio/blob/v0.6.2/pipe.go#L373
[rs-147335]: https://github.com/rust-lang/rust/pull/147335
[uds-win]: https://crates.io/crates/uds_windows
[tokio-np]: https://docs.rs/tokio/latest/tokio/net/windows/named_pipe/struct.ServerOptions.html
[appc-iso]: https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation
[uwp-fw]: https://learn.microsoft.com/en-us/windows/security/operating-system-security/network-security/windows-firewall/troubleshooting-uwp-firewall
[cacp]: https://learn.microsoft.com/en-us/windows/win32/api/userenv/nf-userenv-createappcontainerprofile
[job-limits]: https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_limit_information
[gh-nested]: https://github.com/orgs/community/discussions/25491
[xsys-job]: https://github.com/golang/sys/blob/v0.48.0/windows/syscall_windows.go#L350
[win32job]: https://crates.io/crates/win32job
[go-rebuild]: https://go.dev/blog/rebuild
[rs-crt]: https://doc.rust-lang.org/reference/linkage.html
[rs-121297]: https://github.com/rust-lang/rust/pull/121297
[rb-threema]: https://lists.reproducible-builds.org/pipermail/rb-general/2024-December/003592.html
[xwin]: https://github.com/rust-cross/cargo-xwin
[rs-plat]: https://doc.rust-lang.org/nightly/rustc/platform-support.html
[pe-sign]: https://learn.microsoft.com/en-us/windows/win32/secbp/understanding-pe-signatures
[sac]: https://learn.microsoft.com/en-us/windows/apps/develop/smart-app-control/overview
[smartscreen]: https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation
[signopts]: https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options
[go-faq]: https://go.dev/doc/faq
[go117]: https://go.dev/doc/go1.17
[go121]: https://go.dev/doc/go1.21
[rs191]: https://blog.rust-lang.org/2025/10/30/Rust-1.91.0/
[gh-arm-1]: https://github.blog/changelog/2025-04-14-windows-arm64-hosted-runners-now-available-in-public-preview/
[gh-arm-2]: https://github.blog/changelog/2025-08-07-arm64-hosted-runners-for-public-repositories-are-now-generally-available/
[gh-arm-3]: https://github.blog/changelog/2026-01-29-arm64-standard-runners-are-now-available-in-private-repositories
[prism]: https://learn.microsoft.com/en-us/windows/arm/apps-on-arm-x86-emulation
[go116]: https://go.dev/doc/go1.16
[xmobile]: https://pkg.go.dev/golang.org/x/mobile
[xsys-sock]: https://github.com/golang/sys/blob/v0.48.0/windows/syscall_windows.go#L1104-L1124
