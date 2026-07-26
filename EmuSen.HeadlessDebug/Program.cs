using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.Debug;
using EmuSen.Shell;

// Headless AI-agent-driven debugging harness - the "not yet built" item
// from Man pages/EmuSen_Core_Gameplan.md §1's backlog. Every investigation
// in this project so far (including the Yoshi/coin WRAM-staging one) has
// required a human to run F4 prompt commands during a live play session
// and manually relay the console output back. This loads a ROM, runs the
// real VenusCore for a fixed number of frames with no window/audio/human
// involved, then feeds a script of the exact same debug commands the F4
// prompt accepts (ShellInterpreter.Execute doesn't care where a
// command line comes from - see that class's own comment) and writes the
// results to a plain log file.
//
// Usage:
//   dotnet run -- <rom> <frames> [--watch space:addr:len[:kind]]... [--script path] [--out path] [--tap frame:button[:duration]]... [--tap2 frame:button[:duration]]... [--loadstate path] [--savestate path] [--screenshot frame:path]...
//   dotnet run -- <rom> <maxframes> --commands path [other flags above except --tap/--tap2/--screenshot/--script]
//
// --watch registers an extra watch before the run starts (kind is
// write/read/both, default write) - space/addr/len match `watch add`'s own
// arguments. --script is a text file of newline-separated debug commands
// (anything ShellInterpreter understands - `watch log`, `disasm`,
// `writers`, etc.) run once after the frame loop finishes; if omitted, a
// default script just dumps every registered watch's full event log,
// which is exactly what the Yoshi/coin investigation needs. --out mirrors
// all output to a file in addition to stdout. --tap holds a button down
// for <duration> frames (default 4) starting at frame <frame> - a
// headless run otherwise sends no input at all, which is fine for a game
// with its own idle-timeout demo (SMW) but leaves others (LttP's
// file-select screen) stuck at a "press Start" prompt forever. --loadstate
// loads a VenusCore.SaveState() file (same format the RaylibFrontend's
// F5/F9 hotkeys and its F4 prompt's own `state save|load` command use)
// before frame 0, for starting directly from an already-reached scene
// instead of re-deriving it via --tap every run. --savestate writes one
// out after the frame loop finishes, to capture a moment for reuse later.
// --screenshot dumps an uncompressed BMP of GetFrameBufferRgba() right
// after the given frame runs - the only way to actually see what a
// headless run reached, short of guessing from RAM addresses and sprite
// dumps alone.
//
// --autoshot <dir> works alongside every mode above (classic frame loop
// and --commands both): instead of guessing a frame number and hoping it
// landed on something interesting - the exact "screenshot every N frames,
// eyeball it, adjust" cycle the SMAS Select Game investigation spent many
// rounds on before --commands existed - it hashes GetFrameBufferRgba()
// every frame and only writes a BMP when the hash actually changes from
// the last saved one, to <dir>/frame_<n>.bmp. Cheap enough for a debugging
// tool (one FNV-1a pass over a ~230KB buffer/frame) even though it means
// paying GetFrameBufferRgba()'s existing per-frame allocation cost on
// every frame rather than only the ones a caller explicitly asked for.
//
// --commands is a fundamentally different mode from all of the above:
// instead of pre-declaring every tap/screenshot by frame number before the
// run starts (fine when the exact timing is already known, unworkable for
// open-ended exploration), it reads an ordered script and executes each
// line as it's reached, so frame-stepping, input, screenshots, and debug
// commands can interleave freely in one process. Investigating Super Mario
// All-Stars' Select Game menu needed a separate full relaunch (reboot +
// replay the whole boot sequence) for every single button guess - this
// collapses an entire investigation into one script, one process, one
// `--out` log. Lines:
//   frames <n>              - advance <n> frames, applying whatever's currently held
//   tap[2] <button> [dur]   - press (tap2 = controller 2) for <dur> frames (default 4), release, same as --tap/--tap2 but inline
//   hold/release <button> [controller]  - set a button's held state without advancing any frames (controller defaults to 1)
//   screenshot <path>       - capture the current frame right now, not tied to a frame number
//   waitstable [maxframes=300] [quietframes=10] - advance one frame at a time until the
//                             framebuffer hash stops changing for <quietframes> in a row (or
//                             <maxframes> is hit) - removes the remaining "run N frames and
//                             hope it settled" guesswork plain `frames` still needs.
//   contactsheet <path> <count> [every=1] [cols=8] [scale=4] - capture <count> frames spaced
//                             <every> apart, downsample each by <scale>, tile into one grid
//                             image - for "is this actually moving" questions across a span of
//                             frames without reviewing N separate screenshots one at a time.
//   vramsheet <path>        - export the current tile/character memory as a BMP (via
//                             IDebugTarget.RenderTileSheet() - core-agnostic; a core with
//                             nothing analogous returns a 0x0 empty image).
//   paletteswatch <path>    - export the current color palette memory as a BMP grid (via
//                             IDebugTarget.RenderPaletteSwatch(), same core-agnostic contract).
//   spriteoverlay <path>    - capture the current frame and draw a green bounding-box outline
//                             for every entry IDebugTarget.GetSprites() reports (already
//                             core-agnostic, no interface change needed for this verb).
//   audiodump <path> [maxsamples] - write whatever's currently buffered in
//                             IDebugTarget.GetAudioSamples() (non-destructive - it never
//                             dequeues) out as a standard 16-bit PCM .wav file.
//   anything else           - passed straight to ShellInterpreter.Execute, same as --script
// <frames> is still required and still means what it always did in every
// other mode - here it becomes a hard safety cap (a script's `frames`
// requests refuse to advance past it) so a typo can't hang the process
// indefinitely. --tap/--tap2/--screenshot/--script are ignored when
// --commands is given; everything else (--watch, --flag, --loadstate,
// --savestate, --verbose, --cpulog, --out) still applies normally.
//
// A separate standalone mode, unrelated to running a ROM at all:
//   dotnet run -- --diffshot <bmp1> <bmp2> <outpath>
// Reads back two of this harness's own BMPs (screenshot/autoshot/contact
// sheet output - not arbitrary external images), highlights every
// differing pixel in magenta over a dimmed copy of the second frame, and
// prints the changed-pixel count/percentage and bounding box - answering
// "what specifically changed" between two frames without eyeballing them
// side by side or improvising an image-diff script per investigation.
class Program
{
    static int Main(string[] args)
    {
        // Standalone utility mode - no ROM/core involved, so it's checked
        // before the usual <rom> <frames> positional-argument validation
        // below even looks at args[0].
        if (args.Length >= 1 && args[0] == "--diffshot")
        {
            if (args.Length < 4)
            {
                Console.WriteLine("Usage: dotnet run -- --diffshot <bmp1> <bmp2> <outpath>");
                return 1;
            }
            return RunDiffShot(args[1], args[2], args[3]);
        }

        if (args.Length < 2)
        {
            Console.WriteLine("Usage: dotnet run -- <rom> <frames> [--watch space:addr:len[:kind]]... [--script path] [--out path]");
            return 1;
        }

        string romPath = args[0];
        if (!File.Exists(romPath))
        {
            Console.WriteLine($"[ERROR] ROM not found: {romPath}");
            return 1;
        }
        if (!long.TryParse(args[1], out long frameCount) || frameCount <= 0)
        {
            Console.WriteLine($"[ERROR] Invalid frame count: {args[1]}");
            return 1;
        }

        var extraWatches = new List<string>();
        string? scriptPath = null;
        string? commandsPath = null;
        string? autoshotDir = null;
        string? outPath = null;
        string? loadStatePath = null;
        string? saveStatePath = null;
        bool verbose = false;
        long cpuLogStart = -1, cpuLogEnd = -1;
        var flagsToEnable = new List<string>();
        var taps = new List<(long Start, long End, EmuSen.Cores.Nintendo.Venus.Controllers.SnesButton Button, int Controller)>();
        var screenshots = new List<(long Frame, string Path)>();
        for (int i = 2; i < args.Length; i++)
        {
            if (args[i] == "--flag" && i + 1 < args.Length)
            {
                // Sets any DebugSettings.<Name> field/property by
                // reflection - dozens of bool *Logging flags exist (one per
                // investigation that ever needed a targeted trace, e.g.
                // WindowHdmaLogging) plus the occasional non-bool sidecar
                // (e.g. ColorMathBlendScanline, which scopes
                // ColorMathBlendLogging's output to one scanline) - and
                // hardcoding a CLI switch per flag isn't worth it when this
                // harness's whole point is not needing a rebuild per
                // investigation. Plain "Name" sets a bool to true (the
                // common case); "Name=value" parses value as whatever
                // that member's actual type is. MasterLoggingEnabled is set
                // alongside every one of these since each individual
                // *Logging flag's own getter is gated by that master switch
                // (see DebugSettings' own comment) - setting one without
                // the other is a silent no-op that looks identical to
                // "nothing happened."
                flagsToEnable.Add(args[++i]);
            }
            else if (args[i] == "--cpulog" && i + 1 < args.Length)
            {
                // startFrame:endFrame - windows DebugSettings.CpuVerboseLogging
                // to just the frames given, instead of the F4 prompt's
                // instruction-count-based `trace` command (which has no way
                // to be armed for a future frame in a script that isn't
                // interactive). Every instruction's disassembly prints
                // straight to Console.Out, same as the DMA logging --verbose
                // already enables, so it shows up in --out like everything
                // else - just very high-volume, hence windowing it tightly.
                string[] p = args[++i].Split(':');
                cpuLogStart = long.Parse(p[0]);
                cpuLogEnd = long.Parse(p[1]);
            }
            else if (args[i] == "--watch" && i + 1 < args.Length) extraWatches.Add(args[++i]);
            else if (args[i] == "--script" && i + 1 < args.Length) scriptPath = args[++i];
            else if (args[i] == "--commands" && i + 1 < args.Length) commandsPath = args[++i];
            else if (args[i] == "--autoshot" && i + 1 < args.Length) autoshotDir = args[++i];
            else if (args[i] == "--out" && i + 1 < args.Length) outPath = args[++i];
            else if (args[i] == "--verbose") verbose = true;
            else if (args[i] == "--loadstate" && i + 1 < args.Length) loadStatePath = args[++i];
            else if (args[i] == "--savestate" && i + 1 < args.Length) saveStatePath = args[++i];
            else if (args[i] == "--screenshot" && i + 1 < args.Length)
            {
                // frame:path - dumps GetFrameBufferRgba() as an uncompressed
                // BMP right after that frame runs, since a headless run has
                // no window to look at otherwise. BMP (not PNG) specifically
                // because it needs zero compression/encoding logic - just a
                // header in front of the same top-down-flipped RGBA bytes
                // ICore already hands back.
                string[] p = args[++i].Split(new[] { ':' }, 2);
                screenshots.Add((long.Parse(p[0]), p[1]));
            }
            else if (args[i] == "--tap" && i + 1 < args.Length)
            {
                // frame:button[:durationFrames] - a scripted button press,
                // since a headless run otherwise has no input at all and
                // several games (LttP's file-select screen, unlike SMW's
                // own idle-timeout demo) never progress past a "press
                // Start" prompt without one.
                string[] p = args[++i].Split(':');
                long start = long.Parse(p[0]);
                var button = Enum.Parse<EmuSen.Cores.Nintendo.Venus.Controllers.SnesButton>(p[1], ignoreCase: true);
                long duration = p.Length >= 3 ? long.Parse(p[2]) : 4;
                taps.Add((start, start + duration, button, 1));
            }
            else if (args[i] == "--tap2" && i + 1 < args.Length)
            {
                // Same as --tap but for controller 2 - added specifically to
                // test the widely-reported real-world quirk where Super
                // Mario All-Stars' classic NES-style games (SMB1/2/3, unlike
                // native SMW) read Player 2's controller instead of
                // Player 1's.
                string[] p = args[++i].Split(':');
                long start = long.Parse(p[0]);
                var button = Enum.Parse<EmuSen.Cores.Nintendo.Venus.Controllers.SnesButton>(p[1], ignoreCase: true);
                long duration = p.Length >= 3 ? long.Parse(p[2]) : 4;
                taps.Add((start, start + duration, button, 2));
            }
        }

        // Off by default (matches DebugSettings.MasterLoggingEnabled's own
        // default) - only flip it on when explicitly asked, since it turns
        // on every individually-enabled *Logging flag's live console spam
        // (DmaSourceAddrLogging etc.), useful for a short, targeted run but
        // far too noisy over a long one.
        if (verbose) EmuSen.Debug.DebugSettings.MasterLoggingEnabled = true;

        foreach (string flagSpec in flagsToEnable)
        {
            string[] parts = flagSpec.Split(new[] { '=' }, 2);
            string flagName = parts[0];
            string? rawValue = parts.Length >= 2 ? parts[1] : null;

            var settingsType = typeof(EmuSen.Debug.DebugSettings);
            var prop = settingsType.GetProperty(flagName);
            var field = prop == null ? settingsType.GetField(flagName) : null;
            Type? memberType = prop?.PropertyType ?? field?.FieldType;

            if (memberType == null)
            {
                Console.WriteLine($"[WARN] No DebugSettings.{flagName} property or field found - ignoring --flag {flagSpec}.");
                continue;
            }

            object value = rawValue == null ? true : Convert.ChangeType(rawValue, memberType);
            if (prop != null) prop.SetValue(null, value);
            else field!.SetValue(null, value);

            EmuSen.Debug.DebugSettings.MasterLoggingEnabled = true;
        }

        var log = new List<string>();
        void Emit(string line)
        {
            Console.WriteLine(line);
            log.Add(line);
        }

        Emit($"[ROM] Loading: {romPath}");
        var core = new VenusCore(headless: true);
        core.LoadRom(romPath);

        // Jumping straight to a saved moment (e.g. Link already standing
        // in a room) sidesteps blindly scripting menu-navigation input
        // with --tap, which only works when the exact input timing is
        // already known.
        if (loadStatePath != null)
        {
            if (!File.Exists(loadStatePath))
            {
                Emit($"[ERROR] State file not found: {loadStatePath}");
                return 1;
            }
            core.LoadState(loadStatePath);
            Emit($"[STATE] Loaded: {loadStatePath}");
        }

        var debugTarget = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
        var debugCmd = ShellInterpreter.CreateDefault(debugTarget);

        // Same two ranges registered from power-on in RaylibFrontend's
        // Program.cs for the Yoshi/coin investigation - duplicated here
        // rather than shared, since this entry point has no window/input
        // loop to hang that wiring off of and needs them active before the
        // very first frame regardless.
        debugTarget.Watches.AddWatch("WRAM", 0x8000, 0x1800);
        debugTarget.Watches.AddWatch("WRAM", 0x0D80, 0x0080);

        foreach (string spec in extraWatches)
        {
            string[] parts = spec.Split(':');
            if (parts.Length < 3)
            {
                Emit($"[WARN] Ignoring malformed --watch '{spec}' (expected space:addr:len[:kind])");
                continue;
            }
            string cmd = $"watch add {parts[0]} {parts[1]} {parts[2]}" + (parts.Length >= 4 ? $" {parts[3]}" : "");
            Emit(debugCmd.Execute(cmd));
        }

        const int progressEvery = 600; // ~10s of real 60fps gameplay

        // Shared between both frame-loop modes below (only one of which
        // actually runs per invocation) - see this file's own header
        // comment on --autoshot for why this exists.
        ulong? lastAutoshotHash = null;
        if (autoshotDir != null) Directory.CreateDirectory(autoshotDir);

        void CheckAutoshot(long frameNum)
        {
            if (autoshotDir == null) return;
            byte[] frame = core.GetFrameBufferRgba();
            ulong hash = Fnv1aHash(frame);
            if (lastAutoshotHash == hash) return;
            lastAutoshotHash = hash;
            string path = Path.Combine(autoshotDir, $"frame_{frameNum}.bmp");
            WriteBmp(path, frame, core.ScreenWidth, core.ScreenHeight);
            Emit($"[AUTOSHOT] Frame {frameNum} changed -> {path}");
        }

        // --commands takes over the whole run - see this file's own header
        // comment for the script syntax and why this exists (collapsing an
        // entire multi-guess investigation into one process/one log
        // instead of a full relaunch per experiment).
        if (commandsPath != null)
        {
            if (!File.Exists(commandsPath))
            {
                Emit($"[ERROR] Commands file not found: {commandsPath}");
                return 1;
            }

            long currentFrame = 0;
            var held = new Dictionary<(EmuSen.Cores.Nintendo.Venus.Controllers.SnesButton Button, int Controller), bool>();

            void ApplyHeld()
            {
                foreach (var kv in held) core.Bus!.Input.SetButton(kv.Key.Button, kv.Value, kv.Key.Controller);
            }

            void RunFrames(long count)
            {
                for (long i = 0; i < count; i++)
                {
                    if (currentFrame >= frameCount)
                    {
                        Emit($"[WARN] Hit the {frameCount}-frame safety cap - ignoring the rest of this 'frames' request.");
                        return;
                    }
                    ApplyHeld();
                    if (cpuLogStart >= 0)
                    {
                        bool inWindow = currentFrame >= cpuLogStart && currentFrame < cpuLogEnd;
                        EmuSen.Debug.DebugSettings.MasterLoggingEnabled = inWindow || verbose;
                        EmuSen.Debug.DebugSettings.CpuVerboseLogging = inWindow;
                    }
                    core.RunFrame();
                    currentFrame++;
                    CheckAutoshot(currentFrame);
                    if (currentFrame % progressEvery == 0) Emit($"[frame {currentFrame}/{frameCount}]");
                }
            }

            // Advances one frame at a time (reusing RunFrames(1) so held
            // input/cpuLog windowing/autoshot all still apply exactly as
            // they would for a plain `frames` line) until the framebuffer
            // hash stops changing for <quietFrames> in a row, or
            // <maxFrames> is reached - whichever comes first.
            //
            // Requires seeing at least one real change before a quiet
            // streak counts as "settled" - a real bug caught testing this
            // against the actual SMAS investigation: called right after a
            // `tap Start` while still sitting on the static Nintendo boot
            // logo, the naive "N identical frames in a row" version
            // reported stable after only ~16 frames, because the logo
            // itself doesn't animate and was already "stable" the instant
            // it was checked - long before the tap's own transition had
            // even started, let alone finished. It can't tell "hasn't
            // reacted yet" apart from "finished reacting" without this.
            long WaitStable(long maxFrames, long quietFrames)
            {
                ulong? lastHash = null;
                long quietCount = 0;
                long stepped = 0;
                bool sawChange = false;
                while (stepped < maxFrames && currentFrame < frameCount)
                {
                    RunFrames(1);
                    stepped++;
                    ulong hash = Fnv1aHash(core.GetFrameBufferRgba());
                    if (lastHash.HasValue && hash != lastHash.Value) sawChange = true;
                    if (hash == lastHash) quietCount++;
                    else { quietCount = 0; lastHash = hash; }
                    if (sawChange && quietCount >= quietFrames) break;
                }
                return stepped;
            }

            Emit($"[COMMANDS] Running {commandsPath} (safety cap {frameCount} frames)...");
            Emit("");
            Emit("=== Command output ===");

            foreach (string rawLine in File.ReadAllLines(commandsPath))
            {
                string cmdLine = rawLine.Trim();
                if (cmdLine.Length == 0 || cmdLine.StartsWith('#')) continue;

                string[] parts = cmdLine.Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries);
                string verb = parts[0].ToLowerInvariant();

                if (verb == "frames" && parts.Length >= 2)
                {
                    Emit($"> {cmdLine}");
                    RunFrames(long.Parse(parts[1]));
                }
                else if ((verb == "tap" || verb == "tap2") && parts.Length >= 2)
                {
                    Emit($"> {cmdLine}");
                    int controller = verb == "tap2" ? 2 : 1;
                    var button = Enum.Parse<EmuSen.Cores.Nintendo.Venus.Controllers.SnesButton>(parts[1], ignoreCase: true);
                    long duration = parts.Length >= 3 ? long.Parse(parts[2]) : 4;
                    held[(button, controller)] = true;
                    RunFrames(duration);
                    held[(button, controller)] = false;
                    ApplyHeld();
                }
                else if ((verb == "hold" || verb == "release") && parts.Length >= 2)
                {
                    Emit($"> {cmdLine}");
                    var button = Enum.Parse<EmuSen.Cores.Nintendo.Venus.Controllers.SnesButton>(parts[1], ignoreCase: true);
                    int controller = parts.Length >= 3 ? int.Parse(parts[2]) : 1;
                    held[(button, controller)] = verb == "hold";
                    ApplyHeld();
                }
                else if (verb == "screenshot" && parts.Length >= 2)
                {
                    WriteBmp(parts[1], core.GetFrameBufferRgba(), core.ScreenWidth, core.ScreenHeight);
                    Emit($"[SCREENSHOT] Frame {currentFrame} -> {parts[1]}");
                }
                else if (verb == "waitstable")
                {
                    long maxFrames = parts.Length >= 2 ? long.Parse(parts[1]) : 300;
                    long quietFrames = parts.Length >= 3 ? long.Parse(parts[2]) : 10;
                    Emit($"> {cmdLine}");
                    long stepped = WaitStable(maxFrames, quietFrames);
                    Emit($"[WAITSTABLE] Advanced {stepped} frame(s) (cap {maxFrames}, quiet threshold {quietFrames}) - now at frame {currentFrame}.");
                }
                else if (verb == "contactsheet" && parts.Length >= 3)
                {
                    // <path> <count> [every=1] [cols=8] [scale=4] - captures
                    // <count> frames spaced <every> apart, downsamples each
                    // by <scale>, tiles them into one grid image. Built for
                    // "is this actually animating/moving" questions once
                    // past a menu and into real gameplay, where N separate
                    // --screenshot/screenshot calls would mean N separate
                    // file reads to review instead of one.
                    Emit($"> {cmdLine}");
                    string path = parts[1];
                    int count = int.Parse(parts[2]);
                    long every = parts.Length >= 4 ? long.Parse(parts[3]) : 1;
                    int cols = parts.Length >= 5 ? int.Parse(parts[4]) : 8;
                    int scale = parts.Length >= 6 ? int.Parse(parts[5]) : 4;

                    var thumbs = new List<byte[]>();
                    int thumbW = 0, thumbH = 0;
                    for (int i = 0; i < count; i++)
                    {
                        RunFrames(every);
                        thumbs.Add(Downsample(core.GetFrameBufferRgba(), core.ScreenWidth, core.ScreenHeight, scale, out thumbW, out thumbH));
                    }
                    WriteContactSheet(path, thumbs, thumbW, thumbH, cols);
                    Emit($"[CONTACTSHEET] {count} frame(s), every {every}, {thumbW}x{thumbH} each -> {path}");
                }
                else if (verb == "vramsheet" && parts.Length >= 2)
                {
                    // Delegates to IDebugTarget.RenderTileSheet() rather
                    // than reaching into the core's Renderer directly -
                    // works the same regardless of which core is loaded,
                    // per the standing core-agnostic instruction.
                    Emit($"> {cmdLine}");
                    var (rgba, w, h) = debugTarget.RenderTileSheet();
                    WriteBmp(parts[1], rgba, w, h);
                    Emit($"[VRAMSHEET] {w}x{h} -> {parts[1]}");
                }
                else if (verb == "paletteswatch" && parts.Length >= 2)
                {
                    Emit($"> {cmdLine}");
                    var (rgba, w, h) = debugTarget.RenderPaletteSwatch();
                    WriteBmp(parts[1], rgba, w, h);
                    Emit($"[PALETTESWATCH] {w}x{h} -> {parts[1]}");
                }
                else if (verb == "spriteoverlay" && parts.Length >= 2)
                {
                    // Draws a bounding-box outline for every active sprite
                    // IDebugTarget.GetSprites() reports directly onto the
                    // current frame buffer - no new interface method needed
                    // since GetSprites() was already generic/core-agnostic.
                    Emit($"> {cmdLine}");
                    byte[] rgba = core.GetFrameBufferRgba();
                    var sprites = debugTarget.GetSprites();
                    foreach (var s in sprites)
                    {
                        DrawSpriteOutline(rgba, core.ScreenWidth, core.ScreenHeight, s);
                    }
                    WriteBmp(parts[1], rgba, core.ScreenWidth, core.ScreenHeight);
                    Emit($"[SPRITEOVERLAY] {sprites.Count} sprite(s) outlined -> {parts[1]}");
                }
                else if (verb == "audiodump" && parts.Length >= 2)
                {
                    Emit($"> {cmdLine}");
                    int maxSamples = parts.Length >= 3 ? int.Parse(parts[2]) : int.MaxValue;
                    var (samples, sampleRate) = debugTarget.GetAudioSamples();
                    if (samples.Length > maxSamples) samples = samples[..maxSamples];
                    WriteWav(parts[1], samples, sampleRate);
                    Emit($"[AUDIODUMP] {samples.Length} sample(s) @ {sampleRate}Hz -> {parts[1]}");
                }
                else
                {
                    Emit($"> {cmdLine}");
                    Emit(debugCmd.Execute(cmdLine));
                }
            }

            core.Cpu?.FlushVerboseTrace();
            core.Spc700?.FlushVerboseTrace();
            Emit($"[RUN] Done, {core.TotalFrames} total frames executed.");

            if (saveStatePath != null)
            {
                core.SaveState(saveStatePath);
                Emit($"[STATE] Saved: {saveStatePath}");
            }

            if (outPath != null)
            {
                File.WriteAllLines(outPath, log);
                Console.WriteLine($"[OUT] Wrote log to {outPath}");
            }

            return 0;
        }

        Emit($"[RUN] Executing {frameCount} frames...");
        for (long frame = 0; frame < frameCount; frame++)
        {
            foreach (var tap in taps)
            {
                bool pressed = frame >= tap.Start && frame < tap.End;
                core.Bus!.Input.SetButton(tap.Button, pressed, tap.Controller);
            }
            if (cpuLogStart >= 0)
            {
                // CpuVerboseLogging's own getter is gated by
                // MasterLoggingEnabled (same pattern as DmaVerboseLogging -
                // see --verbose's comment above), so both need setting for
                // this window to actually produce output.
                bool inWindow = frame >= cpuLogStart && frame < cpuLogEnd;
                EmuSen.Debug.DebugSettings.MasterLoggingEnabled = inWindow || verbose;
                EmuSen.Debug.DebugSettings.CpuVerboseLogging = inWindow;
            }
            core.RunFrame();
            CheckAutoshot(frame);
            if (frame > 0 && frame % progressEvery == 0)
            {
                Emit($"[frame {frame}/{frameCount}]");
            }
            foreach (var shot in screenshots)
            {
                if (shot.Frame == frame)
                {
                    WriteBmp(shot.Path, core.GetFrameBufferRgba(), core.ScreenWidth, core.ScreenHeight);
                    Emit($"[SCREENSHOT] Frame {frame} -> {shot.Path}");
                }
            }
        }
        core.Cpu?.FlushVerboseTrace();
        core.Spc700?.FlushVerboseTrace();
        Emit($"[RUN] Done, {core.TotalFrames} total frames executed.");

        // Captures whatever scene --tap/frame-count navigation just
        // reached, so a promising moment (found once, maybe after a lot of
        // trial and error) can be jumped back to instantly with
        // --loadstate on every later run instead of re-deriving it.
        if (saveStatePath != null)
        {
            core.SaveState(saveStatePath);
            Emit($"[STATE] Saved: {saveStatePath}");
        }

        List<string> commands;
        if (scriptPath != null)
        {
            if (!File.Exists(scriptPath))
            {
                Emit($"[ERROR] Script not found: {scriptPath}");
                return 1;
            }
            commands = File.ReadAllLines(scriptPath)
                .Select(l => l.Trim())
                .Where(l => l.Length > 0 && !l.StartsWith('#'))
                .ToList();
        }
        else
        {
            commands = new List<string> { "watch list" };
            commands.AddRange(debugTarget.Watches.GetWatches().Select(w => $"watch log {w.Id} 500"));
        }

        Emit("");
        Emit("=== Command output ===");
        foreach (string cmd in commands)
        {
            Emit($"> {cmd}");
            Emit(debugCmd.Execute(cmd));
        }

        if (outPath != null)
        {
            File.WriteAllLines(outPath, log);
            Console.WriteLine($"[OUT] Wrote log to {outPath}");
        }

        return 0;
    }

    // 64-bit FNV-1a over the raw RGBA bytes - used by --autoshot to decide
    // whether the current frame differs from the last one it saved,
    // without needing a full pixel-by-pixel comparison. Not
    // cryptographically anything; a frame-to-frame "did this change at
    // all" check has no adversarial input to worry about, and FNV-1a's
    // avalanche behavior is more than enough to make two visually
    // different SNES frames collide by chance a non-concern in practice.
    private static ulong Fnv1aHash(byte[] data)
    {
        const ulong FnvOffsetBasis = 14695981039346656037;
        const ulong FnvPrime = 1099511628211;

        ulong hash = FnvOffsetBasis;
        foreach (byte b in data)
        {
            hash ^= b;
            hash *= FnvPrime;
        }
        return hash;
    }

    // Minimal uncompressed 32bpp BMP writer - no case for PNG's DEFLATE
    // needed just to look at a frame. BMP rows are stored bottom-up and
    // BGRA rather than RGBA, both handled by walking rgba backwards a row
    // at a time and swapping R/B per pixel; everything else about the
    // format is a fixed-size header.
    private static void WriteBmp(string path, byte[] rgba, int width, int height)
    {
        int rowSize = width * 4;
        int imageSize = rowSize * height;
        int fileSize = 54 + imageSize;

        using var fs = new FileStream(path, FileMode.Create);
        using var w = new BinaryWriter(fs);

        w.Write((byte)'B'); w.Write((byte)'M');
        w.Write(fileSize);
        w.Write(0); // reserved
        w.Write(54); // pixel data offset

        w.Write(40); // DIB header size
        w.Write(width);
        w.Write(height);
        w.Write((short)1); // planes
        w.Write((short)32); // bits per pixel
        w.Write(0); // no compression
        w.Write(imageSize);
        w.Write(2835); w.Write(2835); // ~72 DPI
        w.Write(0); w.Write(0); // colors used/important

        for (int y = height - 1; y >= 0; y--)
        {
            int rowStart = y * rowSize;
            for (int x = 0; x < width; x++)
            {
                int i = rowStart + x * 4;
                w.Write(rgba[i + 2]); // B
                w.Write(rgba[i + 1]); // G
                w.Write(rgba[i + 0]); // R
                w.Write(rgba[i + 3]); // A
            }
        }
    }

    // Reads back exactly what WriteBmp writes - the fixed 54-byte header,
    // bottom-up BGRA rows - since --diffshot only ever needs to read this
    // harness's own screenshots/autoshots/contact sheets back, not
    // arbitrary externally-authored BMPs.
    private static (byte[] Rgba, int Width, int Height) ReadBmp(string path)
    {
        using var fs = new FileStream(path, FileMode.Open, FileAccess.Read);
        using var r = new BinaryReader(fs);

        fs.Position = 10;
        int dataOffset = r.ReadInt32();
        fs.Position = 18;
        int width = r.ReadInt32();
        int height = r.ReadInt32();

        fs.Position = dataOffset;
        int rowSize = width * 4;
        byte[] rgba = new byte[rowSize * height];
        for (int y = height - 1; y >= 0; y--)
        {
            int rowStart = y * rowSize;
            for (int x = 0; x < width; x++)
            {
                int i = rowStart + x * 4;
                rgba[i + 2] = r.ReadByte(); // B
                rgba[i + 1] = r.ReadByte(); // G
                rgba[i + 0] = r.ReadByte(); // R
                rgba[i + 3] = r.ReadByte(); // A
            }
        }
        return (rgba, width, height);
    }

    // Nearest-neighbor downsample - a debugging contact sheet needs
    // "can I tell this changed shape/position," not photographic
    // fidelity, so there's no reason to pull in a real resampling filter
    // for this.
    private static byte[] Downsample(byte[] src, int srcWidth, int srcHeight, int scale, out int dstWidth, out int dstHeight)
    {
        dstWidth = Math.Max(1, srcWidth / scale);
        dstHeight = Math.Max(1, srcHeight / scale);
        byte[] dst = new byte[dstWidth * dstHeight * 4];

        for (int y = 0; y < dstHeight; y++)
        {
            int srcY = Math.Min(srcHeight - 1, y * scale);
            for (int x = 0; x < dstWidth; x++)
            {
                int srcX = Math.Min(srcWidth - 1, x * scale);
                int srcIdx = (srcY * srcWidth + srcX) * 4;
                int dstIdx = (y * dstWidth + x) * 4;
                Array.Copy(src, srcIdx, dst, dstIdx, 4);
            }
        }
        return dst;
    }

    // Tiles a list of equally-sized RGBA thumbnails into one grid image,
    // <cols> per row - empty trailing cells in the last row stay whatever
    // `new byte[]`'s zero-fill default is (opaque black, since alpha is
    // also 0... actually fully transparent black, harmless either way for
    // a debugging aid).
    private static void WriteContactSheet(string path, List<byte[]> thumbs, int thumbWidth, int thumbHeight, int cols)
    {
        int count = thumbs.Count;
        int rows = (count + cols - 1) / cols;
        int sheetWidth = cols * thumbWidth;
        int sheetHeight = rows * thumbHeight;
        byte[] sheet = new byte[sheetWidth * sheetHeight * 4];

        for (int i = 0; i < count; i++)
        {
            int originX = (i % cols) * thumbWidth;
            int originY = (i / cols) * thumbHeight;
            byte[] thumb = thumbs[i];
            for (int y = 0; y < thumbHeight; y++)
            {
                int srcRowStart = y * thumbWidth * 4;
                int dstRowStart = ((originY + y) * sheetWidth + originX) * 4;
                Array.Copy(thumb, srcRowStart, sheet, dstRowStart, thumbWidth * 4);
            }
        }

        WriteBmp(path, sheet, sheetWidth, sheetHeight);
    }

    // Draws a rectangle outline (not filled - a filled box would hide the
    // very sprite pixels you're trying to locate) directly into an RGBA
    // buffer. Works from IDebugTarget.DebugSpriteInfo alone, so this has
    // no idea what console produced it - same core-agnostic split as
    // every other harness verb here.
    private static void DrawSpriteOutline(byte[] rgba, int width, int height, EmuSen.Shell.DebugSpriteInfo s)
    {
        void SetPixel(int x, int y)
        {
            if (x < 0 || x >= width || y < 0 || y >= height) return;
            int i = (y * width + x) * 4;
            rgba[i] = 0; rgba[i + 1] = 255; rgba[i + 2] = 0; rgba[i + 3] = 255; // green
        }

        for (int x = s.X; x < s.X + s.Width; x++)
        {
            SetPixel(x, s.Y);
            SetPixel(x, s.Y + s.Height - 1);
        }
        for (int y = s.Y; y < s.Y + s.Height; y++)
        {
            SetPixel(s.X, y);
            SetPixel(s.X + s.Width - 1, y);
        }
    }

    // Minimal uncompressed PCM WAV writer, mirroring WriteBmp's style -
    // no external audio library needed just to inspect what the DSP's
    // buffer currently holds. Samples are already interleaved L/R 16-bit
    // PCM (see IDebugTarget.GetAudioSamples), so this is a fixed 44-byte
    // header plus the raw sample bytes, nothing more.
    private static void WriteWav(string path, short[] samples, int sampleRate)
    {
        const int channels = 2;
        const int bitsPerSample = 16;
        int byteRate = sampleRate * channels * bitsPerSample / 8;
        int blockAlign = channels * bitsPerSample / 8;
        int dataSize = samples.Length * sizeof(short);

        using var fs = new FileStream(path, FileMode.Create);
        using var w = new BinaryWriter(fs);

        w.Write(new[] { 'R', 'I', 'F', 'F' });
        w.Write(36 + dataSize);
        w.Write(new[] { 'W', 'A', 'V', 'E' });

        w.Write(new[] { 'f', 'm', 't', ' ' });
        w.Write(16); // fmt chunk size
        w.Write((short)1); // PCM
        w.Write((short)channels);
        w.Write(sampleRate);
        w.Write(byteRate);
        w.Write((short)blockAlign);
        w.Write((short)bitsPerSample);

        w.Write(new[] { 'd', 'a', 't', 'a' });
        w.Write(dataSize);
        foreach (short sample in samples) w.Write(sample);
    }

    // Standalone utility mode - operates purely on two already-rendered
    // BMP files, no ROM/core involved at all, so it's dispatched before
    // Main even looks at the usual <rom> <frames> positional arguments.
    // Highlights every differing pixel in magenta over a dimmed/grayed
    // copy of the second frame, so the change stands out at a glance
    // instead of needing two screenshots held side by side.
    private static int RunDiffShot(string path1, string path2, string outPath)
    {
        var (rgbaA, widthA, heightA) = ReadBmp(path1);
        var (rgbaB, widthB, heightB) = ReadBmp(path2);
        if (widthA != widthB || heightA != heightB)
        {
            Console.WriteLine($"[ERROR] Size mismatch: {path1} is {widthA}x{heightA}, {path2} is {widthB}x{heightB}");
            return 1;
        }

        byte[] outRgba = new byte[rgbaA.Length];
        int changedCount = 0;
        int minX = widthA, minY = heightA, maxX = -1, maxY = -1;

        for (int y = 0; y < heightA; y++)
        {
            for (int x = 0; x < widthA; x++)
            {
                int i = (y * widthA + x) * 4;
                bool changed = rgbaA[i] != rgbaB[i] || rgbaA[i + 1] != rgbaB[i + 1] || rgbaA[i + 2] != rgbaB[i + 2];
                if (changed)
                {
                    changedCount++;
                    if (x < minX) minX = x;
                    if (x > maxX) maxX = x;
                    if (y < minY) minY = y;
                    if (y > maxY) maxY = y;
                    outRgba[i] = 255; outRgba[i + 1] = 0; outRgba[i + 2] = 255; outRgba[i + 3] = 255; // magenta
                }
                else
                {
                    byte gray = (byte)((rgbaB[i] + rgbaB[i + 1] + rgbaB[i + 2]) / 3 / 2);
                    outRgba[i] = gray; outRgba[i + 1] = gray; outRgba[i + 2] = gray; outRgba[i + 3] = 255;
                }
            }
        }

        WriteBmp(outPath, outRgba, widthA, heightA);

        int totalPixels = widthA * heightA;
        if (changedCount == 0)
        {
            Console.WriteLine($"[DIFFSHOT] No differences found ({widthA}x{heightA}, identical) -> {outPath}");
        }
        else
        {
            double pct = 100.0 * changedCount / totalPixels;
            Console.WriteLine($"[DIFFSHOT] {changedCount}/{totalPixels} pixels changed ({pct:F2}%), bounding box ({minX},{minY})-({maxX},{maxY}) -> {outPath}");
        }
        return 0;
    }
}
