using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Native;
using EmuSen.Galaxia.Text;

namespace EmuSen.WiseMan.Galaxia
{
    // The config half: what System.Text.Json writes and reads against what the platform library does - see EmuSen_RustPlatform.md §11.5.
    public partial class GalaxiaParityTests
    {
        private static string Json<T>(T value) => JsonSerializer.Serialize(value, ConfigJson.Options);

        // The config directory each side of a file comparison uses, with the old place config sat beside it.
        private void Side(string side)
        {
            ConfigStore.OverrideDirectory = At(side, "etc");
            ConfigStore.OverrideLegacyDirectory = At(side, "legacy");
        }

        // One scenario run on each side with its own folders: what the file call answered, what it left on disk, and what it said.
        private void FileBothWays<T>(string name, Action<string> arrange, Func<ConfigFile<T>, bool, object?> act, string? category = null, string file = "settings.json") where T : class
        {
            var answers = new List<string>();
            var said = new List<List<string>>();
            foreach (string side in new[] { "native" })
            {
                string root = At(side + "-" + name);
                Directory.CreateDirectory(root);
                ConfigStore.OverrideDirectory = Path.Combine(root, "etc");
                ConfigStore.OverrideLegacyDirectory = Path.Combine(root, "legacy");
                arrange(root);
                Stamp(root);
                var config = category is null ? new ConfigFile<T>(file) : new ConfigFile<T>(category, file);
                object? answer = act(config, side == "native");
                string shown = answer switch { null => "null", bool flag => flag.ToString(), string text => text, _ => JsonSerializer.Serialize(answer, answer.GetType(), ConfigJson.Options) };
                string folders = string.Join(",", Directory.EnumerateDirectories(root, "*", SearchOption.AllDirectories).Select(d => Path.GetRelativePath(root, d)).OrderBy(d => d, StringComparer.Ordinal));
                answers.Add($"{shown} | error {(config.LastLoadError is null ? "none" : "said")} | {string.Join(" ; ", Listing(root).Select(JustWritten))} | folders {folders}");
                said.Add(Reported().Select(m => m.Replace(root, "<root>")).ToList());
            }
            G($"{typeof(T).Name} {name}", answers[0]);
            // The frame is Galaxia's and exact; the words between were each implementation's own.
            G($"{typeof(T).Name} {name} said", said[0].Select(native => native[..native.IndexOf(": ", StringComparison.Ordinal)]));
            foreach (string native in said[0])
            {
                Assert.StartsWith("<root>" + Path.DirectorySeparatorChar, native);
                Assert.EndsWith(" Falling back to defaults.", native);
            }
        }

        // A listing's line with the time of a file written during the test replaced, since the two sides write a moment apart; a copy keeps its source's time and is compared.
        private static string JustWritten(string line)
        {
            int at = line.LastIndexOf(" | ", StringComparison.Ordinal);
            return at > 0 && long.TryParse(line[(at + 3)..], out long ticks) && ticks > DateTime.UtcNow.AddMinutes(-10).Ticks ? line[..at] + " | just written" : line;
        }

        private static T? LoadSide<T>(ConfigFile<T> file, bool native) where T : class => file.Load();

        private static bool SaveSide<T>(ConfigFile<T> file, bool native, T value) where T : class => file.Save(value);

        private void FileScenarios<T>(Func<Random, T> make, string good, string bad, T? unwritable = null) where T : class
        {
            var random = new Random(typeof(T).Name.Length);
            T value = make(random);
            string etc = "etc", legacy = "legacy";
            void Put_(string root, string folder, string name, string text) => Put(Path.Combine(root, folder, name), text);

            FileBothWays<T>("missing", _ => { }, (file, native) => LoadSide(file, native));
            FileBothWays<T>("good", root => Put_(root, etc, "settings.json", good), (file, native) => LoadSide(file, native));
            FileBothWays<T>("bad", root => Put_(root, etc, "settings.json", bad), (file, native) => LoadSide(file, native));
            FileBothWays<T>("not json", root => Put_(root, etc, "settings.json", "{ not json at all"), (file, native) => LoadSide(file, native));
            FileBothWays<T>("null", root => Put_(root, etc, "settings.json", " null "), (file, native) => LoadSide(file, native));
            FileBothWays<T>("empty", root => Put_(root, etc, "settings.json", ""), (file, native) => LoadSide(file, native));
            FileBothWays<T>("a folder in its place", root => Directory.CreateDirectory(Path.Combine(root, etc, "settings.json")), (file, native) => LoadSide(file, native));
            FileBothWays<T>("an error cleared by a good load", root => Put_(root, etc, "settings.json", bad), (file, native) =>
            {
                LoadSide(file, native);
                File.WriteAllText(file.Path, good);
                return LoadSide(file, native);
            });
            FileBothWays<T>("an error kept by a missing file", root => Put_(root, etc, "settings.json", bad), (file, native) =>
            {
                LoadSide(file, native);
                File.Delete(file.Path);
                return LoadSide(file, native);
            });
            FileBothWays<T>("from before Galaxia", root => Put_(root, legacy, "settings.json", good), (file, native) => LoadSide(file, native));
            FileBothWays<T>("from before Galaxia, twice", root => Put_(root, legacy, "settings.json", good), (file, native) => Json(LoadSide(file, native)) + Json(LoadSide(file, native)));
            FileBothWays<T>("both places", root => { Put_(root, legacy, "settings.json", bad); Put_(root, etc, "settings.json", good); }, (file, native) => LoadSide(file, native));
            FileBothWays<T>("the old place, no copy possible", root => { Put_(root, legacy, "settings.json", good); Directory.CreateDirectory(Path.Combine(root, etc, "settings.json")); }, (file, native) => LoadSide(file, native));
            FileBothWays<T>("a category never sat in the old place", root => Put_(root, Path.Combine(legacy, "group"), "settings.json", good), (file, native) => LoadSide(file, native), category: "group");
            FileBothWays<T>("a category, with a file of its name in the old place", root => Put_(root, legacy, "settings.json", good), (file, native) => LoadSide(file, native), category: "group");
            FileBothWays<T>("a category", root => Put_(root, Path.Combine(etc, "group"), "settings.json", good), (file, native) => LoadSide(file, native), category: "group");

            FileBothWays<T>("saved", _ => { }, (file, native) => SaveSide(file, native, value));
            FileBothWays<T>("saved over", root => Put_(root, etc, "settings.json", bad), (file, native) => SaveSide(file, native, value) && SaveSide(file, native, value));
            FileBothWays<T>("saved in a category", _ => { }, (file, native) => SaveSide(file, native, value), category: "group");
            FileBothWays<T>("saved where a file blocks the folder", root => Put(Path.Combine(root, etc), "a file where the folder should be"), (file, native) => SaveSide(file, native, value));
            FileBothWays<T>("saved over a folder", root => Directory.CreateDirectory(Path.Combine(root, etc, "settings.json")), (file, native) => SaveSide(file, native, value));
            FileBothWays<T>("saved, then read by the other", _ => { }, (file, native) => SaveSide(file, native, value) ? LoadSide(file, !native) : null);
            FileBothWays<T>("saved null", _ => { }, (file, native) => SaveSide(file, native, null!));
            // What cannot be serialized is not saved, and its folder is made all the same.
            if (unwritable is not null) FileBothWays<T>("saved what cannot be written", _ => { }, (file, native) => SaveSide(file, native, unwritable), category: "group");

            FileBothWays<T>("deleted", root => Put_(root, etc, "settings.json", good), (file, native) => file.Delete());
            FileBothWays<T>("deleted when absent", root => Directory.CreateDirectory(Path.Combine(root, etc)), (file, native) => file.Delete());
            FileBothWays<T>("deleted with no folder", _ => { }, (file, native) => file.Delete());
            FileBothWays<T>("deleted when a folder", root => Directory.CreateDirectory(Path.Combine(root, etc, "settings.json")), (file, native) => file.Delete());
        }

        [Fact]
        public void A_file_whose_type_is_a_frontends_goes_through_the_same_file_rules()
        {
            _case = nameof(A_file_whose_type_is_a_frontends_goes_through_the_same_file_rules);
            FileScenarios(random => new Dictionary<string, Dictionary<string, int>> { ["NES"] = new() { ["Up"] = random.Next(100), ["é<>"] = -1 }, ["SNES Player 2"] = new() },
                "{\"NES\":{\"Up\":38,/* a key */\"B\":90,},}", "{\"NES\":{\"Up\":\"DPadUpp\"}}");

            // The text reaches C# decoded as .NET decodes a file, whatever mark it carries.
            foreach (Encoding encoding in new Encoding[] { new UTF8Encoding(true), Encoding.Unicode, Encoding.BigEndianUnicode, new UTF32Encoding(false, true), new UTF32Encoding(true, true) })
                FileBothWays<Dictionary<string, string>>("marked " + encoding.WebName + encoding.GetPreamble().Length,
                    root => { Directory.CreateDirectory(Path.Combine(root, "etc")); File.WriteAllBytes(Path.Combine(root, "etc", "settings.json"), encoding.GetPreamble().Concat(encoding.GetBytes("{\"é\":\"日本 😀\"}")).ToArray()); },
                    (file, native) => LoadSide(file, native));
            FileBothWays<Dictionary<string, string>>("not UTF-8",
                root => { Directory.CreateDirectory(Path.Combine(root, "etc")); File.WriteAllBytes(Path.Combine(root, "etc", "settings.json"), new byte[] { (byte)'{', (byte)'"', 0xFF, (byte)'"', (byte)':', (byte)'"', 0xC3, (byte)'"', (byte)'}' }); },
                (file, native) => LoadSide(file, native));
        }

        [Fact]
        public void Settings_are_upgraded_as_the_csharp_upgraded_them_whether_read_or_made()
        {
            (string Name, string? File)[] cases =
            {
                ("no file", null), ("the legacy default", "{\"SelectedCore\":\"SNES (Venus)\",\"OnlineCovers\":false}"), ("the legacy default, chosen since", "{\"SelectedCore\":\"SNES (Venus)\",\"SelectedCoreUpgraded\":true}"),
                ("a choice", "{\"SelectedCore\":\"NES (Moon)\",\"OnlineCovers\":true,\"LogDirectory\":\"/logs/é\"}"), ("no core at all", "{\"SelectedCore\":null}"), ("a null document", "null"),
                ("a file that will not load", "{\"Volume\":\"loud\"}"), ("a different case", "{\"selectedcore\":\"SNES (Venus)\",\"SELECTEDCOREUPGRADED\":false}"),
            };
            foreach ((string name, string? file) in cases)
            {
                Side("native-upgrade-" + name);
                if (file is not null) Put(Path.Combine(ConfigStore.OverrideDirectory!, "appsettings.json"), file);
                string loaded = G(name, Json(AppSettings.LoadNative()));
                G(name + " said", Reported().Count);
                Assert.Contains("\"SelectedCoreUpgraded\": true", loaded);
                Assert.DoesNotContain("OnlineCovers", loaded);
            }
            Assert.Equal(Json(new AudioConfig()), Json(ConfigFile<AudioConfig>.NewNative(upgrade: false)));
            Assert.Equal(Json(new GraphicsConfig()), Json(ConfigFile<GraphicsConfig>.NewNative(upgrade: false)));
        }

        [Fact]
        public void The_log_folder_is_chosen_as_the_csharp_chose_it()
        {
            ErrorLog.DirectoryOverride = null;
            Put(At("blocker"), "a file");
            (string Name, string? Settings)[] cases =
            {
                ("no settings", null), ("no folder named", "{}"), ("a folder that can be made", $"{{\"LogDirectory\":{JsonSerializer.Serialize(At("made", "Logs é"))}}}"),
                ("a folder that cannot", $"{{\"LogDirectory\":{JsonSerializer.Serialize(At("blocker", "Logs"))}}}"), ("a blank folder", "{\"LogDirectory\":\"  \"}"), ("settings that will not load", "{\"LogDirectory\":5}"),
            };
            foreach ((string name, string? settings) in cases)
            {
                Side("native-root-" + name);
                if (settings is not null) Put(Path.Combine(ConfigStore.OverrideDirectory!, "appsettings.json"), settings);
                G(name, GalaxiaNative.LogRoot(null));
                G(name + " said", Reported().Count);
            }
            Assert.True(Directory.Exists(At("made", "Logs é")));
            G("default", GalaxiaNative.Directory(GalaxiaDirectory.LogDefault));
            Assert.Equal(At("given"), GalaxiaNative.LogRoot(At("given")));

            foreach (string? directory in new[] { null, "", " ", "\t", At("usable", "a", "b"), At("blocker", "under"), At("blocker") })
            {
                G($"usable [{directory}]", GalaxiaNative.LogUsable(directory));
                if (directory is not null && Directory.Exists(At("usable"))) Directory.Delete(At("usable"), recursive: true);
            }
        }

    }
}
