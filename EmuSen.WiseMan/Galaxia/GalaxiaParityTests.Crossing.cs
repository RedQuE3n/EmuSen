using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Native;

namespace EmuSen.WiseMan.Galaxia
{
    // Text the library cannot take, asked of every facade that hands it a caller's text - see EmuSen_RustPlatform.md §21.
    public partial class GalaxiaParityTests
    {
        // UTF-16 with half a surrogate pair: a high alone, a low alone, a high at the end, a pair the wrong way round.
        private static readonly string[] Unpaired = { "a\uD800b", "a\uDC00b", "ab\uD83D", "\uDE00\uD83D" };

        private static string Answered(Func<object?> act, string root)
        {
            try
            {
                object? value = act();
                string shown = value switch
                {
                    null => "null",
                    byte[] bytes => Convert.ToHexString(bytes),
                    CheatFile list => Json(list),
                    IEnumerable<string> items => "[" + string.Join("|", items) + "]",
                    _ => value.ToString()!,
                };
                return shown.Replace(root, "<root>");
            }
            catch (Exception fault)
            {
                return "threw " + fault.GetType().Name;
            }
        }

        // A log written a moment apart on each side, as its line past the time, taken away so the listing does not hold the time.
        private static string Logged(string? written)
        {
            if (written is null || !File.Exists(written)) return "nothing logged";
            string line = File.ReadAllText(written);
            File.Delete(written);
            return line[23..];
        }

        // One case run twice, each in a folder of its own, through the facade and through the C#: the answers, the files left and the words said must agree.
        private void BothAlike(string name, Func<string, object?> facade, Func<string, object?> csharp)
        {
            var seen = new List<string>();
            foreach ((string side, Func<string, object?> act) in new[] { ("facade", facade), ("csharp", csharp) })
            {
                string root = At("text", name, side);
                Directory.CreateDirectory(root);
                ConfigStore.OverrideDirectory = Path.Combine(root, "etc");
                ConfigStore.OverrideLegacyDirectory = Path.Combine(root, "legacy");
                string answer = Answered(() => act(root), root);
                seen.Add($"{answer} | {string.Join(" ; ", Listing(root).Select(JustWritten))} | said {string.Join(" || ", Reported()).Replace(root, "<root>")}");
            }
            Assert.True(seen[0] == seen[1], $"{name}:\n  facade: {seen[0]}\n  C#: {seen[1]}");
        }

        [Fact]
        public void Text_with_no_utf8_is_answered_by_every_facade_as_the_csharp_answers_it()
        {
            Assert.Equal(GalaxiaNative.Active, GalaxiaNative.Takes("é 日本 😀", null));
            DataStore.OverrideDirectory = At("data");
            Side("text-paths");
            for (int i = 0; i < Unpaired.Length; i++)
            {
                string text = Unpaired[i], rom = At("roms", text + ".sfc"), folder = At(text);
                Assert.False(GalaxiaNative.Takes(text));
                Assert.False(GalaxiaNative.Takes("SNES", text));
                // The binding refuses such text before the call, which is why no facade may hand it over.
                Assert.ThrowsAny<ArgumentException>(() => GalaxiaNative.SavePath(GalaxiaSave.Sram, rom, "SNES"));

                Assert.Equal(SaveLibrary.Managed.SramPathFor(rom, "SNES"), SaveLibrary.SramPathFor(rom, "SNES"));
                Assert.Equal(SaveLibrary.Managed.SramPathFor("x.sfc", text), SaveLibrary.SramPathFor("x.sfc", text));
                Assert.Equal(SaveLibrary.Managed.FlatSramPathFor(rom), SaveLibrary.FlatSramPathFor(rom));
                Assert.Equal(SaveLibrary.Managed.StatePathFor(rom, 2, null), SaveLibrary.StatePathFor(rom, 2));
                Assert.Equal(SaveLibrary.Managed.StatePathFor("x.sfc", 1, folder), SaveLibrary.StatePathFor("x.sfc", 1, folder));
                Assert.Equal(SaveLibrary.Managed.ResumeStatePathFor(rom, null), SaveLibrary.ResumeStatePathFor(rom));
                Assert.Equal(SaveLibrary.Managed.ResumeStatePathFor("x.sfc", folder), SaveLibrary.ResumeStatePathFor("x.sfc", folder));
                Assert.Equal(SaveLibrary.Managed.PicturePathFor(rom), SaveLibrary.PicturePathFor(rom));
                Assert.Equal(ConfigStore.Managed.For(text), ConfigStore.For(text));
                Assert.Equal(ConfigStore.Managed.For(text, "list.json"), ConfigStore.For(text, "list.json"));
                Assert.Equal(ConfigStore.Managed.LegacyPathFor(text), ConfigStore.LegacyPathFor(text));
                Assert.Equal(ConfigRoot.Managed.ComputeFor(folder, false, At("home")), ConfigRoot.ComputeFor(folder, false, At("home")));
                Assert.Equal(ConfigRoot.Managed.ComputeFor(At("base"), true, folder), ConfigRoot.ComputeFor(At("base"), true, folder));
                Assert.Equal(ConfigRoot.Managed.MacDataDirectoryFor(folder), ConfigRoot.MacDataDirectoryFor(folder));
                Assert.Equal(ConfigRoot.Managed.BundleContentsFor(Path.Combine(folder, "Contents", "MacOS")), ConfigRoot.BundleContentsFor(Path.Combine(folder, "Contents", "MacOS")));
                Assert.Equal(ConfigRoot.Managed.SeedDirectoryFor(Path.Combine(folder, "Contents", "MacOS"), true), ConfigRoot.SeedDirectoryFor(Path.Combine(folder, "Contents", "MacOS"), true));
                Assert.Equal(DataMigration.Managed.LegacyRootFor(folder), DataMigration.LegacyRootFor(folder));
                Assert.Equal(CheatFile.Managed.IsValidName(text), CheatFile.IsValidName(text));

                string Rom(string root) => Put(Path.Combine(root, "roms", text + ".sfc"), "rom " + i);
                CheatFile List() => new() { Cheats = { new CheatFileEntry { Description = "cheat " + text } } };
                BothAlike($"save {i}", root => AtomicFile.Write(Path.Combine(root, "Saves", text + ".srm"), new byte[] { 1, 2, 3 }) + " " + Answered(() => AtomicFile.TryRead(Path.Combine(root, "Saves", text + ".srm")), root),
                    root => AtomicFile.Managed.Write(Path.Combine(root, "Saves", text + ".srm"), new byte[] { 1, 2, 3 }) + " " + Answered(() => AtomicFile.Managed.TryRead(Path.Combine(root, "Saves", text + ".srm")), root));
                BothAlike($"hash {i}", root => RomHash.Md5(Rom(root)), root => RomHash.Managed.Md5(Rom(root)));
                BothAlike($"cheats to a path {i}", root => CheatFile.SaveTo(Path.Combine(root, text, "list.json"), List()) + " " + Answered(() => CheatFile.LoadFrom(Path.Combine(root, text, "list.json")), root),
                    root => CheatFile.Managed.SaveTo(Path.Combine(root, text, "list.json"), List()) + " " + Answered(() => CheatFile.Managed.LoadFrom(Path.Combine(root, text, "list.json")), root));
                BothAlike($"config file {i}", root =>
                {
                    var file = new ConfigFile<CheatFile>(Path.Combine("cheats", text), text + ".json");
                    return $"{file.Save(List())} {Answered(() => file.Load(), root)} {file.Delete()}";
                }, root =>
                {
                    var file = new ConfigFile<CheatFile>(Path.Combine("cheats", text), text + ".json");
                    return $"{file.SaveManaged(List())} {Answered(() => file.LoadManaged(), root)} {file.DeleteManaged()}";
                });
                BothAlike($"a game's list {i}", root =>
                {
                    GameCheats game = GameCheats.For("SNES", Rom(root))!;
                    return $"{game.Name} {game.SaveIfChanged(List())} {Answered(() => game.Load(), root)} {game.SaveIfChanged(List())}";
                }, root =>
                {
                    GameCheats game = GameCheats.Managed.For("SNES", Rom(root))!;
                    return $"{game.Name} {game.SaveIfChanged(List())} {Answered(() => game.Load(), root)} {game.SaveIfChanged(List())}";
                });

                string Legacy(string root)
                {
                    string legacy = Path.Combine(root, "legacy " + text);
                    Put(Path.Combine(legacy, "Saves", text + ".srm"), "save");
                    Put(Path.Combine(legacy, "Games", "SNES", text + ".sfc"), "rom");
                    return legacy;
                }
                BothAlike($"migration {i}", root => DataMigration.Run(Legacy(root), Path.Combine(root, "home " + text)), root => DataMigration.Managed.Run(Legacy(root), Path.Combine(root, "home " + text)));
                BothAlike($"seed {i}", root => DataMigration.SeedFromBundle(Legacy(root), Path.Combine(root, "home")), root => DataMigration.Managed.SeedFromBundle(Legacy(root), Path.Combine(root, "home")));
                BothAlike($"tree {i}", root => DataMigration.CopyTree(Legacy(root), Path.Combine(root, "copy")), root => DataMigration.Managed.CopyTree(Legacy(root), Path.Combine(root, "copy")));
                BothAlike($"remaining {i}", root => DataMigration.RemainingLibraryDirectories(Legacy(root)), root => DataMigration.Managed.RemainingLibraryDirectories(Legacy(root)));

                DateTime day = new(2026, 10, 10, 12, 0, 0);
                BothAlike($"log {i}", root =>
                {
                    ErrorLog.DirectoryOverride = Path.Combine(root, "logs " + text);
                    ErrorLog.ResetForTests();
                    return $"{ErrorLog.Root()} {ErrorLog.PathFor(day)} {Logged(ErrorLog.Error("test", "m"))}";
                }, root =>
                {
                    ErrorLog.DirectoryOverride = Path.Combine(root, "logs " + text);
                    ErrorLog.ResetForTests();
                    return $"{ErrorLog.Managed.Root()} {ErrorLog.Managed.PathFor(day)} {Logged(ErrorLog.Managed.Write(DateTime.Now, "ERROR", "test", "m", null, null))}";
                });
                ErrorLog.DirectoryOverride = _logOverride;
            }
        }

        // The words of a failed write come back as a list, each item ended by a NUL, so a path holding one would be said as two messages.
        [Fact]
        public void A_path_holding_a_nul_is_said_as_one_message_by_whichever_side_writes_it()
        {
            Assert.False(GalaxiaNative.Takes("a\0b", listed: true));
            Assert.Equal(GalaxiaNative.Active, GalaxiaNative.Takes("a\0b"));
            BothAlike("nul write", root => AtomicFile.Write(Path.Combine(root, "a\0b.srm"), new byte[] { 1 }), root => AtomicFile.Managed.Write(Path.Combine(root, "a\0b.srm"), new byte[] { 1 }));
            BothAlike("nul read", root => AtomicFile.TryRead(Path.Combine(root, "a\0b.srm")), root => AtomicFile.Managed.TryRead(Path.Combine(root, "a\0b.srm")));
        }
    }
}
