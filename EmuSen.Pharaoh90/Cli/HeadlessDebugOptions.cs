using EmuSen.Cores.Nintendo.Venus.Controllers;

namespace EmuSen.Pharaoh90.Cli
{
    // Everything the classic-loop/--commands entry points need out of argv,
    // pulled out of Program.cs's old inline parsing loop so it's a plain
    // function of a string array - genuinely unit-testable for the first
    // time, instead of only exercisable by actually launching the process.
    // --diffshot is deliberately NOT handled here - Program.cs dispatches
    // that standalone mode before ever calling Parse, same as before.
    public sealed class HeadlessDebugOptions
    {
        public required string RomPath { get; init; }
        public required long FrameCount { get; init; }
        public List<string> ExtraWatches { get; init; } = new();
        public string? ScriptPath { get; init; }
        public string? CommandsPath { get; init; }
        public string? AutoshotDir { get; init; }
        public string? OutPath { get; init; }
        public string? LoadStatePath { get; init; }
        public string? SaveStatePath { get; init; }
        public bool Verbose { get; init; }
        public long CpuLogStart { get; init; } = -1;
        public long CpuLogEnd { get; init; } = -1;
        public List<string> FlagsToEnable { get; init; } = new();
        public List<(long Start, long End, SnesButton Button, int Controller)> Taps { get; init; } = new();
        public List<(long Frame, string Path)> Screenshots { get; init; } = new();

        // Result of a parse attempt: either Options is set (success) or
        // Error is set (caller should print it and exit 1) - never both.
        // Warnings can be non-empty either way (e.g. an unrecognized
        // --flag alongside an otherwise-successful parse).
        public static (HeadlessDebugOptions? Options, List<string> Warnings, string? Error) Parse(string[] args)
        {
            var warnings = new List<string>();

            if (args.Length < 2)
            {
                return (null, warnings, "Usage: dotnet run -- <rom> <frames> [--watch space:addr:len[:kind]]... [--script path] [--out path]");
            }

            string romPath = args[0];
            if (!long.TryParse(args[1], out long frameCount) || frameCount <= 0)
            {
                return (null, warnings, $"[ERROR] Invalid frame count: {args[1]}");
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
            var taps = new List<(long Start, long End, SnesButton Button, int Controller)>();
            var screenshots = new List<(long Frame, string Path)>();

            for (int i = 2; i < args.Length; i++)
            {
                if (args[i] == "--flag" && i + 1 < args.Length)
                {
                    // See DebugSettings' own comment - dozens of bool
                    // *Logging flags plus the occasional non-bool sidecar,
                    // set by reflection so this harness never needs a
                    // rebuild per investigation. Plain "Name" sets a bool
                    // to true; "Name=value" parses value as whatever that
                    // member's actual type is.
                    flagsToEnable.Add(args[++i]);
                }
                else if (args[i] == "--cpulog" && i + 1 < args.Length)
                {
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
                    string[] p = args[++i].Split(new[] { ':' }, 2);
                    screenshots.Add((long.Parse(p[0]), p[1]));
                }
                else if (args[i] == "--tap" && i + 1 < args.Length)
                {
                    string[] p = args[++i].Split(':');
                    long start = long.Parse(p[0]);
                    var button = Enum.Parse<SnesButton>(p[1], ignoreCase: true);
                    long duration = p.Length >= 3 ? long.Parse(p[2]) : 4;
                    taps.Add((start, start + duration, button, 1));
                }
                else if (args[i] == "--tap2" && i + 1 < args.Length)
                {
                    string[] p = args[++i].Split(':');
                    long start = long.Parse(p[0]);
                    var button = Enum.Parse<SnesButton>(p[1], ignoreCase: true);
                    long duration = p.Length >= 3 ? long.Parse(p[2]) : 4;
                    taps.Add((start, start + duration, button, 2));
                }
            }

            // Validates each --flag against DebugSettings by reflection up
            // front so the warning can flow through Emit/--out like every
            // other diagnostic in this file, instead of the old
            // Console.WriteLine directly from inside argument parsing
            // (which ran before the log/--out plumbing even existed, so it
            // silently never reached --out). The actual SetValue
            // application still happens later, in the same relative order
            // as before (right after ROM load) - this is purely an
            // additional read-only lookup to source the warning text.
            foreach (string flagSpec in flagsToEnable)
            {
                string flagName = flagSpec.Split(new[] { '=' }, 2)[0];
                var settingsType = typeof(EmuSen.Debug.DebugSettings);
                var prop = settingsType.GetProperty(flagName);
                var field = prop == null ? settingsType.GetField(flagName) : null;
                if (prop == null && field == null)
                {
                    warnings.Add($"[WARN] No DebugSettings.{flagName} property or field found - ignoring --flag {flagSpec}.");
                }
            }

            var options = new HeadlessDebugOptions
            {
                RomPath = romPath,
                FrameCount = frameCount,
                ExtraWatches = extraWatches,
                ScriptPath = scriptPath,
                CommandsPath = commandsPath,
                AutoshotDir = autoshotDir,
                OutPath = outPath,
                LoadStatePath = loadStatePath,
                SaveStatePath = saveStatePath,
                Verbose = verbose,
                CpuLogStart = cpuLogStart,
                CpuLogEnd = cpuLogEnd,
                FlagsToEnable = flagsToEnable,
                Taps = taps,
                Screenshots = screenshots,
            };
            return (options, warnings, null);
        }
    }
}
