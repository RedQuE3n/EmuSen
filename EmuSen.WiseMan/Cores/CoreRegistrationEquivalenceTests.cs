using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Security.Cryptography;
using System.Text.Json.Nodes;
using EmuSen.Common.Firmware;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Cores.Nintendo.Mars.Rom;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Every answer CoreCatalog and CoreFactory give, for every console and engine, held to a committed golden: D1's oracle - see EmuSen_CoreAPI.md §25.
    [Collection(TestCollections.ProcessGlobals)]
    public class CoreRegistrationEquivalenceTests : IDisposable
    {
        public const string GoldenName = "CoreRegistrationGolden.txt";
        public const string RecordVariable = "EMUSEN_RECORD_REGISTRATION";
        public const string OffMarkVariable = "EMUSEN_REGISTRATION_OFF_MARK";

        // The switch variables of every engine a build registers by hand, and VenusRT's former shim's.
        public static readonly string[] OffVariables = { MoonNative.Variable, MercuryNative.Variable, MarsNative.Variable, "EMUSEN_VENUS_NATIVE" };

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenRegistration_" + Guid.NewGuid().ToString("N"));
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public CoreRegistrationEquivalenceTests()
        {
            Directory.CreateDirectory(_root);
            CoreOptions.BatteryRamDisabled = false;
        }

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
            CoreDiscovery.UseDirectories(null);
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        [Fact]
        public void Every_answer_with_the_libraries_as_built_equals_the_golden()
        {
            var lines = new RegistrationRecorder(_root).Record("built", includeDiscoveryVariants: true);
            Compare("built", lines);
        }

        // The libraries are loaded once per process, so a process started with them off is the only honest record of the fallbacks.
        [Fact]
        public void Every_answer_with_each_registered_library_turned_off_equals_the_golden()
        {
            string mark = Path.Combine(_root, "off.txt");
            var start = new ProcessStartInfo("dotnet") { RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false };
            foreach (string argument in new[] { "test", typeof(CoreRegistrationEquivalenceTests).Assembly.Location, "--filter", $"FullyQualifiedName={typeof(CoreRegistrationEquivalenceTests).FullName}.{nameof(TheRecordInAProcessWithTheLibrariesOff)}" })
                start.ArgumentList.Add(argument);
            foreach (string variable in OffVariables) start.Environment[variable] = "0";
            start.Environment[OffMarkVariable] = mark;
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync();
            var stderr = process.StandardError.ReadToEndAsync();
            Assert.True(process.WaitForExit(240_000), "the child test process did not finish in four minutes");
            Assert.True(process.ExitCode == 0 && File.Exists(mark), $"the child process failed:\n{stdout.Result}\n{stderr.Result}");
            Compare("off", File.ReadAllLines(mark).ToList());
        }

        // Runs only in the child process the test above starts; elsewhere it records nothing.
        [Fact]
        public void TheRecordInAProcessWithTheLibrariesOff()
        {
            if (Environment.GetEnvironmentVariable(OffMarkVariable) is not { Length: > 0 } mark) return;
            File.WriteAllLines(mark, new RegistrationRecorder(_root).Record("off", includeDiscoveryVariants: false));
        }

        // The golden's section <name>; with RecordVariable=1 the section is written back to the source tree instead of compared.
        private static void Compare(string name, List<string> actual)
        {
            string golden = Path.Combine(AppContext.BaseDirectory, "Cores", GoldenName);
            var sections = ReadSections(File.Exists(golden) ? File.ReadAllLines(golden) : Array.Empty<string>());
            if (Environment.GetEnvironmentVariable(RecordVariable) == "1")
            {
                string source = Path.Combine(Path.GetDirectoryName(SourceFile())!, GoldenName);
                var stored = ReadSections(File.Exists(source) ? File.ReadAllLines(source) : Array.Empty<string>());
                stored[name] = actual;
                File.WriteAllLines(source, stored.OrderBy(s => s.Key, StringComparer.Ordinal).SelectMany(s => new[] { $"[{s.Key}]" }.Concat(s.Value)));
                return;
            }
            Assert.True(sections.TryGetValue(name, out var expected), $"{GoldenName} has no [{name}] section; record it with {RecordVariable}=1");
            var missing = expected!.Except(actual).ToList();
            var extra = actual.Except(expected!).ToList();
            bool sameOrder = missing.Count == 0 && extra.Count == 0 && expected!.SequenceEqual(actual);
            Assert.True(sameOrder, $"[{name}] differs from {GoldenName}: {missing.Count} lines gone, {extra.Count} new" +
                string.Concat(missing.Take(40).Select(l => "\n- " + l)) + string.Concat(extra.Take(40).Select(l => "\n+ " + l)) +
                (missing.Count + extra.Count == 0 ? "\n(the same lines in another order)" : ""));
        }

        private static Dictionary<string, List<string>> ReadSections(IEnumerable<string> lines)
        {
            var sections = new Dictionary<string, List<string>>(StringComparer.Ordinal);
            List<string>? current = null;
            foreach (string line in lines)
            {
                if (line.StartsWith('[') && line.EndsWith(']')) sections[line[1..^1]] = current = new List<string>();
                else current?.Add(line);
            }
            return sections;
        }

        private static string SourceFile([CallerFilePath] string path = "") => path;
    }

    // One record of the registration's answers, as text lines a later build must reproduce - see EmuSen_CoreAPI.md §25.
    public sealed class RegistrationRecorder(string root)
    {
        private readonly List<string> _lines = new();
        private int _sandbox;

        private static readonly string[] Consoles = { "SNES", "NES", "GB", "N64", "Atari 2600" };
        private static readonly string[] ProbeExtensions = { ".sfc", ".smc", ".SFC", ".nes", ".gb", ".gbc", ".z64", ".n64", ".v64", ".fds", ".gba", ".tst", ".xyz", "" };
        private static readonly string[] SampleCodes =
        {
            "7E0DBE09", "7E000000", "DD62-6DAD", "C2A3-DFA4", "SXIOPO", "AAEAULPA", "GZSXSSVK", "00A-17B-C49", "091-17B-C49", "010A2BC0",
            "8033B152 0003", "D0033B15 0003", "800D7E6F", "0012:AB", "12AB:34", "FFFF", "", "ZZZZZZZZ",
        };

        private void Add(string key, object? value) => _lines.Add($"{key} = {Normalize(value switch { null => "<null>", string s => s, _ => value.ToString() })}");

        private string Normalize(string? text)
        {
            if (text is null) return "<null>";
            string bin = AppContext.BaseDirectory.TrimEnd(Path.DirectorySeparatorChar);
            text = text.Replace(root, "<root>").Replace(bin, "<bin>").Replace("\r", "\\r").Replace("\n", "\\n");
            foreach (string crate in new[] { "moonrt", "mercuryrt", "marsrt", "venusrt" })
                text = text.Replace($"lib{crate}.so", $"<lib {crate}>").Replace($"lib{crate}.dylib", $"<lib {crate}>").Replace($"{crate}.dll", $"<lib {crate}>");
            return text;
        }

        // A value the machine decides, MarsCore's one rasteriser thread per three cores, named rather than counted.
        private static string Value(string key, string value) => key == "RdpWorkers" && value == Math.Clamp(Environment.ProcessorCount / 3, 1, 4).ToString() ? "<one per three cores>" : value;

        private static string List(IEnumerable<object?> values) => "[" + string.Join(", ", values.Select(v => v?.ToString() ?? "<null>")) + "]";

        private static string Hash(byte[] data) => Convert.ToHexStringLower(SHA256.HashData(data))[..16];

        public List<string> Record(string world, bool includeDiscoveryVariants)
        {
            Use("catalog");
            Catalog();
            Engines(world);
            Factory(world, withBattery: true);
            Settings();
            Codecs();
            if (includeDiscoveryVariants)
            {
                string refused = RefusedDirectory();
                CoreDiscovery.UseDirectories(new[] { Path.Combine(root, "no-cores") });
                Engines("nothing-discovered");
                Factory("nothing-discovered", withBattery: false);
                CoreDiscovery.UseDirectories(new[] { refused });
                Engines("refused-discovered");
                Factory("refused-discovered", withBattery: false);
                CoreDiscovery.UseDirectories(null);
            }
            return _lines;
        }

        // A fresh DataStore and ConfigStore under the root, so no answer reads the machine's own configuration.
        private string Use(string name)
        {
            string dir = Path.Combine(root, $"{++_sandbox:D3}-{name}");
            Directory.CreateDirectory(dir);
            DataStore.OverrideDirectory = Path.Combine(dir, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(dir, "Config");
            return dir;
        }

        private void Catalog()
        {
            foreach (var (key, core) in CoreCatalog.Registry.OrderBy(r => r.Key, StringComparer.Ordinal)) Add($"catalog.registry[{key}]", core.DisplayName);
            Add("catalog.cores", List(CoreCatalog.Cores.Select(c => c.DisplayName)));
            Add("catalog.release-order", List(CoreCatalog.ConsolesInReleaseOrder.Select(c => c.DisplayName)));
            Add("catalog.filter-choices", List(CoreCatalog.FilterChoices));
            Add("catalog.rom-extensions", List(CoreCatalog.RomExtensions));
            Add("catalog.cheat-systems", List(CoreCatalog.SupportedCheatSystems.Order(StringComparer.Ordinal)));
            Add("catalog.all-consoles", CoreCatalog.AllConsoles);
            Add("catalog.engine-key", CoreCatalog.EngineKey);
            Add("catalog.gbc-shelf", CoreCatalog.GameBoyColorShelf);
            foreach (CoreDescriptor c in CoreCatalog.Cores)
            {
                string k = $"catalog.core[{c.DisplayName}]";
                Add(k + ".console", c.Console);
                Add(k + ".extensions", List(c.Extensions));
                Add(k + ".cheat-systems", List(c.CheatSystems ?? Array.Empty<string>()));
                Add(k + ".maker-year", $"{c.Manufacturer} {c.ReleaseYear}");
                Add(k + ".cover-aspect", c.CoverAspect.ToString("R", System.Globalization.CultureInfo.InvariantCulture));
                Add(k + ".openvgdb", List(c.OpenVgdbSystemNames));
                foreach (var (label, image) in SampleImages(c.Console))
                    Add($"{k}.openvgdb-bytes[{label}]", c.OpenVgdbBytes is { } f ? $"{f(image).Length} {Hash(f(image))}" : "<none>");
            }
            foreach (var s in CoreCatalog.ShelvesInReleaseOrder) Add($"catalog.shelf[{s.Name}]", $"{s.Label} {s.Core.DisplayName} {s.EsdeSystem} {s.EsdeFullName}");
            foreach (string name in new[] { "SNES (Venus)", "NES (Moon)", "Game Boy (Mercury)", CoreCatalog.GameBoyColorShelf, "Nintendo 64 (Mars)", "nes (moon)", "", "Nope" })
                Add($"catalog.shelf-by-name[{name}]", CoreCatalog.ShelfByName(name)?.Name);
            foreach (string name in new[] { "SNES (Venus)", "SNES", "snes", "NES", "nes (moon)", "GB", "GBC", "Game Boy (Mercury)", CoreCatalog.GameBoyColorShelf, "N64", "Nintendo 64 (Mars)", "", "Nope" })
                Add($"catalog.by-name[{name}]", $"{CoreCatalog.ByAnyName(name)?.DisplayName ?? "<null>"} / {CoreCatalog.ByDisplayName(name)?.DisplayName ?? "<null>"}");
            Add("catalog.by-name[<null>]", $"{CoreCatalog.ByAnyName(null)?.DisplayName ?? "<null>"} / {CoreCatalog.ByDisplayName(null)?.DisplayName ?? "<null>"}");
            foreach (string ext in ProbeExtensions)
                Add($"catalog.extension[{ext}]", $"{CoreCatalog.IsRomExtension(ext)} {CoreCatalog.ByExtension(ext)?.DisplayName ?? "<null>"} {CoreCatalog.ConsoleForRom("game" + ext) ?? "<null>"} supported={CoreFactory.IsSupported("game" + ext)}");
            string dir = Path.Combine(root, "shelves");
            Directory.CreateDirectory(dir);
            foreach (var (name, image) in new (string, byte[])[]
            {
                ("plain.gb", SyntheticGbRom.Build()), ("colour.gb", SyntheticGbRom.Build(cgbFlag: 0xC0)), ("only.gb", SyntheticGbRom.Build(cgbFlag: 0x80)),
                ("plain.gbc", SyntheticGbRom.Build()), ("game.nes", SyntheticNesRom.Build()), ("game.sfc", SyntheticRom.Build()), ("game.z64", SyntheticN64Rom.Build()), ("game.xyz", new byte[16]),
            })
            {
                string path = Path.Combine(dir, name);
                File.WriteAllBytes(path, image);
                Add($"catalog.shelf-for[{name}]", $"{CoreCatalog.ShelfFor(path) ?? "<null>"} colour={CoreCatalog.IsGameBoyColor(path)}");
            }
            foreach (string console in Consoles)
            {
                string k = $"catalog.console[{console}]";
                Add(k + ".buttons", List(CoreCatalog.ButtonsFor(console).Cast<object>()));
                Add(k + ".axes", List(CoreCatalog.AxesFor(console).Cast<object>()));
                Add(k + ".controls", List(CoreCatalog.ControlsFor(console).Cast<object>()));
                Add(k + ".system-ids", List(CoreCatalog.SystemIdsFor(console)));
                foreach (CoreSetting s in CoreCatalog.SettingsFor(console)) Setting(k + ".setting", s);
            }
            foreach (string id in new[] { "snes", "nes", "gb", "gbc", "n64", "tst", "" }) Add($"catalog.console-for-system[{id}]", CoreCatalog.ConsoleForSystem(id));
        }

        private void Setting(string prefix, CoreSetting s)
        {
            string k = $"{prefix}[{s.Key}]";
            Add(k, $"{s.Label} | {s.Kind} | default {Value(s.Key, s.Default)} | {s.Min}..{s.Max} | {List(s.Choices ?? Array.Empty<string>())}");
            Add(k + ".hint", s.Hint);
            if (s.Note is { } note) Add(k + ".note-at-defaults", note(key => CoreCatalog.SettingsFor("N64").Concat(CoreCatalog.SettingsFor("GB")).FirstOrDefault(x => x.Key == key)?.Default ?? ""));
        }

        // The engine row and the engine-name answers for every console, under the discovery in force.
        private void Engines(string world)
        {
            foreach (string console in Consoles)
            {
                string k = $"{world}.engine-row[{console}]";
                CoreSetting? row = CoreCatalog.EngineFor(console);
                if (row is null) Add(k, "<null>");
                else Setting(k, row);
                Add(k + ".discovered", List(CoreCatalog.DiscoveredEngines(console)));
                Add(k + ".chosen-default", CoreCatalog.EngineChosen(console, null));
                Add(k + ".chosen-stored", CoreCatalog.EngineChosen(console, "Stored"));
            }
            foreach (string name in EngineNames().Append("Stored").Append("Nope")) Add($"{world}.registered[{name}]", CoreCatalog.IsRegisteredEngine(name));
            Add($"{world}.registered[<null>]", CoreCatalog.IsRegisteredEngine(null));
            Add($"{world}.available", $"MoonRT={MoonRtCore.Available} MercuryRT={MercuryRtCore.Available} MarsRT={MarsRtCore.Available}");
            foreach (var found in CoreDiscovery.Found.OrderBy(c => c.Info.Id, StringComparer.Ordinal))
            {
                bool opens = found.Open() is not null;
                Add($"{world}.discovered[{found.Info.Id}]", $"{found.EngineName} systems={List(found.Info.Systems.Select(s => s.Id))} opens={opens}{(opens ? "" : " report=" + found.Report)}");
            }
        }

        private static IEnumerable<string> EngineNames() =>
            Consoles.SelectMany(c => CoreCatalog.EngineFor(c)?.Choices ?? Array.Empty<string>()).Concat(new[] { CoreCatalog.VenusEngine, CoreCatalog.MoonEngine, CoreCatalog.MoonRtEngine,
                CoreCatalog.MercuryEngine, CoreCatalog.MercuryRtEngine, CoreCatalog.MarsEngine, CoreCatalog.MarsRtEngine, "VenusRT (Rust)" }).Distinct().Order(StringComparer.Ordinal);

        // Synthetic images per console: what each is called, its extension and bytes.
        private static IEnumerable<(string Label, byte[] Image)> SampleImages(string console) => RomsFor(console).Select(r => (r.Label, r.Image));

        private static IEnumerable<(string Label, string Extension, byte[] Image)> RomsFor(string console) => console switch
        {
            "SNES" => new[] { ("plain", ".sfc", SyntheticRom.Build()), ("battery", ".smc", SnesWithSram()) },
            "NES" => new[] { ("plain", ".nes", SyntheticNesRom.Build()), ("battery", ".nes", SyntheticNesRom.Build(battery: true)) },
            "GB" => new[] { ("plain", ".gb", SyntheticGbRom.Build()), ("battery", ".gb", SyntheticGbRom.Build(cartridgeType: 0x03, ramSizeCode: 0x02)),
                ("colour-battery", ".gbc", SyntheticGbRom.Build(cartridgeType: 0x03, ramSizeCode: 0x02, cgbFlag: 0xC0)) },
            "N64" => new[] { ("plain", ".z64", SyntheticN64Rom.Build()), ("eeprom", ".z64", SyntheticN64Rom.BuildHomebrew(N64SaveType.Eeprom4k)), ("byteswapped", ".v64", SyntheticN64Rom.ToByteSwapped(SyntheticN64Rom.Build())) },
            _ => Array.Empty<(string, string, byte[])>(),
        };

        // A LoROM declaring 2 KiB of battery-backed SRAM: cartridge type $02, SRAM size code 1.
        private static byte[] SnesWithSram() => SyntheticRom.Build((0x7FD6, new byte[] { 0x02 }), (0x7FD8, new byte[] { 0x01 }));

        // For each console's images and each engine name a frontend could store: what is built, what it needs, what it says and where its battery goes.
        private void Factory(string world, bool withBattery)
        {
            foreach (string console in Consoles.Where(c => RomsFor(c).Any()))
            {
                string?[] engines = new string?[] { null }.Concat(CoreCatalog.EngineFor(console)?.Choices ?? Array.Empty<string>()).Append("Nope").Append(CoreCatalog.VenusEngine).Distinct().ToArray();
                foreach (var (label, ext, image) in RomsFor(console))
                {
                    if (!withBattery && label != "plain") continue;
                    foreach (string? engine in engines)
                    {
                        string k = $"{world}.factory[{console} {label} {engine ?? "<null>"}]";
                        string dir = Use("factory");
                        string rom = Path.Combine(dir, "Roms", "Game" + ext);
                        Directory.CreateDirectory(Path.GetDirectoryName(rom)!);
                        File.WriteAllBytes(rom, image);
                        if (withBattery) PlacePreviousSaves(rom);
                        Add(k + ".supported", CoreFactory.IsSupported(rom));
                        ICore probe = CoreFactory.ForFirmwareProbe(rom, engine);
                        Add(k + ".probe", probe.GetType().Name);
                        foreach (FirmwareRequest f in probe.GetFirmwareRequirements(rom))
                            Add(k + ".firmware", $"{f.CoreName} | {f.ChipName} | {f.FileName} | {f.Size} | {f.Purpose} | {List(f.AlternateNames)} | {List(f.Parts.Select(p => List(p)))}");
                        (probe as IDisposable)?.Dispose();
                        CoreBundle bundle;
                        try { bundle = CoreFactory.Load(rom, engine: engine); }
                        catch (Exception e) { Add(k + ".load", $"{e.GetType().Name}: {e.Message}"); continue; }
                        Bundle(k, bundle);
                        if (withBattery) bundle.Core.SaveSram();
                        (bundle.Core as IDisposable)?.Dispose();
                        if (withBattery) Files(k + ".file", dir);
                    }
                }
                if (!withBattery) continue;
                string config = Use("configured");
                string game = Path.Combine(config, "Game" + RomsFor(console).First().Extension);
                Add($"{world}.configured[{console} nothing stored]", CoreFactory.ConfiguredEngine(game));
                foreach (string? stored in engines.Skip(1))
                {
                    GraphicsConfig graphics = GraphicsConfig.Load();
                    graphics.SetValue(console, CoreCatalog.EngineKey, stored!);
                    graphics.Save();
                    Add($"{world}.configured[{console} {stored}]", CoreFactory.ConfiguredEngine(game));
                }
            }
        }

        private void Bundle(string k, CoreBundle b)
        {
            ICore c = b.Core;
            Add(k + ".core", c.GetType().Name);
            Add(k + ".notice", b.Notice);
            Add(k + ".debug-target", b.DebugTarget.GetType().Name);
            Add(k + ".trace-switch", b.CpuTraceSwitch?.GetType().Name);
            Add(k + ".codec-auto", Codec(b.CheatAutoDetectCodec));
            Add(k + ".codec-explicit", Codec(b.CheatExplicitCodec));
            Add(k + ".identity", $"{c.CoreName} {c.ScreenWidth}x{c.ScreenHeight} {c.FrameRateHz.ToString("R", System.Globalization.CultureInfo.InvariantCulture)} Hz {c.AudioSampleRate} loaded={c.IsRomLoaded} frames={c.TotalFrames}");
            Add(k + ".buttons", List(c.SupportedButtons.Cast<object>()));
            Add(k + ".axes", List(c.SupportedAxes.Cast<object>()));
            Add(k + ".state-version", (c as IStateFormat)?.StateVersion);
            Add(k + ".features", EngineFeatures.Of(c));
            if (c is ICoreSettings settings)
                foreach (CoreSetting s in settings.Settings) Add($"{k}.setting[{s.Key}]", $"{s.Label} | {s.Kind} | default {Value(s.Key, s.Default)} | {List(s.Choices ?? Array.Empty<string>())} | now {Value(s.Key, settings.Get(s.Key))}");
        }

        private static string Codec(ICheatCodeCodec? codec) => codec is null ? "<null>" : $"{codec.Name} | {codec.Kind} | {codec.SpaceName ?? "<null>"}";

        // Each codec a frontend is handed for a console name, and what it makes of a fixed set of codes.
        private void Codecs()
        {
            var seen = new Dictionary<string, ICheatCodeCodec>(StringComparer.Ordinal);
            void Pair(string k, (ICheatCodeCodec? Auto, ICheatCodeCodec? Explicit) pair)
            {
                Add(k, $"{Codec(pair.Auto)} / {Codec(pair.Explicit)}");
                foreach (var c in new[] { pair.Auto, pair.Explicit }) if (c is not null) seen.TryAdd($"{c.Name} {c.Kind} {c.SpaceName}", c);
            }
            Pair("codecs.default", CoreFactory.DefaultCheatCodecs);
            foreach (string? name in new[] { null, "", "SNES", "SNES (Venus)", "NES", "NES (Moon)", "GB", "GBC", "Game Boy (Mercury)", CoreCatalog.GameBoyColorShelf, "N64", "Nintendo 64 (Mars)", "Nope" })
                Pair($"codecs.for[{name ?? "<null>"}]", CoreFactory.CheatCodecsFor(name));
            foreach (var (key, codec) in seen.OrderBy(s => s.Key, StringComparer.Ordinal))
                foreach (string code in SampleCodes)
                {
                    string answer;
                    try
                    {
                        answer = !codec.CanDecode(code) ? "no" : $"{codec.Decode(code)} compare={codec.DecodeCompare(code)?.ToString() ?? "-"} writes={List((codec.DecodeWrites(code) ?? Array.Empty<CheatWrite>()).Select(w => $"{w.Space}:{w.Address:X}={w.Value:X}"))}";
                    }
                    catch (Exception e) { answer = $"{e.GetType().Name}: {e.Message}"; }
                    Add($"codecs.decode[{key}][{code}]", answer);
                }
        }

        // The settings each engine a console can run offers through ICoreSettings, as built and before a game.
        private void Settings()
        {
            foreach (string console in Consoles)
            {
                foreach (string engine in CoreCatalog.EngineFor(console)?.Choices ?? Array.Empty<string>())
                {
                    if (RomsFor(console).FirstOrDefault() is not { Extension: { } ext }) continue;
                    ICore core = CoreFactory.Create("Game" + ext, engine: engine);
                    Add($"settings[{console} {engine}].core", core.GetType().Name);
                    if (core is ICoreSettings s)
                        foreach (CoreSetting row in s.Settings) Setting($"settings[{console} {engine}].row", row);
                    (core as IDisposable)?.Dispose();
                }
            }
        }

        // Where an older build kept each console's save, filled with a mark, so a load that copies one in shows it.
        private static void PlacePreviousSaves(string rom)
        {
            foreach (string console in new[] { BatterySave.Nes, BatterySave.Snes, BatterySave.N64, BatterySave.GameBoy, BatterySave.GameBoyColor })
                if (BatterySave.PreviousHome(rom, console) is { } path && !File.Exists(path))
                {
                    Directory.CreateDirectory(Path.GetDirectoryName(path)!);
                    File.WriteAllBytes(path, Enumerable.Repeat((byte)0x5A, 64).ToArray());
                }
        }

        private void Files(string k, string dir)
        {
            foreach (string file in Directory.EnumerateFiles(dir, "*", SearchOption.AllDirectories).Order(StringComparer.Ordinal))
            {
                string relative = Path.GetRelativePath(dir, file).Replace('\\', '/');
                if (relative.StartsWith("Roms/Game.", StringComparison.Ordinal) && !relative.EndsWith(".srm", StringComparison.Ordinal)) continue;
                if (relative.StartsWith("Home/Logs/", StringComparison.Ordinal)) continue;
                byte[] data = File.ReadAllBytes(file);
                Add(k, $"{relative} {data.Length} {Hash(data)}");
            }
        }

        // VenusRT's and MoonRT's sidecars beside a file that is not their library, so opening either is refused.
        private string RefusedDirectory()
        {
            string dir = Path.Combine(root, "refused");
            Directory.CreateDirectory(dir);
            foreach (var found in CoreDiscovery.Found)
            {
                var sidecar = (JsonObject)JsonNode.Parse(File.ReadAllText(found.Sidecar.SidecarPath))!;
                File.WriteAllText(Path.Combine(dir, Path.GetFileName(found.Sidecar.SidecarPath)), sidecar.ToJsonString());
                File.WriteAllBytes(Path.Combine(dir, Path.GetFileName(found.Sidecar.LibraryPath)), new byte[] { 0x7F, (byte)'E', (byte)'L', (byte)'F' });
            }
            return dir;
        }
    }
}
