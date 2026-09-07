# Foundation Audit — what already exists, and what HELM would actually be building

> Publication annotation, 2026-09-07: historical G0-2 wording below treats a DirectComposition
> probe as deciding an entire application class. That inference is withdrawn in the
> [current Gate 0 assessment](../experiments/EXP-009-GATE0-REPORT.md#current-assessment).
> G0-2 remains BLOCKED; vanilla's recorded `E_NOTIMPL` does not decide rendering or application
> compatibility. Original G0-3 also remains BLOCKED; the separately recorded mechanics PASS results
> do not satisfy its original criterion. Historical research text and criteria below are preserved.

| Field | Value |
|---|---|
| Document | `docs/research/FOUNDATION_AUDIT.md` |
| Status | **Draft for review, revision 2.** No architectural decision is accepted by this document. |
| Work package | HELM-003 / EXP-001 (prior-art review and reuse map) |
| Research date | 2026-09-06; corrected 2026-09-07 (see [§13](#s13)) |
| Method | Desk research against primary sources (upstream repositories, release notes, specifications, issue trackers), executed as a twelve-domain parallel review with two adversarial cross-checks. |
| Application tests executed | **None.** No installer was run, no benchmark measured, no compatibility rate produced. |
| Language | English, by explicit owner instruction, deviating from `CONTRIBUTING.md`. See [§11](#s11). |

> **How to read this.** This is a *falsification* document. Its purpose is not to show that HELM is
> a good idea; it is to find out which parts of the plan are already built by someone else, which
> parts are wrong, and which single experiment would tell us the most. Several findings contradict
> the master plan. Those are stated plainly rather than softened.

---

## Contents

1. [Executive summary](#s01)
2. [Method, evidence grading and limits](#s02)
3. [State of the foundation, dated](#s03)
4. [Reuse matrix](#s04)
5. [Where HELM has novel technical value](#s05)
6. [The motivating example, re-examined](#s06)
7. [Constraints that bind the architecture](#s07)
8. [Ten highest-risk assumptions](#s08)
9. [The smallest end-to-end proof of concept](#s09)
10. [Tensions with existing ADRs](#s10)
11. [Known defects in this audit](#s11)
12. [Sources](#s12)
13. [Corrections log](#s13)

---

<a id="s01"></a>

## 1. Executive summary

**1.1. The compatibility substrate is solved, mature and free. HELM must not rebuild any of it.**
Wine 11.0 (2026-01-13) declares the new WoW64 mode fully supported, removes the `wine64` loader and
deprecates pure `WINEARCH=win32` prefixes; NTSync is used automatically when the kernel provides it
[A01]. Proton 11.0-2 (2026-08-21), Steam Linux Runtime 4 (Debian 13 base), DXVK 3.1, vkd3d-proton
3.0.1, bubblewrap, Landlock, xdg-desktop-portal and libostree are all actively maintained and are
the correct dependencies. Rebuilding any of them is the most obvious way for this project to waste
its first year.

**1.2. Components covering four of HELM's five named pillars already exist — as capabilities, not
as an integrated, verified product.** *(Corrected in revision 2; see [§13](#s13) C-3.)*
Bottles 67.2 (2026-09-03, GPL-3.0) ships per-application environments, a dependency and verb
manager, per-bottle `bwrap` confinement, content-hashed snapshot and restore, desktop-entry
generation, and — since release 61.0 on 2026-01-24 — **"Eagle"**, which performs `pefile` and YARA
analysis of an executable, its neighbouring files and extracted installer or Electron payloads to
derive a suggested environment, with a packaged ProtonDB- and winetricks-derived SQLite knowledge
base added on 2026-07-31. That is "App Forge as analyzer and planner" as described in master plan
chapter 13, shipped eight months ago, under a licence HELM can read. Separately, `umu-protonfixes`
(BSD-2-Clause) carries several hundred per-application recipes over a roughly fifty-function
vocabulary, and Valve operates the same idea privately at commercial scale.

What those components do **not** amount to is a solved product, and revision 1 blurred that.
Read at commit `d111be2` (2026-09-07, VERIFIED_SOURCE): Eagle is a *static pre-flight analyzer*.
It never executes the target to test a suggestion — the launch callback discards the outcome;
community-derived suggestions are emitted with `apply: False` so their switches default to off;
dependency suggestions are rendered as non-interactive rows and are never installed; there is no
feedback channel, and the bundled intelligence database is opened read-only with a frozen
generation timestamp. Its own results banner reads: *"Suggestions are based on technical analysis
and do not guarantee compatibility."* Bottles' versioning is opt-in and, because the runner and
DXVK directories are siblings of the bottle rather than children, a saved state captures prefix
contents but not the runner or graphics components its own config names; restore reports success
whenever the underlying call does not raise, without checking that the application still runs. Its
per-bottle sandbox is opt-in, and on the main Wine path it inherits a default that binds the entire
host filesystem read-only into the sandbox. Neither project claims otherwise: `umu-protonfixes`
describes itself as *"a stopgap"*.

So the accurate statement is: **the capabilities exist and are shipping; the verification loop that
would turn them into a reliability guarantee does not exist in any of them.** That is a narrower
claim than revision 1 made, and it is the one the evidence supports.

**1.3. The fifth pillar is far more occupied than revision 1 claimed. Two genuine absences remain.**
*(Corrected in revision 2; see [§13](#s13) C-4 and C-5.)*

Revision 1 asserted that "no system anywhere" produces per-application, per-build, per-runtime,
per-hardware, reproducible, expiring evidence. That was an unbounded negative, and the bound
matters: **openQA already ships most of it**, and the audit had not compared against it. What
openQA does today, verified against a live production instance and against source
(VERIFIED_EXECUTION where a live job was fetched):

- **Traceability.** Every job writes a `vars.json` holding its full resolved variable set — 64
  variables on the sampled production job — including the test-code git URL and commit hash, the
  needle repository and its commit hash, worker host, worker instance and worker class, and the
  emulated CPU and memory. Secret-shaped keys are filtered at write time.
- **Artifact collection.** Per job and per module: the autoinst and worker logs, serial output,
  video with frame timings, an uploaded-logs tree, and a per-module detail file recording every
  matched *and rejected* candidate, with similarity scores and coordinates. Screenshots are stored
  content-addressed by digest.
- **A closed result vocabulary.** Seven job states and twelve job results are defined as constants,
  with a free-text reason column alongside — not an ad-hoc convention.
- **Reproducible rerun.** Cloning is first-class, with an origin relation exposed through the API,
  and `--reproduce` pins the test and needle repositories back to the original commits recorded in
  the job's own `vars.json`.
- **Non-pixel oracles.** Exit-code assertions, captured stdout, regexp or callback validation of
  that output, and ingestion of JUnit/TAP output as first-class results. A digest check on a
  produced file is expressible today.
- **Attribution, narrowly.** An investigation view automatically diffs variables, worker and
  system package manifests, and test and needle git history against the last good job; companion
  tooling clones variants to separate test regression, product regression, infrastructure and
  sporadic failure.

Against that, HELM's remainder is two genuine absences and two schema-shape fixes:

| Property | Status in openQA |
|---|---|
| A content hash identifying the exact artifact under test | **Absent.** The assets table declares a checksum column, but a census of production assets returned null for every one; digests exist for transfer integrity and screenshot storage, not for artifact identity. |
| Mechanical expiry or re-validation of a finished verdict | **Absent.** Time-based machinery is garbage collection — a result is authoritative indefinitely or deleted. Carry-over propagates a previous failure's annotation forward. |
| Immutable per-run hardware provenance | **Shape fix.** CPU model, flags, memory and worker class are recorded, but as mutable current state on the worker row, not as an immutable snapshot bound to the run. No GPU, driver or kernel version is representable. |
| Application identity and version-scoped applicability | **Shape fix.** An application is code inside a test suite acting on an image, not a modelled entity; a history-isolation mechanism already exists as the natural hook for an application-build axis. |

Everything else the evidence pillar treats as new — traceability, artifact collection,
reproducible rerun, a machine-checkable result vocabulary, and last-good attribution — has shipped
for years. What remains true, and unchanged, is that nobody runs a functional test over the
existing per-application fix corpora to check whether any of them still work.

**1.4. The motivating example's cause is unestablished — which is not the same as "it is not a Wine
problem".** *(Corrected in revision 2; see [§13](#s13) C-1 and C-2.)*

The master plan is motivated by communication applications feeling unreliable. Revision 1 concluded
that this "is not a Wine problem", reasoning partly from a bug-tracker count. That reasoning was
invalid and the count was also wrong: a WineHQ quicksearch returns five Viber bugs, not one, opened
between 2013 and 2019. More importantly, **bug-report counts do not measure usage**, and their
absence is not evidence of absence of use.

What is established, by direct inspection of the published packages (VERIFIED_EXECUTION), is
narrower and more useful:

- The current Linux build is 27.3.0.2, and its bundled Qt reports 6.10.2 — so the client is Qt/QML,
  not Electron, with a web engine bundled only for embedded content.
- The vendor's own support page states the supported set as Linux Fedora and Ubuntu (64-bit) and
  adds, verbatim, *"Regular updates and support are limited for Linux."*
- The shipped desktop entry contains no `StartupWMClass` and declares an icon name whose case does
  not match the installed icon file — a defect an independent packager works around explicitly.
- Every shipped binary is stripped, with no separate debug information published. **Any crash
  attribution will therefore resolve only exported dynamic symbols**, which is a hard constraint on
  diagnosing the user's actual symptom.

What is **not** established is the cause of the user's reported experience. No crash was reproduced,
no diagnostic artifact was collected, and no layer was ruled in. The candidate layers — the
application, the toolkit, the desktop, the graphics stack, packaging — all remain open.
See [§6](#s06) for the corrected treatment and for what would actually settle it.

**1.5. The real thesis under test is economic, not architectural.**
Every technical gap identified here is closable. What is not obviously closable is the recurring
cost of keeping a catalogue true against a substrate that ships a Wine release roughly every two
weeks, bumps its server protocol version about seventy-five times a year, and maintains **no
stable-branch backports at all** — Wine 10.0 and 11.0 have received zero point releases. The
incumbent that has solved per-application reliability did it by paying people: CodeWeavers reports
roughly 22,655 applications and 4,044 curated one-click profiles, funded by licences at $54 a year
or $494 for a lifetime licence, and states that it tests "hundreds and hundreds of Windows
programs" every month. HELM's implicit claim is that automation substitutes for that labour.
**That claim is currently unevidenced, and it is what the proof of concept must test first.**

**1.6. Recommendation.** Do not begin catalogue, GUI, OS-image or SDK work. Run a three-to-five-day
falsification gate ([§9.1](#s09)), then a single six-week end-to-end proof of concept over three
applications and two Wine builds, whose only purpose is to measure the false-pass rate of automated
evidence and the human cost of re-qualification ([§9.2](#s09)). Both outcomes are useful: a pass
funds the next phase, a failure redirects the project to a smaller and more honest promise before
any architecture is frozen.

---

<a id="s02"></a>

## 2. Method, evidence grading and limits

Twelve independent reviews were run in parallel over: Wine core; Proton and the Steam runtime;
graphics translation; existing Wine environment managers; sandboxing and permissions; packaging and
atomic update; Wayland and desktop integration; testing and evidence; the realities of real Windows
applications; communication applications; data integrity and recovery; and the competitive
landscape. Each was required to cite primary sources, to state what a claim does *not* prove, and to
grade its own confidence. Two further reviews then attacked the results: one hunting contradictions
and overclaims by re-fetching cited URLs, the other arguing that HELM has no defensible novel value
at all. Their findings are incorporated, including where they contradicted the original reports.

Evidence grades used in this document:

| Grade | Meaning |
|---|---|
| **VERIFIED_SOURCE** | Stated in a primary source that was retrieved, with the URL and, where the source moves, the pinned revision recorded. It says what the source says — not that the behaviour was observed. |
| **VERIFIED_EXECUTION** | Someone ran it and observed the result, with the command, environment and artifact recorded. |
| **LIKELY** | Strong indirect evidence, a derived measurement, or a source that could not be re-fetched. |
| **UNVERIFIED** | Reasoning or recollection. Must not be used as the basis for a decision. |

Revision 1 of this document used a single **VERIFIED** grade that conflated the first two. That
conflation is the root of several corrections in [§13](#s13): reading a declaration in source code
is not the same as observing a program behave, and this audit repeatedly treated it as if it were.
Every grade below has been re-stated in the split vocabulary; where a claim was only ever a source
reading, it now says so.

Two facts were additionally re-fetched by hand while writing this document and are first-hand
VERIFIED: the Wine 11.0 release statements on WoW64 parity, the removed `wine64` loader, deprecated
`win32` prefixes, NTSync, and the "experimental Wayland driver" [A01]; and the state of
`uiautomationcore.spec`, where `UiaGetPatternProvider` is a stub while `UiaFind`, `UiaNavigate`,
`UiaNodeFromHandle`, `UiaGetPropertyValue`, `UiaAddEvent` and `UiaGetRuntimeId` are implemented
[A02].

**Limits.** This is desk research. No application was installed, launched, benchmarked or tested. No
hardware matrix exists. No compatibility rate is stated or implied. Version numbers are
point-in-time as of 2026-09-06, and several upstream projects move every two weeks. Corpus counts
(numbers of fixes, verbs, manifests) were taken at different moments against moving branches and
**must not be quoted as stable figures** — they are order-of-magnitude signals only. Section
[§11](#s11) lists the specific claims that did not survive cross-checking.

---

<a id="s03"></a>

## 3. State of the foundation, dated

### 3.1. Wine

Wine ships one annual stable release in January plus a development release every two weeks: 11.0 on
2026-01-13, 11.17 on 2026-09-04. **There is no maintained stable branch.** Wine 8.0 received 8.0.1
and 8.0.2 and 9.0 received 9.0.1, but 10.0 and 11.0 have had no point release at all, so pinning
"the annual stable for reliability" buys a year with no bug or security fixes. Distribution
packaging is a full major version behind on the most likely bases: Debian 13 and Debian unstable
both carry 10.0, and Ubuntu 26.04 LTS carries 10.0, while Arch, Gentoo and Tumbleweed track 11.x.
Only WineHQ's own repository carries current builds. **Consequence: HELM must build, pin and ship
its own Wine, and own the re-qualification pipeline that implies.**

Prefix mechanics matter more than the plan assumes. A `WINEPREFIX` carries no version stamp and has
no migration machinery: `wineboot` compares the installed `wine.inf` modification time against
`.update-timestamp` and re-runs the INF if they differ *in either direction*, so a downgrade is
processed exactly like an upgrade. Wine's own FAQ states that Wine "does not sandbox in any way at
all". `SERVER_PROTOCOL_VERSION` moved 855 to 930 to 961 across 10.0, 11.0 and 11.17, and a mismatch
between the client and a running `wineserver` is fatal, so two Wine versions can never share a
prefix session and `wineserver -k -w` is mandatory around any version switch. The wineserver socket
directory is keyed on the prefix directory's device and inode, so relocating, bind-mounting or
containerising a prefix changes its server identity.

### 3.2. Proton and the Steam runtime

Proton 11.0-2 (2026-08-21) is Valve's Wine fork plus DXVK, vkd3d-proton, dxvk-nvapi and Wine Mono,
and it cannot run standalone: its manifest requires Steam Linux Runtime 4, a Debian 13 base.
`pressure-vessel` solves library-ABI drift by bind-mounting a pinned runtime over `/usr` while
splicing in the host's graphics drivers. It is explicitly **not** a security boundary — Valve's own
shipped known-issues document lists the user's home directory, `/home`, `/media`, `/mnt`, `/opt`,
`/run/media` and `/srv` as shared read/write by default. `umu-launcher` (GPL-3.0) reproduces
Steam's launch chain outside Steam and is now the default Proton path in both Lutris and Heroic, so
running Proton without Steam is real and adopted — though Valve documents no support for it.

Valve already operates the per-title profile registry the master plan imagines, privately: the
Steam client sets `STEAM_COMPAT_CONFIG` from server-side data for known titles, and Proton's own
launch script carries roughly 175 to 207 hard-coded application identifiers under a comment
describing them as short-lived application-specific workarounds for Proton bugs.

### 3.3. Existing managers

| Project | State on 2026-09-06 | What it already does |
|---|---|---|
| Bottles | 67.2, 2026-09-03, GPL-3.0, ~20 releases in 90 days, ~79% of recent human commits by one person | Per-application config (~35 fields), an 18-verb dependency vocabulary, runner/DXVK/VKD3D version management, per-bottle `bwrap` and `flatpak-spawn` sandboxing, content-hashed snapshot and restore, **Eagle** static analysis and suggestion engine, packaged compatibility database |
| Lutris | 0.5.22, 2026-02-25, GPL-3.0 | The largest install-script corpus in existence (~15,562 installers) over a documented 1,054-line DSL; three-level YAML config layering |
| Heroic | 2.22.1, 2026-08-09 | Versioned per-game JSON config *with a migration path*; aggregates ProtonDB, Deck and anti-cheat data; no install DSL at all |
| CrossOver | 26.3.0, 2026-07-21, proprietary | ~22,655 applications, ~4,044 one-click CrossTie profiles, install *detection* primitives no open format has, funded by $54/year or $494 licences |
| Winepak | dead, last push 2019-04-19 | The "one Flatpak per Windows application" experiment; died at roughly a dozen manifests |
| Phoenicis / PlayOnLinux 5 | dead | Enforced per-file checksums and **still died** — integrity discipline alone does not sustain a catalogue |
| Whisky (macOS) | archived 2025-05-11 at 15,116 stars | The maintainer's published post-mortem: a wrapper that ships someone else's Wine and produces no upstream fixes "contributes practically zero" |

Two structural facts follow. First, **there is no extension seam.** Bottles has no plugin API, no
published third-party schema, and archived its integration library in 2021; Lutris' only stable
extension point declares a new runner; CrossOver's runtime is closed. "Build on Bottles" is not
available as an engineering plan even where it would be the right call. Second, the only genuinely
importable asset in the field is `umu-protonfixes` together with the `umu` identifier namespace,
which both Bottles and Heroic already query at runtime.

### 3.4. Sandboxing

`bwrap` 0.12.0 (2026-08-26) removed setuid support entirely, so unprivileged user namespaces are now
required — and Ubuntu 24.04 and later restrict those through AppArmor, meaning **HELM must ship an
AppArmor profile for its own sandbox helper or fail to start on the most common target.** Flatpak's
seccomp policy unconditionally denies `unshare`, `setns`, `mount`, `pivot_root`, `chroot` and
`clone(CLONE_NEWUSER)`, so *a per-application `bwrap` sandbox nested inside a HELM Flatpak is
impossible by construction* — which is precisely why Bottles carries a second, less expressive
`flatpak-spawn` backend, and why `/dev/ntsync` silently disappears on that path. Every shipping
Wine-hosting Flatpak grants `--allow=devel --allow=multiarch --device=all --socket=x11`, which
re-enables `ptrace` and `perf_event_open`, exposes all of `/dev`, and adds an X11 socket that
Flatpak's own documentation calls "completely insecure". Firejail's eighteen CVEs, mostly local
root escalation traceable to its setuid-root design, are the architectural warning: **HELM must not
ship a setuid helper.**

Landlock is the under-used primitive here: a stackable, namespace-free, unprivileged filesystem and
network boundary that survives Ubuntu's userns restriction and works where `unshare` is denied. It
cannot restrict `mount`, does not propagate through OverlayFS and caps at sixteen stacked layers, so
it complements `bwrap` rather than replacing it.

### 3.5. Desktop integration

This is where "feels like a first-class app" is decided, and the evidence redirects blame away from
the desktop and onto Wine. `RegisterHotKey` is implemented only in the macOS driver; on both X11 and
Wayland it falls through to a null driver that **returns success without registering anything**.
`Shell_NotifyIcon` balloons are drawn as internal Win32 tooltips and never reach
`org.freedesktop.Notifications`. `IDXGIOutput::DuplicateOutput` and `MFCreateDeviceSource` are
`E_NOTIMPL` stubs, and Wine links no PipeWire and speaks no portal, so screen sharing and camera
capture are unreachable for a Windows application regardless of how its environment is configured.
`RegisterSuspendResumeNotification` is a stub, so a Windows network application never learns that it
was suspended. Meanwhile the host substrate is healthy: xdg-desktop-portal 1.22.1 exposes roughly
thirty-two interfaces including a USB portal, and GlobalShortcuts backends now exist on both GNOME
and KDE.

Wine still defaults to X11: the driver preference list is `mac,x11,wayland`, and Wine 11.0's own
release notes call the Wayland driver "experimental" [A01]. `winewayland.drv` does not bind
`xdg-activation-v1`, so a Wine application cannot raise its own window, and it binds only the
deprecated `wlr-data-control` clipboard protocol, which neither current GNOME nor current KDE
implements. **HELM v1 is an Xwayland product whether it likes it or not**, and should say so.

One counter-example is worth recording because it contradicts a common assumption: file
associations, URL handlers and dock-icon identity already work. `winemenubuilder` writes real
desktop entries with `MimeType=`, scheme handlers, and a `StartupWMClass` derived from the same
lowercased executable basename that the Wayland driver sets as its `app_id`. The usual "generic Wine
icon" complaint is a missing desktop file or a launch-by-path, not a protocol gap.

### 3.6. Testing and evidence

**Wine implements no UI Automation control patterns** (VERIFIED_SOURCE, at tag `wine-11.0`; `master`
has already drifted). Of 98 declared exports, 36 are implemented and 62 are stubs, and 59 of those
62 are control-pattern or text-range entry points — leaving only three non-pattern stubs. *(Scoped
and corrected in revision 2; see [§13](#s13) C-6.)*

Three corrections to how revision 1 used that fact:

1. **It cites the wrong layer for most of the tools it names.** The `.spec` file is the *deprecated
   flat C API*. That API is what the managed UIA2 clients bind, so the conclusion holds for those.
   The tools revision 1 actually named — pywinauto's UIA backend, the UIA3 clients, WinAppDriver —
   use the COM path, whose blocker is not visible in the spec at all: the COM client's
   `GetCurrentPattern` family returns `E_NOTIMPL`, and the module defines no pattern interfaces.
   Same conclusion, different evidence; revision 1 asserted it from a file that does not show it.
2. **The COM client is thinner than revision 1 implied.** Revision 1 said "name, type and bounding
   rectangle work". Those three do, along with tree search, the root element, element-from-handle
   and tree walking — but the automation id, class name, enabled state, native window handle,
   runtime id and `SetFocus` are also `E_NOTIMPL`. A spec entry says only whether an export has a
   body, never whether it works for a given argument.
3. **"Dead on arrival for the entire off-the-shelf ecosystem" is refuted by a shipping
   counter-example.** winetricks drives GUI installers under Wine today with AutoHotkey, pinned by
   hash, across roughly two dozen verbs using window-wait, control-click and send primitives.
   Message- and input-level automation demonstrably works under Wine. It is *pattern-based*
   automation that does not.

The correctly scoped claim is therefore: **pattern-based UIA actuation cannot succeed against
Wine's built-in implementation, on source evidence at a pinned tag; how each specific client fails
— cleanly, silently, or by crashing — is unmeasured**, and at least one shipping consumer has
already had to guard against a crash on an unimplemented UIA method. Any statement stronger than
that requires naming the Wine build, the automation backend and the application, and running it.

Corroborating prior art points the same way: Xalia's Windows backend was **migrated away from UIA**
to Win32 and MSAA, uses no control patterns, and actuates controls with window messages. Wine also
contains no AT-SPI integration, so Linux-native GUI test tools remain blind to Wine windows. The
viable technique is **find, then synthesise input**, and that is what the working prior art does.

The best prior art by a wide margin is **Xalia** (MIT, 0.4.9, commits through 2026-08-28), which
unifies Win32 messages, MSAA, UIA and AT-SPI behind a single element model and is enabled *by
default* in Proton 11. It is a gamepad-navigation overlay with no assertions, reporting or headless
mode — but its roughly thirty per-control-class Win32 providers are the honest price tag for
introspecting Windows GUIs under Wine, and it is the strongest fork-or-extend candidate in the whole
audit.

The governance model worth copying is Debian and Ubuntu's `autopkgtest`: in-tree declarative test
definitions, clean-VM execution, machine-readable results, and — the property that actually matters
— **failures that block migration**. That is why its data stays fresh while AppDB's and ProtonDB's
rot. Evidence that gates nothing decays.

### 3.7. Real Windows applications

The installer layer is in better shape than folklore suggests: Wine's MSI implementation covers the
full standard action table and all custom-action types, Inno Setup's upstream is actively
Wine-aware, and "reboot required" is a scriptable state transition via `wineboot -r`. The failures
are in what installers *do*: `powershell.exe` is a stub that prints a diagnostic and returns success
without executing anything, WMI exposes only a few dozen classes, and Authenticode verification has
three open 2026 bugs, one of which aborts Wine.

The single largest structural line is **DirectComposition**. Chromium's compositor calls
`DCompositionCreateDevice` as soon as the prefix reports Windows 8.1 or later; vanilla Wine returns
`E_NOTIMPL`, while wine-staging's `dcomp` patchset makes the sequence succeed. That gates the entire
WebView2, CEF and Electron application class — which is most modern business software. Two further
capabilities that decide whole application classes exist only in staging (Wine-export hiding and
integrity levels). **Targeting vanilla Wine alone silently removes the Chromium-embedding class.**

Realistically in reach: single-executable and Inno- or NSIS-installed Win32 and MFC applications,
WinForms and .NET Framework 4.8 or earlier on Wine Mono, well-behaved MSIs, registry-heavy
applications, and out-of-process COM. Reachable only with a specific Wine flavour and per-application
tuning: WebView2, CEF and Electron user interfaces, real-.NET WPF applications, and applications
installing per-machine services. **Structurally out of reach:** kernel-mode anti-cheat and endpoint
security agents, hardware licence dongles, networked DCOM, MSIX and WinRT packages, anything
depending on PowerShell or on Task Scheduler 2.0 actually firing, and any application whose vendor
forces a server-side auto-update that HELM has no contract to freeze.

### 3.8. Data integrity and recovery

This is the least glamorous section and the one most likely to produce a catastrophic bug.

- Nothing on Linux rolls back an application together with its user data. Flatpak, OSTree and bootc
  all revert the image and deliberately leave data alone; snapper excludes `/home` by default and
  documents that there is no mechanism to ensure data consistency when creating a snapshot; the ZFS
  desktop automation layer has been dead since 2024. Only `snapd` and Chrome's own downgrade manager
  perform per-application data rollback, both from *inside* the vendor, and both with documented
  gaps — `snapd` explicitly does not revert its shared "common" tier.
- **Wine's registry save takes a non-atomic in-place truncate path when the target is not a regular
  file or has a link count above one** — and, a precondition revision 1 omitted, only when opening
  the existing hive for writing succeeds; otherwise it falls through to the safe write-temp-then-
  rename path (VERIFIED_SOURCE at the 11.17 release commit). Revision 1 then conflated two distinct
  failures. *(Corrected in revision 2; see [§13](#s13) C-7.)*
  - **Snapshot contamination** — deterministic, silent, permanent, and requiring no fault at all.
    Writing through one hardlink writes the shared inode, so a `cp -al` "snapshot" tracks the live
    registry from that moment on, and keeps doing so, because the extra link keeps the count above
    one. A snapshot that tracks live data is neither a rollback target nor evidence of a prior
    state. **This alone disqualifies hardlink farms as snapshots.** Confirmed by execution — see the
    probe result below.
  - **Corruption of the live prefix** — requires an additional fault. The truncate happens before
    any new byte is written and there is no `fsync`, so a crash, kill or power loss inside that
    window leaves a zero-length or partial hive. Two consequences worth recording: a mid-file
    truncation *loads silently* and drops every key past the cut, while a zero-length hive fails
    loudly; and because the truncate precedes the buffered-write setup, even a local failure there
    can zero the hive with no external fault.
  - **`rsync --link-dest` was wrongly listed.** It hardlinks between the previous backup and the new
    one, leaving the live prefix's link count at one. Withdrawn.
  - Revision 1 also stated the deferral as a general property. The thirty-second periodic save and
    the absence of `fsync` hold upstream; **Proton diverges** — it has no periodic save timer and
    moves the write client-side, carrying the same hardlink hazard but not the same timing.
- **Executed here (VERIFIED_EXECUTION, 2026-09-07):** a synthetic writer on two filesystems
  reproduced the mechanic. An in-place rewrite through a hardlink made the copy track the live file
  exactly; a write-temp-then-rename writer left the copy intact and dropped the link count to one; a
  zero-length window was observable mid-write. **Reflink copies — the mechanism ADR-0017 proposed —
  were unsupported on both filesystems tested.** Artifacts:
  [ext4](../experiments/evidence/G0-3-snapshot-semantics-ext4.json),
  [drvfs](../experiments/evidence/G0-3-snapshot-semantics-drvfs.json). This tests filesystem
  mechanics with a synthetic writer; **Wine was not installed and was not executed**, so it does not
  show which path Wine's own code takes at runtime.
- Windows share modes are enforced only inside `wineserver`'s per-inode table and are invisible to
  every native Linux process, and byte-range locks degrade to advisory `fcntl` locks that Wine
  silently abandons on filesystems that reject them. So HELM's *own* integration features —
  indexers, backup, sync — would be unconstrained by the application's locking.
- The verified real-world total-loss incidents in this space are not schema-compatibility failures.
  They are disk exhaustion corrupting an embedded database, and a launcher script with an empty path
  variable in a recursive delete removing a user's entire home directory along with a mounted backup
  drive. The second is exactly the class of software this project proposes to build.

---

<a id="s04"></a>

## 4. Reuse matrix

Verdicts are **USE AS-IS** (adopt, do not modify), **EXTEND** (adopt and add a layer, or carry
patches), **REPLACE LATER** (acceptable now, expected to be replaced), and **BUILD NEW** (nothing
adequate exists). Cost is HELM's integration effort, not a judgement of the component's quality.

### 4.1. Execution substrate

| Component | Verdict | Cost | Why |
|---|---|---|---|
| Wine (upstream) | USE AS-IS | medium | No alternative at this scale. Build strictly above it; pin and ship a specific build. |
| wine-staging | EXTEND | medium | Three capabilities that decide whole application classes exist only here (DirectComposition, Wine-export hiding, integrity levels). Per-application opt-in, recorded in the profile. |
| New WoW64 / `WINEARCH=wow64` | USE AS-IS | low | Create every prefix 64-bit; never invoke the removed `wine64` [A01]. |
| ntsync | USE AS-IS | low | Nothing to build, but enablement is silent and threefold, so HELM must probe and report it. |
| wineserver | USE AS-IS | low | Not replaceable. Treat `wineserver -k -w` as a mandatory lifecycle step. |
| Wine Mono and Gecko | USE AS-IS | low | Copy their hash-pinned download pattern. Treat "this application embeds a browser engine" as a risk flag. |
| Proton | USE AS-IS | low | Adopt as an alternative runtime through umu; do not fork. |
| Steam Linux Runtime / pressure-vessel | USE AS-IS | low | Best-in-class runtime abstraction. **Never describe it as isolation.** |
| umu-launcher | USE AS-IS | low | Execute it, never link it (GPL-3.0). Vendorable; roughly two maintainers. |
| GE-Proton | REPLACE LATER | low | Useful reach today, but bus factor of one and unclear licensing. HELM must be able to run on stock Proton. |

### 4.2. Graphics

| Component | Verdict | Cost | Why |
|---|---|---|---|
| DXVK | USE AS-IS | low | No alternative for D3D8 to D3D11. HELM must pin per application and own the 2.x-versus-3.x branch decision, since Proton ships a 2.7.x branch while upstream is at 3.1. |
| vkd3d-proton | USE AS-IS | low | The only viable D3D12 path. Needs an explicit snapshot policy. |
| dxvk-nvapi | USE AS-IS | low | Without it a class of NVIDIA-targeting applications misdetects the GPU. |
| Upstream Wine vkd3d | REJECT as runtime | low | Not a Proton-grade D3D12 runtime; it must not be selected by accident. |
| WineD3D | REPLACE LATER | low | Last-resort tier for machines with no usable Vulkan driver. |
| Mesa and NVIDIA drivers | USE AS-IS | medium | Minimum versions are *per feature*, not per API. HELM needs a driver probe and a documented degrade path. |
| gamescope | EXTEND | medium | The closest existing per-application display environment, BSD-licensed — but architecturally single-surface and untested for multi-window business applications. |
| Fossilize | EXTEND | high | Format and tools are reusable; the missing part is capture distribution and invalidation. |

### 4.3. Isolation and permissions

| Component | Verdict | Cost | Why |
|---|---|---|---|
| bubblewrap | USE AS-IS | low | The correct primitive. HELM owns the *policy*, which is the thing to be reviewed. |
| Landlock | EXTEND | medium | A namespace-free per-prefix boundary that survives Ubuntu's userns restriction. HELM would be an early non-trivial desktop adopter. |
| seccomp | EXTEND | medium | Write HELM's own filter; Flatpak's development-mode split is too coarse for Wine. |
| xdg-desktop-portal | USE AS-IS | medium | Correct and unavoidable. The whole gap is on the Wine side. |
| Flatpak — as a content store | EXTEND | medium | Its OSTree storage layer gives real disk and page-cache sharing; call libostree directly. |
| Flatpak — as HELM's security boundary | REJECT | — | Nested per-application sandboxes are impossible inside it, and every Wine Flatpak already punctures the boundary. |
| Flatpak — as a GPU driver provider | REJECT | — | Runtime-built drivers structurally lag the host. |
| Flatpak — as a channel for HELM's own tooling | USE AS-IS | low | Exactly how Bottles ships. |
| AppArmor profile for HELM's helper | BUILD NEW | medium | A packaging obligation on Ubuntu, not optional hardening. |
| systemd unit sandboxing | USE AS-IS | low | Use for cgroup lifecycle and teardown; upstream states its protections are not a boundary for user services. |
| SELinux, Snap, Firejail | REJECT | — | Not viable primary boundaries. Steal exactly one idea: Snap's permission-learning mode, which Flatpak lacks, and whose absence is why so many published manifests grant all devices. |

### 4.4. Environment management and recipes

| Component | Verdict | Cost | Why |
|---|---|---|---|
| umu-protonfixes (the *vocabulary*) | USE AS-IS | low | A decade of empirical pain, BSD-2-Clause. Copy the primitive set nearly verbatim. |
| umu-protonfixes (the *data model*) | BUILD NEW | medium | Identity-keyed imperative Python with no version binding, no evidence and no schema cannot carry updates, permissions or recovery. |
| umu identifiers and database | EXTEND | medium | The only cross-launcher identity agreement that exists — but keyed on storefront SKUs. HELM needs artifact-derived identity for non-store software. |
| winetricks (the *verb knowledge*) | USE AS-IS | low | Do not reimplement several hundred verbs. |
| winetricks (the *fetch and apply machinery*) | REPLACE LATER | medium | Its integrity check is a dialog box: a hash mismatch prompts rather than aborts, and a force flag skips it. Already leaning on archival mirrors. |
| Bottles | EXTEND (in practice: study) | high | Has already built the feature surface, but offers no extension seam and has a bus factor of one at roughly a release every four and a half days. |
| Lutris installer DSL | EXTEND | medium | The best-documented prior art for scripted installation; treat it as the baseline schema rather than inventing a rival. Models installation only. |
| CrossTie install-detection tags | EXTEND | low | The one primitive both open incumbents lack: "is this installed, and at what version". Copy the idea. |
| Steam's per-title precedence model | EXTEND | low | Per-application user override beats vendor recommendation beats global default. Battle-tested; copy it. |
| Heroic's versioned per-app config | EXTEND | low | An explicit config version with a migration path — the thing the other formats lack. |

### 4.5. Storage, update and recovery

| Component | Verdict | Cost | Why |
|---|---|---|---|
| libostree (as a library) | USE AS-IS | medium | Content-addressed storage plus atomic hardlinked checkouts, applied per application environment. |
| composefs | EXTEND | high | The only component whose upstream explicitly promises page-cache sharing across mounts — but an experimental on-disk format. Budget for a migration. |
| btrfs and ZFS reflink snapshots | USE AS-IS | medium | The correct cheap copy mechanism for prefix state — with quiescence and an application-scoped manifest supplied by HELM. |
| Proton's prefix version stamp and tracked-files manifest | USE AS-IS | low | A per-prefix version stamp plus an installed-file manifest is the single most transferable artifact found in this audit. |
| Firefox-style downgrade refusal | USE AS-IS | low | Stamp the last-writing version beside the data and refuse to open newer data with an older binary. Cheap, and it converts silent corruption into a visible event. |
| Chrome's downgrade-snapshot pattern | USE AS-IS | medium | Curated manifest, database-plus-sidecar atomic grouping, N generations, and the version marker written last so a torn snapshot self-invalidates. |
| snapd's revision-scoped user data | EXTEND | medium | The existence proof that per-application data rollback ships on Linux. Note its own escape hatch: the shared tier is not reverted. |
| restic | USE AS-IS | medium | Repository layer; build application awareness above it. |
| Flatpak's application deploy and revert model | REJECT | — | Runtime references are branch-level, pinning prevents removal rather than drift, and the previous deployment is undeployed immediately — there is no local rollback. |
| SteamOS A/B image model | REJECT | — | Designed so that user modifications are destroyed on update. The opposite of HELM's promise. |
| systemd-sysext | REJECT | — | System-global, `/usr` and `/opt` only, no `/var`, no rollback, pinned to the host version. |
| Nix | EXTEND | high | Solves the runtime-closure half more completely than anything else here. Worth serious evaluation rather than dismissal. |
| greenboot's health-then-revert pattern | BUILD NEW | low | The pattern is exactly right; nothing exists at application granularity. |

### 4.6. Testing and evidence

| Component | Verdict | Cost | Why |
|---|---|---|---|
| `test.winehq.org` daily results | USE AS-IS | low | A free daily substrate-health signal, and the one WineHQ host that is not behind anti-bot protection. Not application evidence. |
| Wine TestBot orchestration | EXTEND | medium | LGPL, snapshot-per-run VMs, GPU passthrough hosts. Do not rebuild VM orchestration. |
| Xalia | EXTEND | medium | The only shipping system that introspects and drives Windows GUIs under Wine. MIT-licensed. Add assertions, reporting and a headless mode. |
| Wine `uiautomationcore` | REPLACE LATER | high | Discovery works; every control pattern is a stub [A02]. Funding pattern support upstream is a strategic option, not a v1 dependency. |
| pywinauto UIA backend, FlaUI, WinAppDriver | REJECT | — | Dead on arrival under Wine for the same reason. |
| dogtail, KDE's AT-SPI WebDriver | USE AS-IS | low | Correct for HELM's *own* Linux user interface. Structurally blind to Wine windows. |
| openQA and os-autoinst | USE AS-IS | medium | Right for OS-level and install-path evidence. Do not make image matching the primary in-application oracle — KDE abandoned needles in July 2026 for exactly that reason. |
| libei and the RemoteDesktop portal | USE AS-IS | medium | The correct long-term input layer; unattended consent remains an unsolved sub-problem. |
| `autopkgtest` governance model | EXTEND | medium | Steal the model, not the code: in-tree declarations, clean execution, machine-readable results, and evidence that gates a decision. |
| ProtonDB's fault taxonomy and open-data posture | USE AS-IS | low | A proven, familiar vocabulary. Adopt it rather than inventing one. |
| WineHQ AppDB as a data source | REJECT (renegotiate) | high | No machine interface, `robots.txt` disallows all crawlers, content licence unstated. Seeding from it is a relationship task, not an engineering task. |
| Per-application functional evidence | **BUILD NEW** | high | **Nothing exists. This is HELM's only defensible pillar.** |

### 4.7. Desktop integration

| Component | Verdict | Cost | Why |
|---|---|---|---|
| `winemenubuilder` with the desktop-entry and MIME specifications | EXTEND | low | Already largely works. HELM's value is deterministic generation *and verification*, not new mechanism. |
| Win32-to-portal bridge (file chooser, notifications, global hotkeys) | **BUILD NEW** | medium | Wine has no D-Bus desktop integration and no portal client. Each of these is a bounded, upstreamable patch. |
| Tray icon bridge to StatusNotifierItem | **BUILD NEW** | medium | Display-server independent, and works today on KDE. Waiting for a Wayland tray protocol is not a plan — the proposal has been unmerged since October 2024. |
| Suspend and resume awareness | **BUILD NEW** | low | A user-session agent on logind's sleep signal with a delay inhibitor, synthesising the Windows power-broadcast message. Never a system-sleep hook — the user slice is frozen there. |
| Desktop duplication and Media Foundation capture | **BUILD NEW** | high | Multi-month. Decide early whether video conferencing is in scope, because the answer changes the size of the project. |
| UIA-to-AT-SPI bridge | **BUILD NEW** | high (research-grade) | Would deliver accessibility *and* a semantic test oracle at once. Genuinely novel; genuinely large. |
| Cross-desktop capability probe | **BUILD NEW** | low | Record which Wayland globals and portal interfaces are actually advertised per session, and diff across host upgrades. Turns silent upstream regressions into a dated signal. |

---

<a id="s05"></a>

## 5. Where HELM has novel technical value

### 5.1. What is not novel, and must be removed from the pitch

| Claimed capability | Already shipped by |
|---|---|
| Per-application environments and prefix management | Bottles, Lutris, CrossOver, Proton with the Steam Linux Runtime |
| Automatic derivation of an environment from an executable | **Bottles "Eagle", 2026-01-24** — PE analysis, YARA rules, installer and archive extraction, hash- and import-hash-based identity |
| A per-application fix or profile registry | umu-protonfixes, Valve's private per-title configuration, CrossTie, Lutris installers |
| Content-addressed storage with disk and RAM sharing | libostree, composefs, Flatpak |
| Snapshot and restore of an application environment | Bottles' versioning subsystem, btrfs and ZFS |
| Pinned, versioned runtimes | Nix, Steam Linux Runtime, Snap |
| A per-application sandboxing mechanism | bubblewrap, Landlock, Flatpak, Bottles |
| Compatibility badges | Flathub verification, Steam Deck Verified, AppDB ratings |

### 5.2. What is genuinely unoccupied

**N1. Version-scoped applicability.** No existing format can express "this workaround applies to
application builds X through Y on runtime build Z". `protonfixes` has no build identifier, the umu
database schema has no version column, Lutris' DSL has no version predicate, and CrossTie has
install detection but no applicability range. For games this barely matters — a storefront pins the
build and titles update rarely. Business software inverts every one of those assumptions. Upstream
Wine states the failure mode directly: applying tweaks that are no longer needed can prevent an
application that now runs fine from working at all. **An unversioned recipe store for
auto-updating software is not merely incomplete; it is a mechanism for breaking working
applications, and the harm grows with catalogue size.** This cannot be retrofitted by contributing
to an existing catalogue; it is a schema decision that must precede any catalogue.

**N2. Machine-checkable, reproducible, expiring evidence.** Everything in the ecosystem either asks
a human or checks nothing. The four axes that would make HELM strictly better than the best-funded
crowdsourced dataset are precisely the four ProtonDB lacks: application build identity, normalised
hardware identity, exact runtime build, and a reproducible procedure. Add a fifth, from AppDB's
failure mode: **mechanical expiry**. AppDB's freshness mechanism is social — it removes inactive
maintainers — and nothing ever expires a result.

**N3. Attributable, tiered recovery.** "Rollback" is not novel. What is unbuilt is attributing a
regression to the axis that caused it (a runner bump, a dependency change, or a host driver update)
and reverting *only* that axis, while never reverting the user's documents. Every existing snapshot
mechanism operates at whole-prefix granularity because nobody has built the provenance to do
better.

**N4. A Win32-to-portal bridge.** Portals are pull-based: the application must call them. A Windows
application never will. So a confined Wine application can only see files HELM pre-mounts, which
forces exactly the broad grants that confinement was meant to eliminate. This is why every shipping
Wine Flatpak carries all-device access. The bridge is bounded, upstreamable, and benefits every Wine
user rather than only HELM's.

**N5. Integration conformance testing.** There is no conformance suite for StatusNotifierItem
anywhere, and its absence is the direct mechanism behind the most-reported desktop application
failure of 2026. A cross-host harness is small, buildable, and would have caught both August 2026
regressions before release.

**N6. A measured minimal permission set per Windows application.** Nobody has published one. The
missing enabler is a permission-*learning* mode, which Snap has and Flatpak does not.

### 5.3. The honest one-sentence differentiator

> Reproducible, version-scoped, expiring evidence for third-party desktop applications, published
> openly so that a third party can re-run it — plus recovery that attributes a failure to a single
> axis and never reverts the user's documents.

Every qualifier is doing work. Drop "third-party desktop applications" and Valve occupies the space.
Drop "reproducible" and ProtonDB occupies it. Drop "version-scoped" and protonfixes occupies it.
Drop "never reverts the user's documents" and the feature is dangerous rather than valuable.

---

<a id="s06"></a>

## 6. The motivating example, re-examined

The master plan is motivated by communication applications feeling unreliable, with Viber as the
named example, and it correctly warns against assuming Wine is the cause. The research resolves that
warning against the Wine hypothesis.

**Method note.** Revision 1 of this section inferred a cause. Revision 2 separates three things that
must not be merged: the **user's reported experience**, the **properties of the shipped artifact**,
and the **cause**, which remains unestablished.

*Observed by direct inspection of the published packages (VERIFIED_EXECUTION):*

- The Linux build is 27.3.0.2, dated 2026-02-19 on the vendor CDN; its bundled Qt reports 6.10.2,
  so the client is Qt/QML with a web engine bundled only for embedded content, not an Electron
  application. The Windows and macOS artifacts on the same CDN are dated later, but they are small
  downloader stubs, so that comparison bounds **artifact** freshness, not application version parity.
- The vendor states: *"Regular updates and support are limited for Linux."*
- The shipped desktop entry has no `StartupWMClass`, and its declared icon name differs in case from
  the installed icon file. What the application actually advertises as its window class at runtime
  is **not** determinable from the package and must be observed on a running system.
- All shipped binaries are stripped with no published debug information.

*Bounded, and stated as such:*

- **The tray regression cannot be the mechanism behind a Viber symptom.** The August–September 2026
  StatusNotifierItem breakage is a Chromium change inherited by Electron applications. Viber drives
  its tray through Qt's own StatusNotifierItem adaptor, and its bundled web engine contains no
  StatusNotifier code at all, so the affected code path is not present in the shipping product.
- **That regression is not documented as causing crashes.** Across the tray-specific primary
  sources, the symptom is uniformly a missing or unresponsive icon — one report states plainly that
  the process is alive and everything works except the icon. Revision 1 implied this class of defect
  explained the user's instability. It does not, and no such link was reproduced. (Scoped to the
  defect, not the documents: the same release stream separately fixes unrelated Linux crashes.)
- **The WineHQ bug count was wrong and is worthless either way.** A quicksearch returns five Viber
  bugs, not one. Revision 1 used the count to infer that nobody runs the Windows build. Bug volume
  does not measure usage in either direction, and that inference is withdrawn.
- **What would actually settle attribution**, and has not been done: reproduce the symptom; collect
  the crash artifact through the system crash handler with journal correlation; capture the
  application's own logs and the compositor log for the same window; and only then attribute a
  layer. The stripped-binary finding above sets the ceiling on how far that can go without vendor
  debug symbols.
- **A useful, cheap finding survives all of this:** shipped vendor packages contain machine-checkable
  desktop-metadata defects, with zero Wine involvement. That is verifiable from the package alone,
  and it is the part of Track B that needs no reproduction to be worth doing.

**Consequence.** There are two candidate first products, and they are not the same project:

| | Track A — Windows-application compatibility | Track B — desktop integration reliability |
|---|---|---|
| Substrate | Wine and Proton per-application environments | Native, Electron and Wine applications as they are |
| Addresses the stated user pain? | No, on this evidence | Yes, directly |
| Novelty | N1, N2, N3 above | N5, plus supervision and metadata repair |
| Incumbents | Bottles, Lutris, CrossOver, Valve | None found |
| Cost to test | Weeks | Days |
| Principal risk | The target class is *harder* than games, not easier | The target set may be evaporating toward the browser — Teams has already retired its Linux desktop client |

The proof of concept in [§9](#s09) is deliberately designed so that its falsification gate answers
this question with evidence rather than preference. **This audit does not recommend abandoning
Track A.** It recommends refusing to present Track A as the answer to the Viber problem, because on
the evidence it is not.

---

<a id="s07"></a>

## 7. Constraints that bind the architecture

These are findings a design must obey. Each has a cheap confirming test.

**C1. Never invoke `wine64`; create every prefix 64-bit with `WINEARCH=wow64`.** The loader is
removed, `win32` prefixes are deprecated and fatal on wow64-only builds, and prefix architecture is
immutable [A01].

**C2. Quiescing before a snapshot is mandatory, but `wineserver -k` is not a guaranteed clean
flush.** Protocol mismatch is fatal, so the server must be stopped around any version switch; and
registry saves are deferred (upstream: a thirty-second periodic save, no `fsync`; Proton: no
periodic timer, write moved client-side). But `-k` sends an interrupt, polls for roughly ten
seconds, then sends an uncatchable kill — which bypasses the shutdown flush entirely. **A quiesce
step must verify that the hive was actually written, not assume it.** *(Corrected in revision 2.)*

**C3. Never hardlink a prefix.** A hardlink farm is not a snapshot: an in-place rewrite through one
link makes the copy track the live data permanently, with no fault required. Confirmed by execution
on two filesystems ([§3.8](#s03)). Use full copies, or reflinks **where the filesystem supports
them** — which neither filesystem available on the audit host did, so this is a precondition to
check, not an assumption. *(Corrected in revision 2.)*

> **Revision 1's proposed ten-minute test for this was invalid and is withdrawn.** It diffed two
> hardlinks to the same inode, which are identical by construction whether contamination occurred
> or not, and it diffed `system.reg` while an application setting change lands in `user.reg`. A
> corrected procedure is specified as G0-3 in [EXP-009](../experiments/EXP-009.md): compare a
> **content copy** taken before the write against the copy under test, and cover both hives.

**C4. HELM's per-application state must live in `HKCU\Software\Wine` or outside the prefix.** A Wine
upgrade re-runs `wine.inf` and overwrites a large fraction of registry values, while the
`HKCU\Software\Wine` subtree and installed native DLLs survive. (Graded LIKELY — see [§11](#s11).)

**C5. The sandbox helper runs on the host, not inside a Flatpak,** and must ship an AppArmor profile
for Ubuntu 24.04 and later. Nested per-application sandboxes inside Flatpak are impossible by
construction. Landlock is the namespace-free second layer.

**C6. HELM v1 is an Xwayland product.** Wine defaults to X11, and its Wayland driver is experimental
and lacks window activation and a working clipboard protocol on both major desktops [A01]. The X11
socket is then the largest remaining hole in an otherwise tight policy. Say so publicly rather than
implying a Wayland-native experience.

**C7. Capability detection must be explicit and reported.** ntsync enablement is silent and
threefold; DXVK 3.x needs specific Mesa and driver versions; DirectComposition exists only in
staging. All of these fail silently or cryptically. A health probe is a v1 requirement, not a
nicety.

**C8. Pinned hashes conflict with how vendors actually ship.** Rolling "latest" installer URLs are
the norm, and some runtime bootstrappers have no stable hash by design. The correct primitive is
probably vendor signature verification rather than hash pinning — but Authenticode verification
inside Wine has three open 2026 bugs. Measure before designing.

**C9. Deletion must be driven by an installer-generated manifest, never by a computed recursive
path,** and every update must check and reserve free space first. The highest-probability
catastrophic bugs in this domain are disk exhaustion mid-write and a recursive delete with an empty
variable.

**C10. Publish the structurally unsupportable class up front:** kernel-mode anti-cheat and endpoint
security agents, hardware dongles, networked DCOM, MSIX and WinRT, PowerShell-dependent installers,
and Task Scheduler 2.0. Announcing coverage before knowing which of these are impossible is how a
project ends up promising what it cannot win.

---

<a id="s08"></a>

## 8. Ten highest-risk assumptions

Ranked by impact if false, multiplied by probability of being false, divided by the cost of finding
out. Each is falsifiable cheaply, and each test is small enough to run before any architecture is
frozen.

| # | Assumption | Why it is risky | Cheapest decisive test |
|---|---|---|---|
| **R-A1** | The user's problem — communication applications feeling unreliable — is caused by the Windows-compatibility layer. | The evidence says no ([§6](#s06)). If false, HELM's motivating example is the one thing a Wine-centric architecture cannot help, and the first product is the wrong product. | For each of five motivating applications, answer from primary sources: does a native Linux build exist, and can Wine install the Windows build today? Attempt one MSIX install. **1 day.** |
| **R-A2** | Automation can substitute for the per-application QA labour that CodeWeavers and Valve pay for. | This is the actual product thesis and it is unevidenced. Both funded incumbents kept humans in the loop. If false, HELM is a well-designed catalogue that rots — the observed fate of Winepak and Phoenicis. | Define the weakest evidence signal HELM would be willing to publish, build it for exactly three applications, and measure the false-pass rate by hand. **2 weeks.** |
| **R-A3** | Business and productivity applications are an easier target than games. | The evidence points the other way: WebView2 and CEF gated on staging-only DirectComposition, WPF rendering defects on both .NET paths, per-machine services, Authenticode bugs that abort installers, dongles, DCOM. Games have DXVK, Proton, ProtonDB and Valve's money; business applications have CrossOver's private QA and nothing else. | Classify twenty target applications on five axes from the binaries alone — UI toolkit, .NET target, installer engine, licensing mechanism, declared services and tasks — before installing anything. **2 days.** |
| **R-A4** | A pinned, hashed application profile stays valid over time. | False by construction for a growing share of targets: self-updaters replace the application in place, some runtime bootstrappers have no stable hash, and Wine ships roughly twenty-six releases a year while upstream warns that stale tweaks break working applications. | Hash three prefixes daily for thirty days with normal network access — one self-updating Electron application, one embedding a web runtime, one classic Win32. Separately, retest ten workarounds that were required on Wine 9.x against 11.17 with and without the tweak. **30 days elapsed, about 2 days of work.** |
| **R-A5** | Automated environment derivation from the executable is HELM's novel contribution. | Bottles shipped it on 2026-01-24, with PE analysis, YARA rules, installer extraction and a packaged knowledge base. Building it again wastes the scarcest resource and invites an accurate "this already exists" review. | Install Bottles 67.x, run Eagle over three target applications, and record what it detected, what it suggested, and whether applying the suggestions produced a working application. **1 afternoon.** |
| **R-A6** | Per-application confinement is a solved building block HELM can simply adopt. | It is the newest and least settled part of the stack. The leading implementation's host path exposes the whole root filesystem read-only, and its 67.2 release — three days before this audit — fixed a live host-root exposure. Windows applications also actively resist confinement. | Write a small Windows binary that enumerates and attempts to read and write outside the prefix, and run it under each shipping configuration. The resulting escape matrix is both the answer and HELM's first permanent regression test. **2 days.** |
| **R-A7** | A filesystem snapshot of the prefix is a sufficient safety net for user data. | Three independent failures: crash-consistent is not application-consistent; hardlink schemes actively corrupt Wine's registry; and deferred, unflushed registry saves mean a snapshot of a running prefix can be missing the last half-minute of writes. | The `cp -al`, forty-second, `diff system.reg` test (C3). **10 minutes.** Then snapshot a prefix mid-write and run an integrity check on every embedded database in the restored copy. |
| **R-A8** | A community will contribute and maintain enough tested application profiles. | The base rate is bad: Winepak died at roughly a dozen manifests, Bottles reached about thirty-eight curated installers in five years, and Phoenicis died *despite* enforcing checksums. The only large tested catalogue was produced by a company charging for licences. | Publish the profile schema and ask for contributions for thirty days. Fewer than twenty usable non-game recipes falsifies the volunteer model cheaply. **30 days, near-zero cost.** |
| **R-A9** | HELM can rely on upstream Wine to fix the application-specific failures it finds. | Upstream declines support for third-party prefix managers until a failure is reproduced on plain Wine, has a single merge gatekeeper, is heavily concentrated in one company that is also the direct commercial competitor, and publishes no stable-branch backports. | Reduce one real failure to a plain-Wine reproduction, file it upstream, and measure triage and fix latency. **One bug report.** |
| **R-A10** | The reuse targets are extensible rather than fork-only. | Bottles has no plugin API, archived its integration library in 2021, and ships roughly one release every four and a half days with about 79% of recent human commits from one person. An integration built on today's shapes is a treadmill. | Diff the configuration model, the dependency action vocabulary and the CLI subcommands across three release tags spanning eight months, and count breaking changes. **1 hour.** |

Also-rans, deliberately not in the top ten but recorded so they are not forgotten: that
content-addressed deduplication makes N per-application runtimes nearly free in RAM (the only
published measurement is disk-only and shows roughly a quarter shared); that Flatpak already
provides pinned runtimes with rollback (it provides neither); that Wayland-native is the right
target (it currently makes application-shaped software worse); and that AppDB can be mined to
bootstrap the catalogue (it cannot, legally or technically).

---

<a id="s09"></a>

## 9. The smallest end-to-end proof of concept

### 9.1. Gate 0 — falsification before construction (three to five days, no product code)

Each check has a pre-declared meaning. **Gate 0 runs first, and its results are published whatever
they say.**

| ID | Check | If it fails |
|---|---|---|
| G0-1 | Answer R-A1: for five communication applications, does a native Linux build exist, and can Wine install the Windows build? Attempt one MSIX install. | The motivating example is not a Wine problem. Re-open the Track A / Track B decision in [§6](#s06) before any architecture is written. |
| G0-2 | Run the minimal DirectComposition probe attached to the upstream bug report against vanilla Wine, wine-staging and Proton. | If only staging succeeds, the Chromium-embedding class requires a staging-based runtime. Record it as a dependency and a maintenance cost, or exclude the class publicly. |
| G0-3 | The hardlink registry-corruption test (C3). | Confirms that hardlink snapshots are unsafe. Any design based on them is dead; use reflinks. |
| G0-4 | Install Bottles 67.x and run Eagle over three target applications. | Establishes the true state of the art and the size of the accuracy gap. If Eagle's suggestions already produce working applications, automatic derivation leaves HELM's novelty list permanently. |
| G0-5 | Escape harness: a Windows binary attempting access outside the prefix, run under each shipping Wine sandbox configuration. | Produces the escape matrix and HELM's first permanent regression test. |

**Cost ceiling:** one person, five days, one machine, no new spend. If exceeded, stop and report
rather than continuing.

### 9.2. PoC-1, "Evidence Loop" (six weeks, one person, one machine)

**Thesis under test, in one sentence:**

> For a small set of Windows desktop applications, an automated system can produce reproducible,
> version-scoped evidence that a specific application build works on a specific runtime build,
> detect when a runtime change breaks it, attribute the breakage to one axis, restore the last
> known-good state without losing user data, and do so at a per-application maintenance cost that
> one person can sustain.

**Scope.** Three applications, two Wine builds (11.0 and 11.17), one host, one GPU, one desktop
session. Command-line and file-output oracles first; a deliberately narrow GUI probe second.

**Explicitly out of scope:** any catalogue, any GUI shell, any OS image, the SDK, the Windows VM,
AI features, more than one hardware profile, and any application requiring a purchased licence or a
vendor account.

**Application selection** — three different oracle classes, all obtainable without an account:

| Slot | Class | Proposed candidate | Oracle |
|---|---|---|---|
| A1 | Pure Win32, installer-based, deterministic file output | 7-Zip (Windows build) | Compress and extract a fixed corpus; assert hash round-trip equality and archive integrity. **No GUI automation at all.** |
| A2 | Win32 GUI, document-integrity workflow | Notepad++ (Inno Setup) | Open a UTF-8 file at a non-ASCII path, edit it, save it through synthesised input, then assert exact bytes, encoding and line endings on disk. **This is the GUI-cost measurement.** |
| A3 | The hard class, gated on G0-2 | one Chromium-embedding or .NET WinForms application | Launch to an interactive window and complete one scripted task; primarily a measurement of where the wall is. |

The design deliberately puts a *deterministic, non-visual* oracle in slot A1. The research shows the
GUI oracle problem is the genuinely hard part — Wine implements no UIA control patterns [A02], Wine
exposes nothing on AT-SPI, and the strongest Linux QA teams are moving away from image matching. So
the proof of concept measures how far honest evidence gets *without* solving that problem, and
prices the GUI half separately instead of assuming it.

**The loop to build — this is the whole deliverable:**

```text
identify   (artifact hash + PE/MSI metadata)
  -> plan     (runtime build, verbs, overrides, permissions)   [shown to the user, approved]
  -> stage    (bubblewrap + Landlock, host-side, no root)
  -> install  (logged and observed; produces an ownership manifest)
  -> verify   (run mandatory workflows, assert, record an evidence record)
  -> pin      (application build x runtime build x host capability)
        ...
  -> bump one axis (Wine 11.0 -> 11.17)
  -> re-verify -> detect regression -> attribute the axis -> revert that axis only
  -> re-verify green, with user documents untouched
```

**Pre-registered success criteria.** All six must hold.

| ID | Criterion | Threshold |
|---|---|---|
| P1 | **Reproducibility** | A second person, on a clean machine, following only the written recipe, reproduces every evidence verdict for A1 and A2 with **zero** undocumented manual steps. A manual DLL override that is not in the recipe counts as a defect, not a workaround. |
| P2 | **Oracle quality** | Over twenty consecutive runs per application: **false-pass rate of zero**, flake rate at most 5% for A1 and at most 20% for A2, with every flake explained. Measured by injecting five known breakages per application; the harness must fail on at least four of five. |
| P3 | **Regression detection** | The 11.0 to 11.17 bump and one deliberately broken build are both detected within one scheduled run, with the failing workflow named. |
| P4 | **Attribution** | For at least two of three injected multi-axis changes (runtime build, dependency verb, host driver), the system names the correct single axis. |
| P5 | **Data safety** | Rollback restores the last known-good state and re-verifies green, **and** a user document created after the known-good snapshot survives byte-identical, in ten trials out of ten. |
| P6 | **Cost** | Onboarding application number three takes at most eight hours of human time; re-qualifying all three after a Wine bump takes at most one hour of human time. |

**Pre-registered failure and redirect criteria.** Any one of these triggers a written decision, not a
silent retry:

| ID | Observation | Consequence |
|---|---|---|
| F1 | False-pass rate above zero on the weakest signal | The evidence pillar is redesigned around version-stamped human attestation. Publishing a signal that says "works" when it does not is worse than publishing nothing, because it will be cited. |
| F2 | A2's GUI oracle costs more than three person-days to author, or flakes above 20% | Narrow the public claim to "we verify only what we can assert deterministically", and price GUI verification separately. |
| F3 | Any user document lost or mutated by rollback | Stop. The data-safety design is wrong and no catalogue may be built until it is fixed (see [§3.8](#s03), C3 and C9). |
| F4 | Re-qualification takes more than four hours of human time per Wine bump for three applications | The extrapolated catalogue cost is unaffordable. At roughly twenty-six releases a year, this is the number that decides whether HELM can be a catalogue at all. |
| F5 | G0-2 shows the target class needs staging-only features HELM cannot maintain | Publish the scope boundary and shrink the target class before promising anything. |

**Budget and stop conditions.** One person, six calendar weeks, one machine, no new spend, no second
hardware profile. Stop and report on exceeding any of these rather than continuing.

**Artifacts required at the end, whatever the outcome:** the harness source; exact versions of every
component; the recipes; every evidence record including failures and repeated attempts; the injected
breakages and whether each was caught; the measured human-time log; and a written
`supported` / `not_supported` / `inconclusive` verdict on the thesis sentence above.

### 9.3. Missing decisions that block execution

None of these may be invented to make the plan look complete.

1. The base distribution and its exact version, and whether the session is Wayland or X11 — this
   affects C6 and every integration result.
2. The exact physical machine or machines, GPU vendor and driver version. "One machine" is a valid
   answer; an unstated one is not.
3. Whether the owner accepts English for new documents, and whether `CONTRIBUTING.md` is amended
   accordingly.
4. A named owner, a named reviewer, and who may accept an ADR.
5. The Track A versus Track B decision in [§6](#s06), which Gate 0 informs but does not make.
6. Whether the project will fund or perform upstream Wine work — the answer sets the maximum
   reliability HELM can ever deliver.
7. Licence policy, still open, and now with additional inputs: GPL-3.0 on `umu-launcher` and the umu
   database, share-alike obligations on any ProtonDB-derived compatibility data, unclear licensing on
   some community Proton builds, and unresolved redistribution terms for Microsoft runtimes.

---

<a id="s10"></a>

## 10. Tensions with existing ADRs

**No existing ADR is modified by this document.** The following tensions are recorded so that the
owner can decide. Each is addressed by a new *proposed* ADR rather than by editing an existing one.

| Existing | Tension | Proposed handling |
|---|---|---|
| [ADR-0003](../adr/ADR-0003-upstream-first.md) — reuse upstream, limit forks | HELM must ship its own pinned Wine build, and the Chromium-embedding class needs wine-staging patches. That is a standing delta from upstream. | [ADR-0013](../adr/ADR-0013-pinned-wine-runtime.md) keeps upstream-first as the *patch* policy while making the *build* HELM's own, with a recorded patch inventory. |
| [ADR-0004](../adr/ADR-0004-appforge-orchestration.md) — App Forge as analyzer, planner, packager, validator | The analyzer and planner halves are shipped by Bottles. Presenting them as HELM's contribution is no longer defensible. | [ADR-0014](../adr/ADR-0014-version-scoped-profiles.md) moves the novelty to version scoping and evidence, and explicitly cedes derivation. |
| [ADR-0002](../adr/ADR-0002-compatibility-metrics.md) — three separate metrics | Sound, but silent on expiry, reproducibility and hardware granularity — the axes on which every incumbent dataset fails. | [ADR-0015](../adr/ADR-0015-evidence-expiry.md) extends it rather than replacing it. |
| [ADR-0005](../adr/ADR-0005-sandbox-boundary.md) — a prefix is not a security boundary | Confirmed and strengthened, but it does not say *where* the boundary runs. | [ADR-0016](../adr/ADR-0016-host-side-sandbox.md) places it on the host, using bubblewrap plus Landlock. |
| [ADR-0006](../adr/ADR-0006-versioned-runtime-lifecycle.md) — versioned runtime lifecycle | Compatible, but written before it was known that there is no upstream stable branch to pin to. | [ADR-0013](../adr/ADR-0013-pinned-wine-runtime.md) supplies the missing mechanism. |
| [ADR-0007](../adr/ADR-0007-reuse-desktop-first.md) — reuse the existing desktop | Compatible, but the audit shows the integration gaps are mostly inside Wine, not in the desktop. | [ADR-0018](../adr/ADR-0018-win32-portal-bridge.md) targets the Wine side. |
| [ADR-0011](../adr/ADR-0011-vm-separate.md) — the VM is a separate optional path | Unchanged. Worth noting only that the remote-application integration technique is proven prior art if that path is ever taken. | No new ADR. |

---

<a id="s11"></a>

## 11. Known defects in this audit

Stated so that a reviewer does not have to rediscover them.

1. **Corpus counts drift.** Fix counts, verb counts and manifest counts were taken at different
   moments against moving branches and disagreed between reviewers — for example 311 versus 312
   Steam fixes in the same week. None was pinned to a commit hash. **Do not quote any of these
   numbers as stable figures.** Re-derive them at a pinned tree hash before using them in a design
   document.
2. **The `wine.inf` upgrade analysis is LIKELY, not VERIFIED.** The claim that a specific number of
   registry values survive a Wine upgrade is a reviewer's own static parse of the INF source, never
   confirmed by executing an upgrade, and the parsing script was not published. The derived rule
   (C4) is sound regardless and can be adopted now; the numbers should not be quoted.
3. **Commit and authorship statistics are LIKELY.** The concentration of Wine development is well
   known and the conclusion is almost certainly right, but the cited URLs cannot support
   commit-range statistics.
4. **Some Steam runtime documentation was read from a third-party fork** of files dated 2018 to 2022,
   because the canonical host blocks automated fetching. The default-shared filesystem list is
   confirmed from a current Valve-hosted document; the claim that per-application home directories
   are a dormant feature is UNVERIFIED and may be up to eight years stale. If Valve has resumed that
   work, part of HELM's permissions gap closes without HELM.
5. **The claim that Microsoft runtimes cannot legally be mirrored is UNVERIFIED, and it was used to
   justify an architectural conclusion.** Microsoft's current documentation describes conditional
   permission, not prohibition. This must be settled with counsel before the dependency-acquisition
   layer is designed, because a licensed mirror would change the design from "pin a hash and hope"
   to "hard-fail hashes plus our own mirror".
6. **Snap's data rollback is narrower than first reported:** revision-scoped data reverts, the shared
   common tier does not. The corrected conclusion — that there is *no* shipped precedent for
   atomically rolling back an application together with all of its user data — is stronger, not
   weaker.
7. **Viber's support page could not be re-fetched while writing this document** (HTTP 403). The
   statement that the vendor documents limited Linux updates and support is carried from the
   original research and from the master plan's own source register, and is graded LIKELY here.
8. **One cross-checking reviewer received only half of the domain reports** because of a size limit,
   and said so. Its "nobody covered this" claims were re-checked against the full set where they
   mattered; the UI Automation finding survived that check and was then verified first-hand [A02].
9. **No enterprise Windows-migration tooling market was surveyed.** The claim that no system produces
   reproducible per-application evidence is a negative result from a bounded search; a proprietary
   internal tool could exist.
10. **Not covered at all, and material:** audio behaviour under Wine (device enumeration, exclusive
    mode, latency), which is the most common "it launches but is unusable" failure and would be
    scored as a pass by a naive smoke test; printing and scanning; complex-script input; per-monitor
    and fractional scaling; and whether restoring a snapshot consumes or invalidates a software
    licence activation. That last one could turn HELM's flagship feature into its worst bug and must
    be tested before recovery is promised.

---

<a id="s12"></a>

## 12. Sources

Sources verified first-hand while writing this document:

| ID | Source | Used for |
|---|---|---|
| A01 | [Wine 11.0 release announcement][A01] | New WoW64 feature parity; removal of the `wine64` loader; deprecation of `WINEARCH=win32` prefixes; NTSync; the "experimental Wayland driver". |
| A02 | [Wine `uiautomationcore.spec`][A02] | `UiaGetPatternProvider` is a stub, while `UiaFind`, `UiaNavigate`, `UiaNodeFromHandle`, `UiaGetPropertyValue`, `UiaAddEvent` and `UiaGetRuntimeId` are implemented. |

The wider evidence base — several hundred primary-source URLs across the twelve domains — belongs
with the EXP-001 report as reviewable research artifacts rather than being duplicated here. The
master document's own source register remains at
[master plan, chapter 42](../../HELM_MASTER_PLAN.md#s42), and the claims register at
[CLAIMS_REGISTER.md](CLAIMS_REGISTER.md). The prior-art map this document fills in is
[PRIOR_ART.md](PRIOR_ART.md), and the executable plan derived from [§9](#s09) is
[EXP-009](../experiments/EXP-009.md).

[A01]: https://raw.githubusercontent.com/wine-mirror/wine/wine-11.0/ANNOUNCE.md
[A02]: https://raw.githubusercontent.com/wine-mirror/wine/master/dlls/uiautomationcore/uiautomationcore.spec

---

<a id="s13"></a>

## 13. Corrections log

Revision 2, dated 2026-09-07. Every change is recorded here **before** any Gate 0 results were
collected, so that no acceptance criterion can be adjusted after seeing an outcome. Revision 1's
text is not deleted silently: each entry names what it said, why it was wrong, and what replaced it.

The single systematic error behind most of these: revision 1 used one **VERIFIED** grade for both
"I read this in a source" and "this was observed to behave this way". [§2](#s02) now splits that
into VERIFIED_SOURCE and VERIFIED_EXECUTION.

| ID | What revision 1 said | Why it was wrong | Revision 2 |
|---|---|---|---|
| **C-1** | "WineHQ's tracker holds exactly one Viber bug", used to conclude nobody runs the Windows build. | Two faults. The count is wrong — a quicksearch returns five. And the inference is invalid regardless: **bug-report volume does not measure usage**, in either direction. | Count corrected; the inference withdrawn entirely. [§1.4](#s01), [§6](#s06). |
| **C-2** | Implied that the August–September 2026 StatusNotifierItem regression explained the user's Viber instability. | Never reproduced, and mechanically impossible: the regression is a Chromium code path, while Viber drives its tray through Qt's own adaptor and its bundled web engine contains no StatusNotifier code. The documented symptom in tray-specific sources is a missing or unresponsive icon, not a crash. | Link withdrawn; symptom scoped to the defect rather than to the sources; cause recorded as **unestablished**, with the diagnostic procedure that would settle it. [§6](#s06). |
| **C-3** | "Four of HELM's five named pillars are already shipped by an incumbent." | Conflated *a component with capability X exists* with *the problem is solved as a product*. Read at a pinned commit, the analyzer never executes the target, its suggestions default to off, its dependency suggestions are never installed, it has no feedback channel, and its own banner disclaims any compatibility guarantee. | Restated: the capabilities exist and are shipping; **the verification loop does not exist in any of them**. [§1.2](#s01). |
| **C-4** | "No system anywhere produces per-application, per-build, per-runtime, per-hardware, reproducible, expiring evidence." | An unbounded negative asserted without comparing against the obvious incumbent. **openQA already ships** per-job input recording with pinned test-code commits, per-module artifact collection, a closed result vocabulary, clone-and-reproduce rerun, non-pixel oracles, and last-good attribution. | Claim replaced with a per-property comparison table against openQA. [§1.3](#s01). |
| **C-5** | Treated the whole evidence pillar as HELM's novel contribution. | Overstated by roughly half. | Narrowed to **two genuine absences** (artifact content-hash identity; mechanical verdict expiry) and **two schema-shape fixes** (immutable per-run hardware provenance; application identity and version-scoped applicability). Everything else is conceded to openQA. [§1.3](#s01), [§5.2](#s05). |
| **C-6** | "Wine implements no UIA control patterns, therefore the entire off-the-shelf Windows GUI-automation ecosystem is dead on arrival under Wine." | The source observation is real but was cited from the *deprecated flat C API*, which is not the layer most of the named tools use; the COM blocker is elsewhere and not visible in that file. Revision 1 also overstated what does work, and the universal claim is refuted by a shipping counter-example — winetricks drives GUI installers under Wine with AutoHotkey today. | Scoped to a pinned tag and to pattern-based actuation specifically; the counter-example recorded; per-client failure modes marked **unmeasured**. [§3.6](#s03). |
| **C-7** | "Hardlink snapshots corrupt the live prefix and the snapshot together." | Conflated two different failures with different likelihoods and different consequences. Also omitted a precondition in the source path, and wrongly listed `rsync --link-dest`, which does not raise the live file's link count. | Split into **contamination** (deterministic, silent, needs no fault — and disqualifying on its own) and **corruption** (needs an additional fault). `rsync --link-dest` withdrawn. Proton's divergent timing recorded. [§3.8](#s03), C3 in [§7](#s07). |
| **C-8** | "`wineserver -k -w` is a mandatory lifecycle step", implying it guarantees a clean flush. | It escalates to an uncatchable kill after roughly ten seconds, bypassing the shutdown flush. | Quiesce must **verify** the hive was written rather than assume it. C2 in [§7](#s07). |
| **C-9** | Revision 1's ten-minute test for the hardlink claim. | Invalid by construction: it diffed two hardlinks to the same inode — always identical — and inspected the wrong hive for the change it induced. | Withdrawn and replaced with a corrected G0-3 procedure in [EXP-009](../experiments/EXP-009.md). |
| **C-10** | Reflink snapshots presented as the available safe mechanism. | **Refuted by execution on this host:** neither available filesystem supported reflink copies. | Reflink is now a *precondition to check*, not an assumption; ADR-0017 carries a correction note. [§3.8](#s03). |

### 13.1. What is unchanged, and deliberately so

Several revision 1 findings survived an explicit attempt to weaken them, and are restated rather
than softened:

- **Pattern-based UIA actuation is unusable under Wine.** The scoping in C-6 narrows the evidence,
  not the conclusion.
- **Hardlink farms must never be used as prefix snapshots.** C-7 makes the reason *stronger*:
  contamination is deterministic and needs no crash, so the practice fails even on a machine that
  never faults.
- **No functional verification exists over the per-application fix corpora.** Unchanged.
- **The economic thesis remains the project's central unevidenced claim.** Unchanged, and the
  openQA finding sharpens it: if most of the evidence machinery already exists, the remaining
  question is even more clearly about sustained curation cost than about engineering.

### 13.2. Corrections still owed

Recorded so they are not lost: revision 2 does not re-examine the master plan's remaining
quantitative claims, the corpus counts flagged in [§11](#s11) are still unpinned, and the audit
still contains no coverage of audio, printing, complex-script input, scaling, or licence
activation under snapshot restore.
