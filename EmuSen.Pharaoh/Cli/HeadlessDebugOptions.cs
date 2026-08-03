using EmuSen.Cores.Nintendo.Venus.Controllers;

namespace EmuSen.Pharaoh.Cli
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
        public long CpuTraceEnd { get; init; } = -1;
        public string? CpuTracePath { get; init; }
        public List<string> FlagsToEnable { get; init; } = new();
        public List<(long Start, long End, SnesButton Button, int Controller)> Taps { get; init; } = new();
        public List<(long Frame, string Path)> Screenshots { get; init; } = new();

        // Hex, bools and bare names - see §3.15's --flag entry.
        public static bool TryConvertFlagValue(string? rawValue, Type memberType, out object? value, out string? error)
        {
            value = null;
            error = null;

            if (rawValue == null)
            {
                // "Switch this on" only means anything for a bool - see §3.15.
                if (memberType == typeof(bool)) { value = true; return true; }
                error = $"{memberType.Name} needs an explicit value, e.g. Name=100.";
                return false;
            }

            if (memberType == typeof(bool))
            {
                if (bool.TryParse(rawValue, out bool b)) { value = b; return true; }
                if (rawValue == "1") { value = true; return true; }
                if (rawValue == "0") { value = false; return true; }
                error = $"'{rawValue}' is not a bool (use true/false or 1/0).";
                return false;
            }

            if (memberType == typeof(int))
            {
                bool negative = rawValue.StartsWith('-');
                string body = negative ? rawValue[1..] : rawValue;
                bool hex = body.StartsWith("0x", StringComparison.OrdinalIgnoreCase) || body.StartsWith('$');
                string digits = hex ? (body.StartsWith('$') ? body[1..] : body[2..]) : body;

                bool ok = hex
                    ? int.TryParse(digits, System.Globalization.NumberStyles.HexNumber, null, out int n)
                    : int.TryParse(digits, out n);
                if (ok) { value = negative ? -n : n; return true; }

                error = $"'{rawValue}' is not an integer (decimal, or hex as 0x94 or $94).";
                return false;
            }

            error = $"unsupported DebugSettings member type {memberType.Name}.";
            return false;
        }

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
            long cpuLogStart = -1, cpuLogEnd = -1, cpuTraceEnd = -1;
            string? cpuTracePath = null;
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
                else if (args[i] == "--cputrace" && i + 1 < args.Length)
                {
                    // <frame>:<path>, same shape as --screenshot - see §3.40.
                    string[] p = args[++i].Split(new[] { ':' }, 2);
                    if (p.Length < 2 || !long.TryParse(p[0], out cpuTraceEnd) || p[1].Length == 0)
                    {
                        return (null, warnings, $"[ERROR] --cputrace wants <frame>:<path>, got '{args[i]}'.");
                    }
                    cpuTracePath = p[1];
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
                string[] flagParts = flagSpec.Split(new[] { '=' }, 2);
                string flagName = flagParts[0];
                var settingsType = typeof(EmuSen.Debug.DebugSettings);
                var prop = settingsType.GetProperty(flagName);
                var field = prop == null ? settingsType.GetField(flagName) : null;
                if (prop == null && field == null)
                {
                    warnings.Add($"[WARN] No DebugSettings.{flagName} property or field found - ignoring --flag {flagSpec}.");
                    continue;
                }

                // Same converter as apply time, so a bad value reports not throws.
                Type memberType = prop?.PropertyType ?? field!.FieldType;
                string? rawValue = flagParts.Length >= 2 ? flagParts[1] : null;
                if (!TryConvertFlagValue(rawValue, memberType, out _, out string? convertError))
                {
                    return (null, warnings, $"[ERROR] --flag {flagSpec}: {convertError}");
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
                CpuTraceEnd = cpuTraceEnd,
                CpuTracePath = cpuTracePath,
                FlagsToEnable = flagsToEnable,
                Taps = taps,
                Screenshots = screenshots,
            };
            return (options, warnings, null);
        }
    }
}
