using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.Debug;

// Headless AI-agent-driven debugging harness - the "not yet built" item
// from Man pages/EmuSen_Core_Gameplan.md §1's backlog. Every investigation
// in this project so far (including the Yoshi/coin WRAM-staging one) has
// required a human to run F4 prompt commands during a live play session
// and manually relay the console output back. This loads a ROM, runs the
// real VenusCore for a fixed number of frames with no window/audio/human
// involved, then feeds a script of the exact same debug commands the F4
// prompt accepts (DebugCommandProcessor.Execute doesn't care where a
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
// (anything DebugCommandProcessor understands - `watch log`, `disasm`,
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
//   anything else           - passed straight to DebugCommandProcessor.Execute, same as --script
// <frames> is still required and still means what it always did in every
// other mode - here it becomes a hard safety cap (a script's `frames`
// requests refuse to advance past it) so a typo can't hang the process
// indefinitely. --tap/--tap2/--screenshot/--script are ignored when
// --commands is given; everything else (--watch, --flag, --loadstate,
// --savestate, --verbose, --cpulog, --out) still applies normally.
class Program
{
    static int Main(string[] args)
    {
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

        var debugTarget = new SnesDebugTarget(core.Cpu!, core.Bus!);
        var debugCmd = new DebugCommandProcessor(debugTarget);

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
                    if (currentFrame % progressEvery == 0) Emit($"[frame {currentFrame}/{frameCount}]");
                }
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
}
