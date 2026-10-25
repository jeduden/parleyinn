# Compile times: what Bun's Zig-to-Rust port shows

Scope: Bun's build times before and after its port from Zig to Rust
(merged 14 May 2026, released as Bun 1.4), gathered on 4 October 2026
for the factory-fitness criteria of the [options](options.md). Bun is
the largest codebase to have run in both languages, and an agent
factory did the port. Figures are as the sources state them; none was
measured here.

## Before: Zig

- **Debug build.** "About 1 minute and 30 seconds" for "~850k lines
  of Zig code" in March 2025. Each invocation "builds everything from
  scratch", incremental compilation was in beta without
  `usingnamespace`, and semantic analysis is single-threaded
  ([zackoverflow][zack]).
- **Bun's own Zig fork** claimed four times faster compilation through
  parallel semantic analysis, shown as about 20 seconds for Bun
  ([Ziggit][fork]).
- **Upstream Zig's answer.** The self-hosted x86_64 backend builds in
  31 seconds what LLVM builds in 2 minutes, and incremental rebuilds
  after a 40-second first build take 172 to 371 ms ([Ziggit][fork]).
  The Zig compiler itself, about 600k lines, is quoted at 16 seconds
  clean and 90 ms incremental ([fawadhs][fawadhs], quoting Andrew
  Kelley). A fork of pre-port Bun on modern Zig, Buz, reports sub-second
  incremental builds ([daily.dev][buz]).

## After: Rust

- **CI, end to end.** The median Linux build fell from 30 m 6 s (Zig,
  1.3.14) to 5 m 37 s (Rust, 1.4.0); Jarred Sumner put it at about 3.4
  times faster end to end, "including C++ compilation, switching full
  LTO -> thin LTO, and cross-compilation on Linux"
  ([Lalit Maganti][buildprof], [Jarred Sumner][jarred]).
- **Where the time went.** A reproduction measured 24 m 24 s against
  5 m 40 s. In the Zig build the `ld.lld` link "ran alone at the very
  end for over sixteen minutes, about two-thirds of the entire build";
  switching to ThinLTO alone cut that link to 7 m 22 s. "Rust spreads
  compilation across >90 crates while the Zig build funnelled
  everything through a single module" ([Lalit Maganti][buildprof]).
- **Local and incremental.** Bun's own write-up gives no post-port
  build numbers; it split the code into about 100 crates so that "the
  Rust would compile faster" ([Bun][bun]). A review calls the missing
  numbers "a conspicuous gap" ([morello.dev][morello]).

## The port itself

- 535,496 lines of Zig became about 780,000 lines of Rust in 11 days,
  with up to 64 Claude instances in parallel and 6,778 commits; the
  method was a mechanical, file-by-file port against the test suite
  ([Bun][bun], [morello.dev][morello]).
- Memory safety, not speed, was the stated motive: Bun had suffered
  many memory bugs in Zig ([The Register][reg]). About 4% of the Rust
  sits inside `unsafe`, 78% of those blocks a single line ([Bun][bun]).
- The API cost is reported as about 5.9 billion input tokens and
  $165,000 ([fawadhs][fawadhs]); unverified against Bun's own figures.
- Runtime: 2.8 to 4.8% faster HTTP, 2.2 to 4.7% faster app workloads,
  about 20% smaller binaries ([Bun][bun]).

## What this means for ParleyInn

- **The headline is about structure and linking, not the language.**
  Bun's 5.4 times faster CI build came mostly from leaving full LTO and
  one giant compilation unit. Neither is forced by Zig, and the same
  build in Zig with ThinLTO and split modules was not measured.
- **The factory's inner loop is incremental debug builds, and there
  the order is Go, then Zig, then Rust.** Zig 0.16's incremental
  rebuilds on the self-hosted backend run in hundreds of milliseconds,
  but only for x86_64 Linux debug builds; iOS, Android and arm64 macOS
  builds go through LLVM. Rust's incremental rebuilds are seconds to
  tens of seconds on large crates, and Bun published none. Go builds a
  codebase of ParleyInn's size in seconds.
- **At ParleyInn's size it matters less.** ParleyInn will be a tenth of Bun or
  smaller, so a clean Rust build is minutes, not half an hour. With
  dozens of agents in compile–test loops it is still CPU per attempt,
  so the bake-off measures it.
- **The language choice is reversible.** A factory ported 535k lines
  in 11 days against a strong test suite. ParleyInn's executable SRS, every
  requirement a scenario driving the binary through the CLI and MCP, is
  that oracle, language-neutral if the bindings stay black-box (open
  point 15). Starting in one language and porting later costs days, not
  a rewrite.
- **Bun's own reason to leave Zig is ParleyInn's main risk.** ParleyInn parses
  hostile input; memory bugs are what drove the move to Rust.

[zack]: https://zackoverflow.dev/writing/i-spent-181-minutes-waiting-for-the-zig-compiler-this-week/
[fork]: https://ziggit.dev/t/bun-s-zig-fork-got-4x-faster-compilation-times/15183
[fawadhs]: https://fawadhs.dev/blog/bun-rust-rewrite-technical-review
[buz]: https://daily.dev/posts/a-drop-in-replacement-for-bun-using-modern-zig-with-sub-1s-incremental-builds-eccl8aynl
[buildprof]: https://lalitm.com/post/buildprof/
[jarred]: https://x.com/jarredsumner/status/2090619419059974620
[bun]: https://bun.com/blog/bun-in-rust
[morello]: https://morello.dev/blog/bun-14-rust-rewrite
[reg]: https://www.theregister.com/2026/05/05/bun_rust_port/
