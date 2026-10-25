# Five-platform apps: frameworks for nodes and phones

Scope: written 4 October 2026, from primary sources (release feeds,
official docs, source code and repositories) read that day. The
stakeholder added a requirement: the same experience on Linux, Windows,
macOS, iOS and Android, "start ParleyInn, then you see your stuff". Desktop
apps are nodes: they hold the record, receive Claude Code's hooks and
MCP calls from harnesses the person starts as usual, and show the room
(rooms left, conversation middle, outcome right: diff, terminal, test
runs, running-app preview). Phone apps are clients of the person's
nodes: they view, sync signed append-only logs, and hold a device key
that signs allow, deny and messages. The core still opens no socket
(SEC-01) and hooks stay fast with no resident core process. This note
extends the [options](options.md), the [comparison](comparison.md)
and the UI and packaging note's
[shells section](ui-and-packaging-components.md#c-desktop-and-mobile-shells);
facts already there are linked, not repeated. Claims marked
(unverified) rest on reasoning or a secondary source; claims marked
(reasoning) are this note's inference. Nothing was measured for this
note.

## What the requirement asks of a framework

- **Desktop: an app that is a node.** The window is a UI component the
  person starts. The hook and MCP entry stays a small, separate core
  executable shipped inside the app bundle, which the harness starts
  per call, so no core process stays resident and the core never starts
  the window (SEC-01, X8) (reasoning). A framework must therefore let
  the bundle carry a second, signed executable, and must not force the
  hook path through the GUI binary: on Linux a Tauri binary links
  WebKitGTK and GTK, so every hook would pay their loading
  ([Tauri prerequisites](https://v2.tauri.app/start/prerequisites/))
  (reasoning, unmeasured).
- **Desktop: the room without a loopback port.** In every app framework
  below the window reaches its backend by in-process IPC, not HTTP, so
  an app window needs no B1 listener; the browser page of options A to
  E keeps one only for browsers and the stage-one phone (reasoning).
- **Phone: a client, not a node.** Sync of writer logs is a B2 peer
  component, off until the tenant turns it on (I4). The phone needs a
  hardware-held key, local notifications and, ideally, background
  wake-up, which no framework can supply without a vendor push service
  (see [the platforms](#what-the-platforms-impose-whatever-the-framework)).
- **Windows becomes a node target.** CON-02 lists linux/amd64,
  linux/arm64 and darwin/arm64 only
  ([02-context.md](../../../docs/srs/02-context.md)); a Windows node
  needs a CON-02 change and ConPTY for the run component (reasoning).

## What the platforms impose, whatever the framework

### Windows: WebView2 reports to Microsoft

- WebView2 "collects a set of optional and required diagnostic data";
  optional data follows the Windows Diagnostic data setting, and
  "Regardless of the Windows Diagnostic data setting, WebView2 collects
  required data". SmartScreen "is enabled by default" and crash dumps
  "are created and sent to Microsoft" unless the app sets
  `IsCustomCrashReportingEnabled` —
  [WebView2 data and privacy](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/data-privacy).
- Tauri's webview layer wry passes
  `--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection` by
  default — [wry 0.57.0 source](https://docs.rs/crate/wry/0.57.0/source/src/webview2/mod.rs).
  A search of that source found no crash-reporting switch (reasoning
  from a text search).
- The Evergreen runtime "updates automatically"; the Fixed Version
  binaries "are over 250 MB" and are patched by the app vendor —
  [WebView2 distribution](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution).
  It "will be included as part of the Windows 11 operating system"
  (same source).
- Consequence: any framework that draws the room in WebView2 (Tauri,
  Wails, Electrobun, Dioxus, a Flutter webview pane) ships an OS
  component that sends required diagnostics. I4 says no component sends
  telemetry; whether an OS component counts is a stakeholder ruling
  (reasoning). Flutter and Compose draw their own pixels and avoid
  WebView2 unless a webview pane is added.

### Android: the system WebView calls Google by default

- WebView verifies URLs with Google Safe Browsing; "the default value
  of `EnableSafeBrowsing` is true", and the app opts out of metrics with
  the `android.webkit.WebView.MetricsOptOut` manifest flag —
  [Manage WebView objects](https://developer.android.com/develop/ui/views/layout/webapps/managing-webview).
  Every WebView-based phone app (Tauri, Capacitor, Wails, Dioxus) must
  set both (reasoning).

### Apple: notarized outside the Mac App Store

- "To distribute a macOS app through the Mac App Store, you must enable
  the App Sandbox capability" —
  [App Sandbox](https://developer.apple.com/documentation/security/app-sandbox).
  A sandboxed hook binary reading `~/.claude` and the tenant's home
  fights I8's home binding, so the node ships with Developer ID signing
  and notarization outside the store (reasoning). Tauri notes that a
  free Apple account cannot notarize —
  [Tauri macOS signing](https://v2.tauri.app/distribute/sign/macos/).
- Local network privacy covers iOS since 14 and macOS since 15:
  "Making an outgoing TCP connection" to a local network address needs
  the person's approval; listening and accepting does not. On macOS,
  "Command-line tools run from Terminal or over SSH" and `launchd`
  daemons are allowed automatically —
  [TN3179](https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy).
  A phone reaching a node on the LAN asks once; the node's own hook
  binary is exempt (reasoning from the same source).
- iOS background work: a refresh task runs when "The system decides the
  best time", with "up to 30 seconds of background runtime"; to wake an
  app when new content exists, Apple points to background pushes sent
  through APNs —
  [Choosing background strategies](https://developer.apple.com/documentation/backgroundtasks/choosing-background-strategies-for-your-app).
  VIEW-17 forbids a vendor push service, so an iPhone learns of a held
  request only when opened or at a refresh the system chooses, in every
  framework (reasoning).

### Android: verification, page size and key types

- Developer verification: from 30 September 2026 "protections begin"
  for installs in Brazil, Indonesia, Singapore and Thailand on certified
  devices, with global expansion in "2027 and beyond"; hobbyist
  accounts may share apps "with up to 20 devices", and an "advanced
  flow" serves power users —
  [Android developer verification](https://developer.android.com/developer-verification).
  An owner-built APK is affected from 2027 (unverified for adb).
- Google Play: apps targeting Android 15 must support 16 KB pages, and
  from 1 February 2027 updates that do not "won't be able to release";
  NDK r28 aligns by default —
  [16 KB page sizes](https://developer.android.com/guide/practices/page-sizes).
  Go libraries built with older NDKs need
  `-Wl,-z,max-page-size=16384` —
  [Dan Ballard, 2025](https://danballard.com/2025/09/28/generating-android-16kb-page-size-libraries-from-go/)
  (secondary).
- StrongBox supports "ECDSA, ECDH P-256", not Ed25519 —
  [Android keystore](https://developer.android.com/privacy-and-security/keystore).
  AOSP added Curve 25519 to the keystore provider
  ([AOSP commit](https://android.googlesource.com/platform/frameworks/base/+/25c8f48a8dd4)),
  at an API level not verified here. With the Secure Enclave also P-256
  ([options, common ground 10](options.md#common-ground)), both phones
  hold hardware keys as P-256 only: open point 13 gets stronger, and no
  framework changes it.

## Tauri 2

- **Status.** 2.12.1 shipped 30 September 2026; 3.0.0-alpha.4 on
  1 October 2026, the first v3 alpha on 13 September —
  [crates.io](https://crates.io/crates/tauri). 2.12 dropped Windows 7,
  moved Android templates to Gradle 9 and Kotlin 2 and raised the
  minimum Rust to 1.90 — [Tauri 2.12](https://v2.tauri.app/blog/tauri-2.12/).
- **Tauri 3.** The webview runtime is chosen when building the app:
  `tauri_runtime_wry::Wry` (system webview) or `tauri_runtime_cef::Cef`
  (Chromium Embedded Framework); `tauri-runtime-wry` uses GTK 3 and
  `tauri-runtime-cef` GTK 4; plugin hooks may now run concurrently —
  [v3 changelog](https://github.com/tauri-apps/tauri/blob/v3/crates/tauri/CHANGELOG.md).
  `tauri-runtime-cef` is at 3.0.0-alpha.5 —
  [crates.io](https://crates.io/crates/tauri-runtime-cef). A bundled
  Chromium is the eventual answer to WebKitGTK problems on Linux, at a
  size not stated (unverified).
- **Mobile maturity.** Since 2.0 iOS and Android ship from the same
  project, but "We are not completely happy about the developer
  experience at the moment" and "On mobile not all of the official
  plugins are supported" — [Tauri 2.0](https://v2.tauri.app/blog/tauri-20/).
  2.11 and 2.12 still fix Android crashes and permission bugs (activity
  restarts, plugin permission requests) —
  [v2 changelog](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/CHANGELOG.md).
- **Plugins** (official table) —
  [plugins-workspace](https://github.com/tauri-apps/plugins-workspace/blob/v2/README.md):
  - biometric: iOS and Android only; it prompts and reports success,
    with no key binding — [biometric](https://v2.tauri.app/plugin/biometric/);
  - notification: all five, local only; on Windows it "Only works for
    installed apps" — [notification](https://v2.tauri.app/plugin/notification/);
  - deep-link: all five; iOS and Android register in config only —
    [deep linking](https://v2.tauri.app/plugin/deep-linking/);
  - stronghold: its own encrypted vault file, not the OS key store,
    mobile marked unknown — [stronghold](https://v2.tauri.app/plugin/stronghold/);
  - updater: desktop only, opt-in, and "needs a signature ... This
    cannot be disabled"; it checks only when the app calls `check()` —
    [updater](https://v2.tauri.app/plugin/updater/);
  - no official key-store, Secure Enclave or background-task plugin. A
    community plugin for Secure Enclave and StrongBox is at 0.1.0 —
    [tauri-plugin-crypto-hw](https://unpkg.com/@auvo/tauri-plugin-crypto-hw-api@0.1.0/README.md).
    ParleyInn writes its own Swift and Kotlin key plugin (reasoning).
- **IPC, no server.** Commands use "a JSON-RPC like protocol" over
  message passing — [IPC](https://v2.tauri.app/concept/inter-process-communication/);
  2.0 moved IPC onto custom protocols — [Tauri 2.0](https://v2.tauri.app/blog/tauri-20/).
  The page loads from `tauri://localhost` or `http://tauri.localhost`, a
  scheme the runtime serves in process, not a listener (reasoning)
  ([v3 changelog](https://github.com/tauri-apps/tauri/blob/v3/crates/tauri/CHANGELOG.md)).
  A `localhost` plugin exists for apps that want a real server
  ([plugins-workspace](https://github.com/tauri-apps/plugins-workspace/blob/v2/README.md));
  ParleyInn would not add it.
- **Security model.** Capabilities grant permissions per window or
  webview and per platform; remote URLs get commands only when listed —
  [capabilities](https://v2.tauri.app/security/capabilities/). That fits
  a running-app preview in its own webview with no capability
  (reasoning). 2.12 fixed GHSA-w28w-mhc8-qvjv (channel data readable by
  other webviews) and an ACL bug where a deny for remote origins also
  denied local ones; 2.11.1 fixed `.localhost` origin confusion on
  Windows and Android — [v2 changelog](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/CHANGELOG.md).
  2.0 had an external audit by Radically Open Security —
  [Tauri 2.0](https://v2.tauri.app/blog/tauri-20/).
- **Webviews.** WebView2, WKWebView, WebKitGTK and Android System
  WebView — [webview versions](https://v2.tauri.app/reference/webview-versions/).
  Linux needs `libwebkit2gtk-4.1` —
  [prerequisites](https://v2.tauri.app/start/prerequisites/). Tauri's
  own page lists blank windows, flicker and crashes with the DMABUF
  renderer on NVIDIA, and WebGL "may fall back to slow rendering paths
  without errors" — [Linux graphics](https://v2.tauri.app/develop/debug/linux-graphics/).
  That matters for ghostty-web's canvas (unverified).
- **WebView2 install modes.** `downloadBootstrapper` (the default)
  needs the Internet; `offlineInstaller` adds about 127 MB;
  `fixedVersion` about 180 MB —
  [Windows installer](https://v2.tauri.app/distribute/windows-installer/).
  An installer that fetches from Microsoft at install time is a choice
  the owner should see (reasoning).
- **Go core.** Desktop: the Go binary is an `externalBin` sidecar named
  by target triple, run under a `shell:allow-execute` or
  `shell:allow-spawn` capability with scoped arguments —
  [sidecar](https://v2.tauri.app/develop/sidecar/). Mobile: the shell
  plugin "Only allows to open URLs" on iOS and Android —
  [shell](https://v2.tauri.app/plugin/shell/) — so Go would be linked
  as a library into the generated Xcode and Gradle projects (no example
  found; unverified).
- **Rust core.** In process everywhere; on Android the app calls
  `System.loadLibrary` and mobile plugins are Swift and Kotlin classes —
  [mobile plugins](https://v2.tauri.app/develop/plugins/develop-mobile/).
- **One app per platform.** Desktop: one executable with embedded
  assets plus sidecar files in the bundle; AppImage "bundles all
  dependencies" and must be built on the oldest base system with
  WebKitGTK 4.1 — [AppImage](https://v2.tauri.app/distribute/appimage/).
  Mobile: one `.ipa` or `.apk`; iOS builds need macOS and Xcode —
  [prerequisites](https://v2.tauri.app/start/prerequisites/).
- **Size and reproducibility.** "As little as 600KB" is claimed
  ([UI note](ui-and-packaging-components.md#c-desktop-and-mobile-shells));
  the size page gives profile settings and no numbers —
  [size](https://v2.tauri.app/concept/size/). No reproducibility
  statement was found (unverified).
- **License.** Apache-2.0 OR MIT ([crates.io](https://crates.io/crates/tauri)).
  No telemetry found in the framework (unverified).
- **Who ships on five.** NextGraph targets "Linux, MacOS, Windows,
  Android, iOS" with Tauri, but its README says "iOS hasn't been tested
  yet" and the app is mid-refactor —
  [NextGraph ng-app](https://git.nextgraph.org/NextGraph/nextgraph-rs/src/branch/master/ng-app/README.md),
  [new app README](https://git.nextgraph.org/NextGraph/nextgraph-rs/src/branch/allelo/app/nextgraph/README.md).
  Block Buzz uses Tauri for desktop only (below). No product verified
  shipping all five on Tauri.

## Block Buzz, verified

- Buzz's README lists "Desktop app (Tauri + React)" as working today
  and "Mobile clients (iOS + Android, Flutter)" as "Being wired up";
  push notifications are "pending code". Releases are macOS, Linux
  (AppImage, deb) and an unsigned Windows installer —
  [block/buzz README](https://github.com/block/buzz/blob/main/README.md).
- So the shape is Rust relay, Tauri desktop, Flutter phone, and today
  it ships on three platforms, not five.

## Wails v3

- **Status.** v3.0.0-beta.27, 1 October 2026 —
  [Go module proxy](https://proxy.golang.org/github.com/wailsapp/wails/v3/@latest).
  The beta contract covers Windows, macOS and Linux (GTK4 with
  WebKitGTK 6.0); "Android and iOS support is experimental and does not
  block the desktop Beta" —
  [Wails status](https://docs-preview.wails.io/status/).
- **Mobile.** iOS is a "WKWebView + UIKit host"; Android compiles Go to
  `libwails.so` through the NDK; assets come "from Go memory, not a
  localhost server. No open ports"; multiple windows, menus and tray are
  no-ops — [mobile guide](https://docs-preview.wails.io/guides/mobile/).
- **cgo.** Windows "doesn't require CGO by default"; macOS and Linux
  "require CGO for WebView integration", cross-built with a Zig-based
  Docker image —
  [cross-platform](https://docs-preview.wails.io/guides/build/cross-platform/).
  Go on ios/arm64 must link externally —
  [Go internal/platform](https://github.com/golang/go/blob/master/src/internal/platform/supported.go).
- **Dependencies.** The v3 module requires 35 modules directly,
  including `modernc.org/sqlite`, the MCP Go SDK and two WebSocket
  libraries — [v3 go.mod](https://github.com/wailsapp/wails/blob/master/v3/go.mod).
  How many reach an app binary was not measured.
- **License.** MIT. **Verdict.** A Go-only desktop app is in reach; the
  phone half is experimental and the cgo webview stays out of the core
  binary (CON-02). Not a five-platform base in 2026.

## Go on mobile

- **gomobile is maintained.** `golang.org/x/mobile` has no tags; the
  latest pseudo-version is from 8 September 2026 —
  [Go module proxy](https://proxy.golang.org/golang.org/x/mobile/@latest) —
  and August 2026 commits fix NDK handling, Apple build docs and source
  overlays — [x/mobile log](https://go.googlesource.com/mobile/+log).
- **What it builds.** `gomobile bind` makes an `.aar` for Android (API
  16 minimum by default) and an XCFramework for iOS, iOS simulator,
  macOS and Mac Catalyst (iOS 13 by default) —
  [gomobile](https://pkg.go.dev/golang.org/x/mobile/cmd/gomobile).
  "Only a subset of Go types" cross the binding —
  [Go wiki: Mobile](https://go.dev/wiki/Mobile).
- **Without gomobile.** `c-archive` is supported on ios, `c-shared` is
  not; `c-shared` covers android, darwin, linux and windows —
  [Go internal/platform](https://github.com/golang/go/blob/master/src/internal/platform/supported.go).
  Both need cgo, so a phone build is never `CGO_ENABLED=0`; that is
  fine because CON-02 binds the core binary, not a phone library
  (reasoning).
- **Known issues.** Two Go runtimes in one process collide: a Go
  `c-shared` library loaded by a Go program crashed with "address space
  conflict" — [golang/go#18976](https://github.com/golang/go/issues/18976).
  So a phone app embeds exactly one Go library (reasoning). The 16 KB
  page rule above applies to Go `.so` files.
- **Precedent.** Tailscale's Android app runs `gomobile bind -target
  android -androidapi 26` over its Go core —
  [tailscale-android Makefile](https://github.com/tailscale/tailscale-android/blob/main/Makefile).
  Lantern builds its Go core with gomobile for Android (`.aar`) and iOS
  (`.xcframework`) and with FFI on desktop, under Flutter —
  [lantern-client README](https://github.com/getlantern/lantern-client/blob/main/README.md).
  Cwtch uses `c-shared` plus `dart:ffi` on desktop and gomobile on
  Android, and passes JSON strings across both —
  [Cwtch: Flutter with native Go](https://openprivacy.ca/discreet-log/09-flutter-with-native-go-libraries/).
- **Go inside Tauri.** On desktop, a sidecar. On a phone, a gomobile
  XCFramework or `.aar` linked into Tauri's generated projects and
  called from a small Swift or Kotlin plugin; no example was found
  (unverified, a spike).
- **Size.** The Go core is 1.6 MB stripped on linux/amd64
  ([UI note](ui-and-packaging-components.md#a2-go-multi-call-binary));
  phone library sizes were not measured.

## Compose Multiplatform

- **Status.** 1.12.1 is the latest stable and 1.13.0-alpha01 appeared
  on 22 September 2026 —
  [Maven Central](https://repo1.maven.org/maven2/org/jetbrains/compose/compose-gradle-plugin/maven-metadata.xml);
  Kotlin 2.4.20 is stable —
  [Maven Central](https://repo1.maven.org/maven2/org/jetbrains/kotlin/kotlin-gradle-plugin/maven-metadata.xml).
  1.12 added an experimental MCP server to Hot Reload and a second
  desktop window API — [JetBrains](https://blog.jetbrains.com/kotlin/2026/08/compose-multiplatform-1-12-0/).
- **Per platform.** Android, iOS and "Desktop (JVM)" are Stable; web on
  Kotlin/Wasm is Beta —
  [supported platforms](https://kotlinlang.org/docs/multiplatform/supported-platforms.html).
  Compose on desktop is a JVM app.
- **Desktop packaging.** `jpackage` with a bundled, `jlink`-trimmed
  JDK; dmg, pkg, exe, msi, deb, rpm; "Cross-compilation is currently
  not supported", so a `.dmg` is built on macOS —
  [native distributions](https://kotlinlang.org/docs/multiplatform/compose-native-distribution.html).
  No single executable, and the hook path cannot be a JVM start
  (reasoning).
- **Kotlin/Native core.** macosArm64 and iosArm64 are Tier 1, linuxX64
  and linuxArm64 Tier 2, mingwX64 (Windows) and androidNativeArm64
  Tier 3, macosX64 deprecated —
  [Kotlin/Native targets](https://kotlinlang.org/docs/native-target-support.html).
  A Kotlin/Native hook binary on Windows rests on a Tier 3 target, and
  option F's missing core libraries remain ([comparison](comparison.md)).
- **Panes.** Compose draws to a canvas; the nearest terminal is
  JetBrains' JediTerm, a Swing panel dual-licensed LGPLv3 and
  Apache-2.0 — [JediTerm](https://github.com/JetBrains/jediterm).
  Desktop only; nothing comparable found for Compose on phones
  (unverified).
- **Foreign cores.** Rust through JNI (`jni` crate, MIT OR Apache-2.0)
  or UniFFI, which is MPL-2.0 and off the ENG-18 list —
  [crates.io uniffi](https://crates.io/crates/uniffi),
  [crates.io jni](https://crates.io/crates/jni). Go through gomobile on
  Android and a C library on iOS and the JVM (unverified).
- **License.** Apache-2.0. **Who ships on five.** None found.

## Flutter

- **Status.** 3.47.6 stable, 1 October 2026 (Dart 3.13.5) —
  [Flutter releases](https://storage.googleapis.com/flutter_infra_release/releases/releases_linux.json).
  Supported: Android API 24 to 37, iOS 15 to 27, Windows 10 and 11 on
  x64 and Arm64, macOS 12 to 27, Debian 10 to 13 and Ubuntu 20.04 to
  24.04 LTS on x64 and Arm64; macOS Intel is being phased out —
  [supported platforms](https://docs.flutter.dev/reference/supported-platforms).
- **UI tech.** Flutter draws its own widgets (Impeller or Skia), so the
  room needs no webview, except for the running-app preview (reasoning).
- **Native code.** `dart:ffi` with build hooks since Flutter 3.38:
  "no OS-specific logic required for `dlopen`-ing" on Android, iOS,
  macOS, Windows and Linux; prebuilt libraries are bundled as
  `CodeAsset`s — [bind native code](https://docs.flutter.dev/platform-integration/bind-native-code).
  Build hooks arrived in Dart 3.10, link hooks in 3.13 —
  [Dart hooks](https://dart.dev/tools/hooks).
- **Rust core.** flutter_rust_bridge 2.13.0 (MIT, a Flutter Favorite)
  covers all five platforms and web —
  [pub.dev](https://pub.dev/packages/flutter_rust_bridge).
- **Go core.** `c-shared` on desktop and Android; on iOS, where Go has
  no `c-shared`, a gomobile XCFramework, as Lantern does (sources in
  [Go on mobile](#go-on-mobile)). Whether Dart's bundled `CodeAsset`
  takes a static Go archive on iOS was not verified.
- **The three panes in Flutter widgets.**
  - Terminal: xterm.dart 4.0.0 (MIT), last published 27 February 2024 —
    [pub.dev xterm](https://pub.dev/packages/xterm); it renders and
    leaves the pty to the app —
    [xterm.dart](https://github.com/TerminalStudio/xterm.dart).
  - Diff: flutter_diff_viewer 1.4.0 (3 likes) and diffine 1.0.0 (one
    release, September 2026) —
    [flutter_diff_viewer](https://pub.dev/packages/flutter_diff_viewer),
    [diffine](https://pub.dev/packages/diffine); the maintained code
    editor is re_editor 0.10.0 (MIT) —
    [re_editor](https://pub.dev/packages/re_editor).
  - Webview, for the preview: the official webview_flutter supports
    Android, iOS and macOS only —
    [webview_flutter](https://pub.dev/packages/webview_flutter);
    flutter_inappwebview has no Linux and last published 6.1.5 in
    October 2024 — [flutter_inappwebview](https://pub.dev/packages/flutter_inappwebview);
    webview_all claims all five with 90 likes —
    [webview_all](https://pub.dev/packages/webview_all).
  - So the room's panes are the weak part: the TypeScript panes of
    options B to D (CodeMirror merge, ghostty-web) do not carry over,
    and the Linux preview has no maintained official webview
    (reasoning from the above).
- **Phone plugins.** flutter_secure_storage (all five, BSD-3-Clause),
  local_auth (no Linux), flutter_local_notifications (all five),
  workmanager (Android, iOS, macOS), app_links (all five) —
  [flutter_secure_storage](https://pub.dev/packages/flutter_secure_storage),
  [local_auth](https://pub.dev/packages/local_auth),
  [flutter_local_notifications](https://pub.dev/packages/flutter_local_notifications),
  [workmanager](https://pub.dev/packages/workmanager),
  [app_links](https://pub.dev/packages/app_links). None signs with a
  Secure Enclave key; that stays a Swift and Kotlin plugin (reasoning).
- **Telemetry.** The `flutter` tool's analytics and crash reports are on
  by default; `flutter --disable-analytics` turns them off —
  [crash reporting](https://docs.flutter.dev/reference/crash-reporting).
  That is the developer tool; no app-side reporting was found
  (unverified). Updates are the stores' or the app's own.
- **Size.** A default demo app was 5.4 MB to download on iOS, measured
  with Flutter 1.17 — [app size](https://docs.flutter.dev/perf/app-size).
  Current numbers not verified.
- **License.** BSD-3-Clause
  ([UI note](ui-and-packaging-components.md#b1-frameworks)).
- **Who ships on five.** AppFlowy (Flutter and Rust, AGPL-3.0) ships
  desktop releases for macOS, Windows and Linux and apps on the App
  Store and Play Store —
  [AppFlowy README](https://github.com/AppFlowy-IO/AppFlowy/blob/main/README.md).
  RustDesk's "Flutter code for desktop and mobile" sits on a Rust core
  through flutter_rust_bridge 1.80 —
  [RustDesk README](https://github.com/rustdesk/rustdesk/blob/master/README.md),
  [Cargo.toml](https://github.com/rustdesk/rustdesk/blob/master/Cargo.toml).
  Lantern does the same over a Go core (above).

## Capacitor, Electron and Electrobun

- **Capacitor 8.5.2** (MIT, 11 September 2026) —
  [npm](https://www.npmjs.com/package/@capacitor/core) — targets "iOS,
  Android, and more" with plugins in Swift, Java and JavaScript —
  [Capacitor docs](https://capacitorjs.com/docs). Its desktop path,
  `@capacitor-community/electron`, last published 5.0.1 on
  21 September 2023 —
  [npm](https://www.npmjs.com/package/@capacitor-community/electron).
  The CLI sends anonymous telemetry until `npx cap telemetry off` —
  [Capacitor telemetry](https://capacitorjs.com/docs/cli/telemetry).
  A phone shell only.
- **Electron 44.5.1** (MIT, 30 September 2026) —
  [npm](https://www.npmjs.com/package/electron). Desktop only, 123 to
  158 MB zipped and a framework directory, not one binary
  ([UI note](ui-and-packaging-components.md#c-desktop-and-mobile-shells));
  CON-03 bars Node.js. Its spellchecker downloads dictionaries "from a
  Google CDN by default" —
  [Electron spellchecker](https://www.electronjs.org/docs/latest/tutorial/spellchecker).
- **Electrobun 2.0.2** (MIT, 29 September 2026) —
  [npm](https://www.npmjs.com/package/electrobun). Officially macOS 14+,
  Windows 11 x64 and Ubuntu 24.04+, Windows Arm64 in beta, other Linux
  by the community; system webview by default or a bundled CEF; no
  mobile — [Electrobun](https://github.com/blackboardsh/electrobun).
  Bun's LGPL-2 JavaScriptCore problem from option E applies
  ([options](options.md#option-e-typescript-on-bun)).
- **Together.** Electron or Electrobun on desktop plus Capacitor on the
  phone covers five with one TypeScript page, two shells and no native
  core in the shells; the core stays a separate binary. Precedent:
  Happy and T3 Code ship Electron desktops and Expo phones
  ([prior art](prior-art-stacks.md#patterns)).

## React Native with Windows and macOS

- react-native 0.87.1, react-native-windows 0.84.0 and
  react-native-macos 0.83.0, all MIT —
  [npm](https://www.npmjs.com/package/react-native),
  [npm](https://www.npmjs.com/package/react-native-windows),
  [npm](https://www.npmjs.com/package/react-native-macos).
- Windows and macOS are partner platforms; the only Linux entry is
  React Native Skia, "Currently supports Linux and macOS", from the
  community — [out-of-tree platforms](https://reactnative.dev/docs/out-of-tree-platforms).
- Native views, so the panes are new components on each platform, and
  a Go or Rust core enters as a native module (unverified). No
  maintained Linux path: it does not cover five.

## Dioxus

- **Status.** 0.7.10 stable; 0.8.0-alpha.1 on 31 July 2026; MIT OR
  Apache-2.0 — [crates.io](https://crates.io/crates/dioxus).
- **Platforms.** Desktop renders "using Webview or - experimentally -
  with WGPU or Freya", in "Portable <3mb binaries" for macOS, Linux and
  Windows; mobile builds `.ipa` and `.apk` and calls "directly into Java
  and Objective-C" — [Dioxus README](https://github.com/DioxusLabs/dioxus).
  The webview is wry, as in Tauri, so the WebView2 and WebKitGTK points
  above apply (reasoning).
- **Maturity.** 0.7 (8 September 2025) put "Native platform APIs
  (camera, geolocation, file system, push notifications, and more)" in
  0.8, and Blitz, the native renderer, "is still considered a 'work in
  progress'" — [Dioxus 0.7](https://dioxuslabs.com/blog/release-070/).
- **Telemetry.** The `dx` CLI collects anonymous telemetry, off with the
  `disable-telemetry` feature or `TELEMETRY=false` —
  [dioxus-cli-telemetry](https://docs.rs/dioxus-cli-telemetry/).
- **Verdict.** Covers five on paper with the Rust core in process; the
  UI is Rust (RSX), not the TypeScript page, and the phone plugins are
  a release away. Watch, not adopt.

## The table

| Framework                         | Linux                             | Windows                      | macOS             | iOS                        | Android                    | UI tech                         | Go core path                                             | Rust core path                     | License           | Verdict                        |
| --------------------------------- | --------------------------------- | ---------------------------- | ----------------- | -------------------------- | -------------------------- | ------------------------------- | -------------------------------------------------------- | ---------------------------------- | ----------------- | ------------------------------ |
| Tauri 2.12 (3.0 alpha)            | stable; WebKitGTK 4.1, GPU issues | stable; WebView2 diagnostics | stable            | since 2.0; DX "not on par" | since 2.0; fixes continue  | system webview + TS page        | desktop sidecar; phone via gomobile library (unverified) | in process everywhere              | Apache-2.0 OR MIT | lead                           |
| Wails v3 beta.27                  | beta; cgo, GTK4                   | beta; no cgo                 | beta; cgo         | experimental               | experimental               | system webview + TS page        | in process                                               | as a C library (unverified)        | MIT               | desktop only in 2026           |
| gomobile                          | n/a                               | n/a                          | bind target only  | XCFramework library        | `.aar` library             | none; native Swift or Kotlin UI | `gomobile bind`                                          | n/a                                | BSD-3-Clause      | the Go phone library path      |
| Compose Multiplatform 1.12        | stable on the JVM                 | stable on the JVM            | stable on the JVM | stable                     | stable                     | Skia canvas, Kotlin             | gomobile on Android; C library elsewhere (unverified)    | JNI; UniFFI is MPL-2.0             | Apache-2.0        | no: JVM node, no single binary |
| Flutter 3.47                      | stable; Debian, Ubuntu LTS        | stable; 10, 11               | stable; 12 to 27  | stable; 15 to 27           | stable; API 24 to 37       | own widgets, Dart               | `c-shared` + FFI; gomobile on iOS                        | flutter_rust_bridge                | BSD-3-Clause      | runner-up; panes weak          |
| Capacitor 8.5                     | none maintained                   | none maintained              | none maintained   | stable                     | stable                     | system webview + TS page        | gomobile in a native plugin (unverified)                 | C library in a plugin (unverified) | MIT               | phone shell only               |
| Electron 44.5                     | yes                               | yes                          | yes               | none                       | none                       | Chromium + Node                 | sidecar                                                  | sidecar or N-API module            | MIT               | no: CON-03, size               |
| Electrobun 2.0                    | Ubuntu 24.04+; others community   | Windows 11                   | macOS 14+         | none                       | none                       | system webview or CEF + Bun     | sidecar                                                  | sidecar or FFI                     | MIT               | no: desktop only, LGPL engine  |
| React Native 0.87 (+ RNW, RN-mac) | community Skia only               | partner                      | partner           | stable                     | stable                     | native views, JS                | native module (unverified)                               | native module (unverified)         | MIT               | no: no Linux                   |
| Dioxus 0.7 (0.8 alpha)            | webview (wry)                     | webview (wry)                | webview (wry)     | `.ipa`; native APIs in 0.8 | `.apk`; native APIs in 0.8 | RSX on wry; Blitz WIP           | C library (unverified)                                   | in process                         | MIT OR Apache-2.0 | watch                          |

## Shapes that cover all five

Every shape keeps the hook and MCP entry as a separate core executable
in the desktop bundle, the device key in a small Swift and Kotlin
plugin, and the platform limits above. They differ in languages and in
where the panes come from.

1. **Rust core, Tauri 2 everywhere.** Option D carried to five: one
   Rust workspace for core, lane view, peer sync and seals; Tauri on
   desktop with the core binary as an `externalBin`; Tauri on phones
   with the same crates in process; the TypeScript page and its panes
   (CodeMirror merge, ghostty-web) unchanged. Two languages (Rust,
   TypeScript). Pays option D's full cost of the change
   ([comparison](comparison.md#what-the-comparison-says)). Precedent:
   NextGraph, iOS untested.
2. **Go node, Tauri shell on desktop, Tauri on the phone with Rust.**
   Options B and C stay as they are; Tauri replaces the browser tab on
   desktop. Its window either loads the Go lane view's loopback page
   ([UI note](ui-and-packaging-components.md#client-stack-per-surface))
   or relays a Go lane-view sidecar's JSON over stdio into IPC, so no
   port opens (unverified, a spike). The phone runs the same page with
   Rust code for sync and seals, held to Go's by shared test vectors
   (HC-10). Three languages; the smallest change to the SRS.
3. **Go node, Tauri on desktop and phone, Go in the phone too.** As
   shape 2, but the phone links the Go core's sync and seal packages as
   a gomobile XCFramework and `.aar` behind a small Tauri plugin, so
   seals are computed by one implementation everywhere. Rust is only
   glue. Unproven: no Tauri app with a Go library was found.
4. **Go node, Tauri desktop, native phones with gomobile.** The
   Tailscale shape: Swift and Kotlin phone UIs over the Go core,
   reduced surfaces as VIEW-14 allows, the key in the platform store
   natively. Four UI codebases' worth of phone work (two native, the TS
   page, Tauri glue), but each phone app is small (OWN-16, OWN-17).
5. **Flutter UI, Rust core.** The AppFlowy and RustDesk shape: one
   Flutter UI on five platforms over flutter_rust_bridge. Rust plus
   Dart; the terminal, diff and preview panes are rebuilt in Flutter
   widgets that are stale (xterm.dart) or new (diff viewers), and Linux
   has no maintained official webview for the preview.
6. **Flutter UI, Go core.** The Lantern and Cwtch shape: `c-shared`
   plus FFI on desktop, gomobile on phones, JSON strings across. Keeps
   CON-01 for the core; adds Dart; same pane gaps as shape 5.
7. **Tauri desktop, Flutter phone.** Block Buzz's announced shape, with
   either core. Each surface uses its strongest framework, at the price
   of two UI codebases; Buzz has not shipped its phone half yet.

Compose with a Kotlin core does not make the list: the desktop node is
a JVM app without a single binary or cross-builds, the Windows
Kotlin/Native target is Tier 3, and option F's core gaps stand.
Electron with Capacitor, React Native and Wails v3 each leave a
platform uncovered or break CON-03.

## Reading the shapes against the options

- **For the Go family (B, C),** shapes 2 to 4 keep the core, the gates
  and every ADR; the five-platform requirement adds a desktop shell and
  a phone app, not a new core language. Shape 2 is the cheapest start;
  shape 3 is the better end state if its spike passes.
- **For D,** shape 1 is the cleanest five-platform story in this
  survey: one Rust core in every app. The five-platform requirement
  strengthens D but does not change its costs.
- **Flutter** is the only framework here with stable support on all
  five and shipped five-platform products over a Rust or Go core; it
  loses on the room's panes, which are the product's centre.
- **Tauri's risks** are platform ones: WebKitGTK rendering on Linux
  (CEF in 3.0 is the escape, alpha today), WebView2's diagnostics on
  Windows, mobile plugin gaps ParleyInn fills itself, and a 3.0 migration
  already under way.

## Spikes that decide

| Spike                       | Builds                                                                                                  | Measures                                                          | Decides                       |
| --------------------------- | ------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------- | ----------------------------- |
| Tauri shell, Go node        | a notarized macOS app, a signed Windows installer and an AppImage carrying the Go core as `externalBin` | hook p95 from the bundled binary; window to Go lane view by stdio | shapes 2 to 4 on desktop      |
| Go library in a Tauri phone | gomobile XCFramework and `.aar` linked into Tauri's generated projects                                  | builds, size, App Store and Play review, 16 KB pages              | shape 3                       |
| Device key plugin           | Swift (Secure Enclave) and Kotlin (StrongBox) P-256 signing with a presence check                       | OWN-16 presence binding; key never leaves hardware                | open point 13, every shape    |
| Egress on Windows           | the room in WebView2 with SmartScreen off and custom crash reporting on                                 | every outbound connection of the app's processes                  | the I4 ruling for WebView2    |
| Linux rendering             | ghostty-web and CodeMirror merge in WebKitGTK on NVIDIA and Wayland; then the CEF runtime               | frame time, blank windows, CEF size                               | Tauri 2 against 3 on Linux    |
| Flutter panes               | the room's terminal, diff and preview in Flutter widgets                                                | parity with the TS panes; Linux preview                           | whether shapes 5 and 6 remain |
