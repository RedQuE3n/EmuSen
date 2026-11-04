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
    // Text the library cannot take as it is, asked of every facade that hands it a caller's text: half a surrogate pair crosses as U+FFFD, and a NUL in a call quoted back in a list is refused - see EmuSen_RustPlatform.md §21.6.
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

        // One case run twice, each in a folder of its own, with the text and with the text as .NET's encoder writes it: the answers, the files left and the words said must agree.
        private void BothAlike(string name, string text, Func<string, string, object?> act)
        {
            var seen = new List<string>();
            foreach ((string side, string given) in new[] { ("half", text), ("mended", PlatformText.Mend(text)) })
            {
                string root = At("text", name, side);
                Directory.CreateDirectory(root);
                ConfigStore.OverrideDirectory = Path.Combine(root, "etc");
                ConfigStore.OverrideLegacyDirectory = Path.Combine(root, "legacy");
                string answer = Answered(() => act(root, given), root);
                seen.Add($"{answer} | {string.Join(" ; ", Listing(root).Select(JustWritten))} | said {string.Join(" || ", Reported()).Replace(root, "<root>")}");
            }
            Assert.True(seen[0] == seen[1], $"{name}:\n  half a pair: {seen[0]}\n  U+FFFD:      {seen[1]}");
        }

        [Fact]
        public void Half_a_surrogate_pair_is_answered_by_every_facade_as_the_text_with_U_FFFD()
        {
            DataStore.OverrideDirectory = At("data");
            Side("text-paths");
            for (int i = 0; i < Unpaired.Length; i++)
            {
                string text = Unpaired[i], mended = PlatformText.Mend(text);
                Assert.NotEqual(text, mended);
                string Rom_(string t) => At("roms", t + ".sfc");
                string Folder(string t) => At(t);
                void Same(Func<string, object?> facade) => Assert.Equal(Answered(() => facade(mended), _temp), Answered(() => facade(text), _temp));

                Same(t => SaveLibrary.SramPathFor(Rom_(t), "SNES"));
                Same(t => SaveLibrary.SramPathFor("x.sfc", t));
                Same(t => SaveLibrary.FlatSramPathFor(Rom_(t)));
                Same(t => SaveLibrary.StatePathFor(Rom_(t), 2));
                Same(t => SaveLibrary.StatePathFor("x.sfc", 1, Folder(t)));
                Same(t => SaveLibrary.ResumeStatePathFor(Rom_(t)));
                Same(t => SaveLibrary.ResumeStatePathFor("x.sfc", Folder(t)));
                Same(t => SaveLibrary.PicturePathFor(Rom_(t)));
                Same(t => ConfigStore.For(t));
                Same(t => ConfigStore.For(t, "list.json"));
                Same(t => ConfigStore.LegacyPathFor(t));
                Same(t => ConfigRoot.ComputeFor(Folder(t), false, At("home")));
                Same(t => ConfigRoot.ComputeFor(At("base"), true, Folder(t)));
                Same(t => ConfigRoot.MacDataDirectoryFor(Folder(t)));
                Same(t => ConfigRoot.BundleContentsFor(Path.Combine(Folder(t), "Contents", "MacOS")));
                Same(t => ConfigRoot.SeedDirectoryFor(Path.Combine(Folder(t), "Contents", "MacOS"), true));
                Same(t => DataMigration.LegacyRootFor(Folder(t)));
                Same(t => CheatFile.IsValidName(t));
                Same(t => global::EmuSen.Galaxia.Text.Suggestion.Hint(t, new[] { t, "cheat" }));
                // A path handed back holds U+FFFD where the caller's held the half.
                Assert.Equal(Path.Combine(DataStore.Saves, mended + ".srm"), SaveLibrary.FlatSramPathFor(text + ".srm"));

                string Rom(string root, string t) => Put(Path.Combine(root, "roms", t + ".sfc"), "rom " + i);
                CheatFile List(string t) => new() { Cheats = { new CheatFileEntry { Description = "cheat " + t } } };
                BothAlike($"save {i}", text, (root, t) => AtomicFile.Write(Path.Combine(root, "Saves", t + ".srm"), new byte[] { 1, 2, 3 }) + " " + Answered(() => AtomicFile.TryRead(Path.Combine(root, "Saves", t + ".srm")), root));
                BothAlike($"hash {i}", text, (root, t) => RomHash.Md5(Rom(root, t)));
                BothAlike($"cheats to a path {i}", text, (root, t) => CheatFile.SaveTo(Path.Combine(root, t, "list.json"), List(t)) + " " + Answered(() => CheatFile.LoadFrom(Path.Combine(root, t, "list.json")), root));
                BothAlike($"config file {i}", text, (root, t) =>
                {
                    var file = new ConfigFile<CheatFile>(Path.Combine("cheats", t), t + ".json");
                    return $"{file.Save(List(t))} {Answered(() => file.Load(), root)} {file.Delete()}";
                });
                BothAlike($"a game's list {i}", text, (root, t) =>
                {
                    GameCheats game = GameCheats.For("SNES", Rom(root, t))!;
                    return $"{game.Name} {game.SaveIfChanged(List(t))} {Answered(() => game.Load(), root)} {game.SaveIfChanged(List(t))}";
                });

                string Legacy(string root, string t)
                {
                    string legacy = Path.Combine(root, "legacy " + t);
                    Put(Path.Combine(legacy, "Saves", t + ".srm"), "save");
                    Put(Path.Combine(legacy, "Games", "SNES", t + ".sfc"), "rom");
                    return legacy;
                }
                BothAlike($"migration {i}", text, (root, t) => DataMigration.Run(Legacy(root, t), Path.Combine(root, "home " + t)));
                BothAlike($"seed {i}", text, (root, t) => DataMigration.SeedFromBundle(Legacy(root, t), Path.Combine(root, "home")));
                BothAlike($"tree {i}", text, (root, t) => DataMigration.CopyTree(Legacy(root, t), Path.Combine(root, "copy")));
                BothAlike($"remaining {i}", text, (root, t) => DataMigration.RemainingLibraryDirectories(Legacy(root, t)));

                DateTime day = new(2026, 10, 10, 12, 0, 0);
                BothAlike($"log {i}", text, (root, t) =>
                {
                    ErrorLog.DirectoryOverride = Path.Combine(root, "logs " + t);
                    ErrorLog.ResetForTests();
                    return $"{ErrorLog.Root()} {ErrorLog.PathFor(day)} {Logged(ErrorLog.Error("test", "m " + t))}";
                });
                ErrorLog.DirectoryOverride = _logOverride;
            }
        }

        // The file lands with the bytes .NET's own file calls give such a name, as measured in §21.2.
        [Fact]
        public void A_file_named_with_half_a_pair_lands_with_the_bytes_dotnet_writes()
        {
            string folder = At("bytes");
            Assert.True(AtomicFile.Write(Path.Combine(folder, "a\uD800b.srm"), new byte[] { 7 }));
            Assert.Equal(new[] { Path.Combine(folder, "a�b.srm") }, Directory.GetFiles(folder));
            Assert.Equal(new byte[] { 7 }, AtomicFile.TryRead(Path.Combine(folder, "a\uDC00b.srm")));
        }

        // The words of a failed write come back as a list, each item ended by a NUL, so a path holding one is refused before the call, said as one message.
        [Fact]
        public void A_path_holding_a_nul_is_refused_in_one_message_and_read_as_no_file()
        {
            string path = At("nul", "a\0b.srm");
            Assert.False(AtomicFile.Write(path, new byte[] { 1 }));
            Assert.Equal(new[] { $"{path}: a file name cannot hold a NUL; the previous file is unchanged" }, Reported());
            Assert.False(Directory.Exists(At("nul")));
            Assert.Null(AtomicFile.TryRead(path));
            Assert.Empty(Reported());
            Assert.Throws<ArgumentException>(() => MediumLock.TryTake(path, "Game"));
        }
    }
}
