using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var.Shell;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaSh
{
    // One line submitted and what it gave: its bytes, its exit status, whether it left a block open, and its host action.
    public sealed class GoldenStep
    {
        public string Input = "";
        public byte[]? Output;
        public int Status;
        public bool NeedsMore;
        public string Action = "none";
    }

    public sealed class GoldenCase
    {
        public string Title = "";
        public List<(string Path, byte[] Bytes)> Files = new();
        public bool NoSessions;
        public bool NoWindow;
        public List<GoldenStep> Steps = new();
    }

    // Runs a case on a fresh shell with no game, in a fresh home, and gives back what each line printed; the corpus itself is the dianash crate's to run - see EmuSen_DianaSh.md §13.4 and EmuSen_Backend.md §40.10.
    public sealed class GoldenRunner : IDisposable
    {
        private readonly string _temp = Path.Combine(Path.GetTempPath(), "EmuSenGolden", Guid.NewGuid().ToString("N"));
        private readonly string? _dataOverride = DataStore.OverrideDirectory;
        private readonly string? _configOverride = ConfigStore.OverrideDirectory;
        private readonly string? _legacyOverride = ConfigStore.OverrideLegacyDirectory;
        private readonly string _cwd = Environment.CurrentDirectory;
        private readonly IReadOnlyDictionary<string, CoreChoice> _cores;

        public string Home => Path.Combine(_temp, "home");

        public GoldenRunner()
        {
            DianaShSandbox.EnsureInitialWorkingDirectory();
            Directory.CreateDirectory(_temp);
            DataStore.OverrideDirectory = Home;
            ConfigStore.OverrideDirectory = Path.Combine(Home, ConfigStore.ConfigDirName, ConfigStore.ProgramDirName);
            ConfigStore.OverrideLegacyDirectory = Path.Combine(_temp, "legacy");
            using var runtime = new EmuSen.Crystal.Client.CrystalRuntime();
            _cores = EmuSen.Hotaru.HotaruCores.Of(runtime).Registry;
        }

        public void Dispose()
        {
            Environment.CurrentDirectory = _cwd;
            DataStore.OverrideDirectory = _dataOverride;
            ConfigStore.OverrideDirectory = _configOverride;
            ConfigStore.OverrideLegacyDirectory = _legacyOverride;
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

        // A home as a fresh shell first sees it: the skeleton's folders and nothing else.
        private void FreshHome()
        {
            Environment.CurrentDirectory = _temp;
            if (Directory.Exists(Home)) Directory.Delete(Home, recursive: true);
            foreach (string dir in new[] { DataStore.Logs, DataStore.Saves, DataStore.SaveStates, DataStore.Firmware, DataStore.Cheats, DataStore.Games, ConfigStore.Directory, DianaShSandbox.TempDirectory })
                Directory.CreateDirectory(dir);
            Environment.CurrentDirectory = Home;
        }

        // The commands the shell program adds, `core` over the runtime's systems; `hotaru`'s window is a stand-in that opens nothing, and a ROM named broken* will not load.
        private IDianaShCommand[] HostCommands(Func<string> statePath, bool window) => new IDianaShCommand[]
        {
            new CoreCommand(_cores),
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
            string statePath = Path.Combine(DataStore.SaveStates, "golden.state");
            IDianaShCommand[] extras = HostCommands(() => statePath, window: !c.NoWindow);
            DianaShSessionManager? sessions = c.NoSessions ? null : new DianaShSessionManager();
            DianaShInterpreter Build() => DianaShInterpreter.CreateWithoutSession(extraCommands: extras, sessions: sessions);
            DianaShInterpreter? alone = sessions is null ? Build() : null;
            sessions?.RegisterInitial(new DianaShSession("main", Build()));

            var steps = new List<GoldenStep>();
            foreach (GoldenStep s in c.Steps)
            {
                // A case that reaches such a line is the file system's from there on, and is left uncompared, as the crate-alone runner leaves it.
                if (AsksTheSystemsCase(s.Input)) { SystemsCase = true; break; }
                DianaShInterpreter shell = alone ?? sessions!.Current!.Interpreter;
                var (needsMore, output, action) = shell.Submit(s.Input);
                steps.Add(new GoldenStep
                {
                    Input = s.Input,
                    Output = Encoding.UTF8.GetBytes(Normalise(output)),
                    Status = int.Parse(shell.LastExitStatus),
                    NeedsMore = needsMore,
                    Action = action is null ? "none" : Normalise(action.ToString()!),
                });
            }
            return steps;
        }
    }
}
