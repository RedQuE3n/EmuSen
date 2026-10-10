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
using EmuSen.DianaSh.DianaSh.Var;
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
                    case "file":
                        int cut = rest.LastIndexOf(' ');
                        current!.Files.Add((rest[..cut], Block(int.Parse(rest[(cut + 1)..]))));
                        break;
                    case "$": step = new GoldenStep { Input = rest }; current!.Steps.Add(step); break;
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
                if (c.VaryNumbers) Text("vary numbers\n");
                foreach (var (path, bytes) in c.Files) Block($"file {path}", bytes);
                foreach (GoldenStep s in c.Steps)
                {
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
        private readonly bool _logging = DianaShLogging.MasterEnabled;
        private readonly Dictionary<string, string> _images = new();

        public string Home => Path.Combine(_temp, "home");

        public GoldenRunner()
        {
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
            DianaShLogging.MasterEnabled = _logging;
            try { Directory.Delete(_temp, recursive: true); } catch (IOException) { }
        }

        // The machine's own paths, written as the patterns the file's header names.
        public string Normalise(string text)
        {
            foreach (var (real, token) in new[] { (Home, "<ROOT>"), (_temp, "<TEMP>"), (DianaShSandbox.InstallDirectory, "<INSTALL>") }.OrderByDescending(p => p.Item1.Length))
                text = text.Replace(real, token);
            return text;
        }

        // A home as a fresh shell first sees it: the skeleton's folders and nothing else.
        private void FreshHome()
        {
            Environment.CurrentDirectory = _temp;
            if (Directory.Exists(Home)) Directory.Delete(Home, recursive: true);
            foreach (string dir in new[] { DataStore.Logs, DataStore.Saves, DataStore.SaveStates, DataStore.Firmware, DataStore.Cheats, DataStore.Games, ConfigStore.Directory, DianaShSandbox.TempDirectory })
                Directory.CreateDirectory(dir);
            Environment.CurrentDirectory = Home;
        }

        private string Image(string engine)
        {
            if (_images.TryGetValue(engine, out string? path)) return path;
            (string extension, byte[] bytes) = engine switch
            {
                "venus" or "venusrt" => (".sfc", SyntheticRom.BuildBlank()),
                "venus-sa1" => (".sfc", SyntheticRom.BuildSa1()),
                "venus-gsu" => (".sfc", SyntheticRom.BuildSuperFx()),
                "moon" or "moonrt" => (".nes", SyntheticNesRom.Build()),
                "mercury" or "mercuryrt" => (".gb", SyntheticGbRom.Build()),
                "mars" or "marsrt" => (".z64", SyntheticN64Rom.Build(patches: (0, new byte[] { 0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00 }))),
                "nephrite" => (".md", SyntheticMdRom.Cartridge()),
                _ => throw new ArgumentException($"no golden target '{engine}'"),
            };
            Directory.CreateDirectory(Path.Combine(_temp, "images"));
            path = Path.Combine(_temp, "images", engine + extension);
            File.WriteAllBytes(path, bytes);
            return _images[engine] = path;
        }

        private static string? EngineName(string engine) => engine switch
        {
            "venusrt" => CoreCatalog.VenusRtEngine,
            "moonrt" => CoreCatalog.MoonRtEngine,
            "mercuryrt" => CoreCatalog.MercuryRtEngine,
            "marsrt" => CoreCatalog.MarsRtEngine,
            _ => null,
        };

        private sealed class Built(IDebugTarget? target, CoreBundle? bundle, IDisposable? owner) : IDisposable
        {
            public IDebugTarget? Target { get; } = target;
            public CoreBundle? Bundle { get; } = bundle;
            public void Dispose() => owner?.Dispose();
        }

        private Built Target(GoldenCase c)
        {
            switch (c.Target)
            {
                case "none": return new Built(null, null, null);
                case "fake": return new Built(new GoldenDebugTarget(), null, null);
                case "bare": return new Built(new GoldenDebugTarget(bare: true), null, null);
                case "nobus": return new Built(new GoldenDebugTarget(bus: false), null, null);
            }
            string path = Image(c.Target);
            ICore core = CoreFactory.Create(path, headless: true, EngineName(c.Target));
            core.LoadRom(path);
            for (int i = 0; i < c.Frames; i++) core.RunFrame();
            CoreBundle bundle = CoreFactory.Bundle(core);
            return new Built(bundle.DebugTarget, bundle, core as IDisposable);
        }

        private static readonly System.Reflection.MethodInfo GetVariable =
            typeof(DianaShInterpreter).GetMethod("GetVariable", System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.NonPublic)!;

        // $? read the way the parser reads it, without a line of the shell's own - see EmuSen_DianaSh.md §13.4.
        private static string Status(DianaShInterpreter shell) => (string)GetVariable.Invoke(shell, new object[] { "?" })!;

        // Every command a host's shell has, in its own order, with the two the hosts build themselves.
        public static IReadOnlyList<IDianaShCommand> Commands()
        {
            DianaShInterpreter shell = DianaShInterpreter.CreateDefault(new GoldenDebugTarget(), HostCommands(() => ""));
            var field = typeof(DianaShInterpreter).GetField("_orderedCommands", System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.NonPublic)!;
            return ((IEnumerable<IDianaShCommand>)field.GetValue(shell)!).ToList();
        }

        private static IDianaShCommand[] HostCommands(Func<string> statePath) => new IDianaShCommand[]
        {
            new CoreCommand(CoreCatalog.Registry),
            new StateCommand(p => File.WriteAllText(p, "state"), p => _ = File.ReadAllText(p), statePath),
        };

        public List<GoldenStep> Run(GoldenCase c)
        {
            FreshHome();
            foreach (var (relative, bytes) in c.Files)
            {
                string full = Path.Combine(Home, relative);
                Directory.CreateDirectory(Path.GetDirectoryName(full)!);
                File.WriteAllBytes(full, bytes);
            }
            foreach (string user in DianaShUserRegistry.Users.Where(u => !u.Equals("root", StringComparison.OrdinalIgnoreCase)).ToList()) DianaShUserRegistry.TryRemove(user);
            DianaShLogging.MasterEnabled = false;

            using Built built = Target(c);
            string statePath = Path.Combine(DataStore.SaveStates, "golden.state");
            IDianaShCommand[] extras = HostCommands(() => statePath);
            DianaShSessionManager? sessions = c.NoSessions ? null : new DianaShSessionManager();
            CoreBundle? b = built.Bundle;
            DianaShInterpreter Build() => DianaShInterpreter.CreateDefault(built.Target, extras, b?.CheatAutoDetectCodec, b?.CheatExplicitCodec, b?.CpuTraceSwitch, sessions,
                () => CoreCatalog.SupportedCheatSystems);
            DianaShInterpreter? alone = sessions is null ? Build() : null;
            sessions?.RegisterInitial(new DianaShSession("main", Build()));

            var steps = new List<GoldenStep>();
            foreach (GoldenStep s in c.Steps)
            {
                DianaShInterpreter shell = alone ?? sessions!.Current!.Interpreter;
                var (needsMore, output, action) = shell.Submit(s.Input);
                steps.Add(new GoldenStep
                {
                    Input = s.Input,
                    Output = Encoding.UTF8.GetBytes(Normalise(output)),
                    Status = int.Parse(Status(shell)),
                    NeedsMore = needsMore,
                    Action = action is null ? "none" : Normalise(action.ToString()!),
                });
            }
            return steps;
        }

        // What the case would be recorded as: literal bytes, or a pattern where its numbers vary.
        public static GoldenStep Recorded(GoldenStep run, GoldenStep? existing, bool varyNumbers)
        {
            var step = new GoldenStep { Input = run.Input, Status = run.Status, NeedsMore = run.NeedsMore, Action = run.Action };
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
                else
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
                if (want.Action != got.Action) return $"{where}: action recorded '{want.Action}', now '{got.Action}'";
            }
            return null;
        }
    }
}
