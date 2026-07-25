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
//   dotnet run -- <rom> <frames> [--watch space:addr:len[:kind]]... [--script path] [--out path]
//
// --watch registers an extra watch before the run starts (kind is
// write/read/both, default write) - space/addr/len match `watch add`'s own
// arguments. --script is a text file of newline-separated debug commands
// (anything DebugCommandProcessor understands - `watch log`, `disasm`,
// `writers`, etc.) run once after the frame loop finishes; if omitted, a
// default script just dumps every registered watch's full event log,
// which is exactly what the Yoshi/coin investigation needs. --out mirrors
// all output to a file in addition to stdout.
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
        string? outPath = null;
        bool verbose = false;
        for (int i = 2; i < args.Length; i++)
        {
            if (args[i] == "--watch" && i + 1 < args.Length) extraWatches.Add(args[++i]);
            else if (args[i] == "--script" && i + 1 < args.Length) scriptPath = args[++i];
            else if (args[i] == "--out" && i + 1 < args.Length) outPath = args[++i];
            else if (args[i] == "--verbose") verbose = true;
        }

        // Off by default (matches DebugSettings.MasterLoggingEnabled's own
        // default) - only flip it on when explicitly asked, since it turns
        // on every individually-enabled *Logging flag's live console spam
        // (DmaSourceAddrLogging etc.), useful for a short, targeted run but
        // far too noisy over a long one.
        if (verbose) EmuSen.Debug.DebugSettings.MasterLoggingEnabled = true;

        var log = new List<string>();
        void Emit(string line)
        {
            Console.WriteLine(line);
            log.Add(line);
        }

        Emit($"[ROM] Loading: {romPath}");
        var core = new VenusCore(headless: true);
        core.LoadRom(romPath);

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

        Emit($"[RUN] Executing {frameCount} frames...");
        const int progressEvery = 600; // ~10s of real 60fps gameplay
        for (long frame = 0; frame < frameCount; frame++)
        {
            core.RunFrame();
            if (frame > 0 && frame % progressEvery == 0)
            {
                Emit($"[frame {frame}/{frameCount}]");
            }
        }
        Emit($"[RUN] Done, {core.TotalFrames} total frames executed.");

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
}
