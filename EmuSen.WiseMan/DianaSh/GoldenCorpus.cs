using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.RegularExpressions;
using EmuSen.Cores;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Var.Shell;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaSh
{
    // One line submitted and what it gave: its bytes, or a pattern they must match, its exit status, whether it left a block open, and its host action.
    public sealed class GoldenStep
    {
        public string Input = "";
        public byte[]? Output;
        public string? Pattern;
        public int Status;
        public bool NeedsMore;
        public string Action = "none";

        // Frames the host runs on the case's core before this line, stopping at a halt.
        public int RunBefore;
    }

    public sealed class GoldenCase
    {
        public string Title = "";
        public string Target = "none";
        public int Frames = 2;
        public List<int> Covers = new();
        public List<(string Path, byte[] Bytes)> Files = new();
        public bool VaryNumbers;
        public bool NoSessions;
        public bool NoWindow;
        public List<GoldenStep> Steps = new();
    }

    // A command's golden file: the usage lines it was recorded against, the failures it cannot reach, and its cases - see EmuSen_DianaSh.md §13.4.
    public sealed class GoldenFile
    {
        public string Command = "";
        public List<string> Usage = new();
        public List<(string Fragment, string Reason)> Unreachable = new();
        public List<(int Line, string Reason)> Uncovered = new();
        public List<GoldenCase> Cases = new();

        private const string Header =
            "# DianaSh golden corpus. Recorded from the C# shell, byte for byte; EMUSEN_DIANASH_GOLDEN=record rewrites it - see EmuSen_DianaSh.md §13.4.\n" +
            "# Patterns: <ROOT> is the sandbox's real root, <TEMP> the corpus's real temporary folder, <INSTALL> the install's real directory;\n" +
            "# a 'match' block is a regular expression the whole output must match, written where a line prints a time or a measured number.\n";

        public static GoldenFile Parse(byte[] bytes)
        {
            var file = new GoldenFile();
            int at = 0;
            GoldenCase? current = null;
            GoldenStep? step = null;
            int run = 0;
            string Line()
            {
                int end = Array.IndexOf(bytes, (byte)'\n', at);
                if (end < 0) end = bytes.Length;
                string text = Encoding.UTF8.GetString(bytes, at, end - at);
                at = Math.Min(bytes.Length, end + 1);
                return text;
            }
            byte[] Block(int count)
            {
                byte[] block = bytes.AsSpan(at, count).ToArray();
                at += count;
                if (at < bytes.Length && bytes[at] == (byte)'\n') at++;
                return block;
            }
            while (at < bytes.Length)
            {
                string line = Line();
                if (line.Length == 0 || line.StartsWith('#')) continue;
                int space = line.IndexOf(' ');
                string word = space < 0 ? line : line[..space];
                string rest = space < 0 ? "" : line[(space + 1)..];
                switch (word)
                {
                    case "command": file.Command = rest; break;
                    case "usage": file.Usage.Add(rest[(rest.IndexOf(' ') + 1)..]); break;
                    case "unreachable":
                        int dash = rest.IndexOf(" -- ", StringComparison.Ordinal);
                        file.Unreachable.Add((Unquote(rest[..dash]), rest[(dash + 4)..]));
                        break;
                    case "uncovered":
                        int sep = rest.IndexOf(" -- ", StringComparison.Ordinal);
                        file.Uncovered.Add((int.Parse(rest[..sep]), rest[(sep + 4)..]));
                        break;
                    case "case": current = new GoldenCase { Title = rest }; file.Cases.Add(current); step = null; break;
                    case "target": current!.Target = rest; break;
                    case "frames": current!.Frames = int.Parse(rest); break;
                    case "covers": current!.Covers.AddRange(rest.Split(',').SelectMany(Range)); break;
                    case "vary": current!.VaryNumbers = rest == "numbers"; break;
                    case "sessions": current!.NoSessions = rest == "none"; break;
                    case "window": current!.NoWindow = rest == "none"; break;
                    case "file":
                        int cut = rest.LastIndexOf(' ');
                        current!.Files.Add((rest[..cut], Block(int.Parse(rest[(cut + 1)..]))));
                        break;
                    case "run": run = int.Parse(rest); break;
                    case "$": step = new GoldenStep { Input = rest, RunBefore = run }; current!.Steps.Add(step); run = 0; break;
                    case "out": step!.Output = Block(int.Parse(rest)); break;
                    case "match": step!.Pattern = Encoding.UTF8.GetString(Block(int.Parse(rest))); break;
                    case "status":
                        string[] parts = rest.Split(' ');
                        step!.Status = int.Parse(parts[0]);
                        step.NeedsMore = parts.Length > 1 && parts[1] == "more";
                        break;
                    case "action": step!.Action = rest; break;
                    case "end": current = null; step = null; break;
                    default: throw new InvalidDataException($"unknown golden line '{line}'");
                }
            }
            return file;
        }

        private static IEnumerable<int> Range(string text)
        {
            int dash = text.IndexOf('-');
            if (dash < 0) return new[] { int.Parse(text) };
            int from = int.Parse(text[..dash]);
            return Enumerable.Range(from, int.Parse(text[(dash + 1)..]) - from + 1);
        }

        private static string Ranges(List<int> lines)
        {
            var parts = new List<string>();
            var sorted = lines.Distinct().OrderBy(n => n).ToList();
            for (int i = 0; i < sorted.Count;)
            {
                int j = i;
                while (j + 1 < sorted.Count && sorted[j + 1] == sorted[j] + 1) j++;
                parts.Add(j == i ? $"{sorted[i]}" : $"{sorted[i]}-{sorted[j]}");
                i = j + 1;
            }
            return string.Join(',', parts);
        }

        private static string Quote(string text) => "\"" + text.Replace("\\", "\\\\").Replace("\"", "\\\"").Replace("\n", "\\n") + "\"";

        private static string Unquote(string text) => Regex.Unescape(text.Trim('"').Replace("\\\"", "\""));

        public byte[] Write()
        {
            var o = new MemoryStream();
            void Text(string s) { byte[] b = Encoding.UTF8.GetBytes(s); o.Write(b); }
            void Block(string word, byte[] block) { Text($"{word} {block.Length}\n"); o.Write(block); Text("\n"); }
            Text(Header);
            Text($"command {Command}\n");
            for (int i = 0; i < Usage.Count; i++) Text($"usage {i + 1} {Usage[i]}\n");
            foreach (var (line, reason) in Uncovered) Text($"uncovered {line} -- {reason}\n");
            foreach (var (fragment, reason) in Unreachable) Text($"unreachable {Quote(fragment)} -- {reason}\n");
            foreach (GoldenCase c in Cases)
            {
                Text($"\ncase {c.Title}\n");
                Text($"target {c.Target}\n");
                if (c.Frames != 2) Text($"frames {c.Frames}\n");
                if (c.Covers.Count > 0) Text($"covers {Ranges(c.Covers)}\n");
                if (c.NoSessions) Text("sessions none\n");
                if (c.NoWindow) Text("window none\n");
                if (c.VaryNumbers) Text("vary numbers\n");
                foreach (var (path, bytes) in c.Files) Block($"file {path}", bytes);
                foreach (GoldenStep s in c.Steps)
                {
                    if (s.RunBefore > 0) Text($"run {s.RunBefore}\n");
                    Text($"$ {s.Input}\n");
                    if (s.Pattern is not null) Block("match", Encoding.UTF8.GetBytes(s.Pattern));
                    else if (s.Output is not null) Block("out", s.Output);
                    Text($"status {s.Status}{(s.NeedsMore ? " more" : "")}\n");
                    Text($"action {s.Action}\n");
                }
                Text("end\n");
            }
            return o.ToArray();
        }
    }

    // Runs a case on a fresh shell over a fresh target, in a fresh home, and gives back what each line printed - see EmuSen_DianaSh.md §13.4.
    public sealed class GoldenRunner : IDisposable
    {
        private readonly string _temp = Path.Combine(Path.GetTempPath(), "EmuSenGolden", Guid.NewGuid().ToString("N"));
        private readonly string? _dataOverride = DataStore.OverrideDirectory;
        private readonly string? _configOverride = ConfigStore.OverrideDirectory;
        private readonly string? _legacyOverride = ConfigStore.OverrideLegacyDirectory;
        private readonly string _cwd = Environment.CurrentDirectory;
        private readonly bool _logging = DebugLogging.MasterEnabled;
        private readonly GoldenTargets _targets;

        public string Home => Path.Combine(_temp, "home");

        public GoldenRunner()
        {
            _targets = new GoldenTargets(_temp);
            DianaShSandbox.EnsureInitialWorkingDirectory();
            Directory.CreateDirectory(_temp);
            DataStore.OverrideDirectory = Home;
            ConfigStore.OverrideDirectory = Path.Combine(Home, ConfigStore.ConfigDirName, ConfigStore.ProgramDirName);
            ConfigStore.OverrideLegacyDirectory = Path.Combine(_temp, "legacy");
        }

        public void Dispose()
        {
            Environment.CurrentDirectory = _cwd;
            DataStore.OverrideDirectory = _dataOverride;
            ConfigStore.OverrideDirectory = _configOverride;
            ConfigStore.OverrideLegacyDirectory = _legacyOverride;
            DebugLogging.MasterEnabled = _logging;
            try { Directory.Delete(_temp, recursive: true); } catch (IOException) { }
        }

        // The machine's own paths, written as the patterns the file's header names, each also as the system resolves its links (macOS's /var is /private/var).
        public string Normalise(string text)
        {
            var paths = new[] { (Home, "<ROOT>"), (_temp, "<TEMP>"), (DianaShSandbox.InstallDirectory, "<INSTALL>") };
            foreach (var (real, token) in paths.Concat(paths.Select(p => (Resolved(p.Item1), p.Item2))).Where(p => p.Item1.Length > 0).OrderByDescending(p => p.Item1.Length))
                text = text.Replace(real, token);
            return text;
        }

        // A path with each component's link followed, as the system names a directory entered through one.
        private static string Resolved(string path)
        {
            string full = Path.GetFullPath(path);
            string? parent = Path.GetDirectoryName(full);
            if (parent is null) return full;
            string current = Path.Combine(Resolved(parent), Path.GetFileName(full));
            var info = new DirectoryInfo(current);
            return info.LinkTarget is not null && info.ResolveLinkTarget(returnFinalTarget: true) is { } target ? Resolved(target.FullName) : current;
        }

        // Whether a line names an entry of the home only in another case, as `cd /LOGS` names `Logs`: the file system's question where it ignores case, which Linux's recording does not answer.
        public bool AsksTheSystemsCase(string input)
        {
            if (!OperatingSystem.IsWindows() && !OperatingSystem.IsMacOS()) return false;
            string[] entries = Directory.Exists(Home) ? Directory.EnumerateFileSystemEntries(Home).Select(Path.GetFileName).OfType<string>().ToArray() : [];
            return input.Split(' ', StringSplitOptions.RemoveEmptyEntries)
                .Select(w => w.Trim('\'', '"').TrimStart('/').Split('/')[0])
                .Any(first => first.Length > 0 && !entries.Contains(first, StringComparer.Ordinal) && entries.Contains(first, StringComparer.OrdinalIgnoreCase));
        }

        // The recordings are Linux's: on Windows a path is printed with the system's separator and a line may end as the system ends it.
        public static bool SameOnThisSystem(string recorded, string now) =>
            recorded == now || (OperatingSystem.IsWindows() && recorded.Replace('\\', '/').Replace("\r\n", "\n") == now.Replace('\\', '/').Replace("\r\n", "\n"));

        // A home as a fresh shell first sees it: the skeleton's folders and nothing else.
        private void FreshHome()
        {
            Environment.CurrentDirectory = _temp;
            if (Directory.Exists(Home)) Directory.Delete(Home, recursive: true);
            foreach (string dir in new[] { DataStore.Logs, DataStore.Saves, DataStore.SaveStates, DataStore.Firmware, DataStore.Cheats, DataStore.Games, ConfigStore.Directory, DianaShSandbox.TempDirectory })
                Directory.CreateDirectory(dir);
            Environment.CurrentDirectory = Home;
        }

        private GoldenTargets.Built Target(GoldenCase c) => _targets.Build(c.Target, c.Frames);

        // Every command a host's shell has, in its own order, with the two the hosts build themselves.
        public static IReadOnlyList<IDianaShCommand> Commands()
        {
            return DianaShInterpreter.CreateDefault(new GoldenDebugTarget(), HostCommands(() => "", window: true)).RegisteredCommands;
        }

        // The commands the shell program adds; `hotaru`'s window is a stand-in that opens nothing, and a ROM named broken* will not load.
        private static IDianaShCommand[] HostCommands(Func<string> statePath, bool window) => new IDianaShCommand[]
        {
            new CoreCommand(CoreChoices.Of(CoreCatalog.Registry), console => CoreCatalog.EmptyStartFor(console) is not null),
            new StateCommand(p => File.WriteAllText(p, "state"), p => _ = File.ReadAllText(p), statePath),
            new EmuSen.Hotaru.HotaruCommand(window ? OpenNothing : null),
        };

        private static void OpenNothing(string rom, EmuSen.Hotaru.HotaruOutput output)
        {
            if (Path.GetFileName(rom).StartsWith("broken", StringComparison.Ordinal)) throw new InvalidOperationException("unsupported mapper");
        }

        // Set by the last Run that stopped at a line asking the file system's case.
        public bool SystemsCase { get; private set; }

        public List<GoldenStep> Run(GoldenCase c)
        {
            SystemsCase = false;
            FreshHome();
            foreach (var (relative, bytes) in c.Files)
            {
                string full = Path.Combine(Home, relative);
                Directory.CreateDirectory(Path.GetDirectoryName(full)!);
                File.WriteAllBytes(full, bytes);
            }
            foreach (string user in DianaShUserRegistry.Users.Where(u => !u.Equals("root", StringComparison.OrdinalIgnoreCase)).ToList()) DianaShUserRegistry.TryRemove(user);
            DebugLogging.MasterEnabled = false;

            using GoldenTargets.Built built = Target(c);
            string statePath = Path.Combine(DataStore.SaveStates, "golden.state");
            IDianaShCommand[] extras = HostCommands(() => statePath, window: !c.NoWindow);
            DianaShSessionManager? sessions = c.NoSessions ? null : new DianaShSessionManager();
            CoreBundle? b = built.Bundle;
            DianaShInterpreter Build() => DianaShInterpreter.CreateDefault(built.Target, extras, b?.CheatAutoDetectCodec, b?.CheatExplicitCodec, b?.CpuTraceSwitch, sessions,
                () => CoreCatalog.SupportedCheatSystems);
            DianaShInterpreter? alone = sessions is null ? Build() : null;
            sessions?.RegisterInitial(new DianaShSession("main", Build()));

            var steps = new List<GoldenStep>();
            foreach (GoldenStep s in c.Steps)
            {
                // A case that reaches such a line is the file system's from there on, and is left uncompared, as the crate-alone runner leaves it.
                if (AsksTheSystemsCase(s.Input)) { SystemsCase = true; break; }
                if (s.RunBefore > 0) RunFrames(b?.Core ?? throw new InvalidDataException($"'run {s.RunBefore}' needs a core, and case '{c.Title}' has none"), s.RunBefore);
                DianaShInterpreter shell = alone ?? sessions!.Current!.Interpreter;
                var (needsMore, output, action) = shell.Submit(s.Input);
                steps.Add(new GoldenStep
                {
                    Input = s.Input,
                    RunBefore = s.RunBefore,
                    Output = Encoding.UTF8.GetBytes(Normalise(output)),
                    Status = int.Parse(shell.LastExitStatus),
                    NeedsMore = needsMore,
                    Action = action is null ? "none" : Normalise(action.ToString()!),
                });
            }
            return steps;
        }

        // As a frontend's loop runs them: a halted core resumes, and a halt ends the run.
        private static void RunFrames(ICore core, int frames)
        {
            for (int f = 0; f < frames; f++)
            {
                core.RunFrame();
                if (core.IsHaltedAtBreakpoint) return;
            }
        }

        // What the case would be recorded as: literal bytes, or a pattern where its numbers vary.
        public static GoldenStep Recorded(GoldenStep run, GoldenStep? existing, bool varyNumbers)
        {
            var step = new GoldenStep { Input = run.Input, RunBefore = run.RunBefore, Status = run.Status, NeedsMore = run.NeedsMore, Action = run.Action };
            string text = Encoding.UTF8.GetString(run.Output!);
            if (existing?.Pattern is { } kept && Regex.IsMatch(text, kept, RegexOptions.Singleline)) step.Pattern = kept;
            else if (varyNumbers) step.Pattern = "^" + Regex.Replace(Regex.Escape(text), @"\d+(\\\.\d+)?", @"\d+(\.\d+)?") + "$";
            else step.Output = run.Output;
            return step;
        }

        // Null when the run is what the file holds; otherwise where the first difference is.
        public static string? Compare(string file, GoldenCase c, List<GoldenStep> run)
        {
            if (run.Count != c.Steps.Count) return $"{file}: case '{c.Title}' has {c.Steps.Count} recorded lines and ran {run.Count}";
            for (int i = 0; i < run.Count; i++)
            {
                GoldenStep want = c.Steps[i], got = run[i];
                string where = $"{file}: case '{c.Title}', line {i + 1} ($ {want.Input})";
                if (want.Pattern is { } pattern)
                {
                    string text = Encoding.UTF8.GetString(got.Output!);
                    if (!Regex.IsMatch(text, pattern, RegexOptions.Singleline)) return $"{where}: the output does not match its pattern:\n{text}";
                }
                else if (!SameOnThisSystem(Encoding.UTF8.GetString(want.Output ?? Array.Empty<byte>()), Encoding.UTF8.GetString(got.Output!)))
                {
                    byte[] w = want.Output ?? Array.Empty<byte>(), g = got.Output!;
                    int n = 0;
                    while (n < w.Length && n < g.Length && w[n] == g[n]) n++;
                    if (n < w.Length || n < g.Length)
                        return $"{where}: first difference at byte {n}: recorded {(n < w.Length ? $"0x{w[n]:X2}" : "end")}, now {(n < g.Length ? $"0x{g[n]:X2}" : "end")}; " +
                               $"now reads \"{Encoding.UTF8.GetString(g, Math.Max(0, n - 20), Math.Min(g.Length, n + 40) - Math.Max(0, n - 20))}\"";
                }
                if (want.Status != got.Status) return $"{where}: status recorded {want.Status}, now {got.Status}";
                if (want.NeedsMore != got.NeedsMore) return $"{where}: continuation recorded {want.NeedsMore}, now {got.NeedsMore}";
                if (!SameOnThisSystem(want.Action, got.Action)) return $"{where}: action recorded '{want.Action}', now '{got.Action}'";
            }
            return null;
        }
    }
}
