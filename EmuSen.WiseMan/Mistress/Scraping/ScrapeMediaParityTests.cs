using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Client.Scrape;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // A picture's sources and a game's metadata in Rust against their C# twins: generated stores, art folders, ES-DE folders, OpenEmu covers, packaged games' packages and covers chosen from other games, every game asked every type through both, each shelf's present types, and every field resolved - see EmuSen_Backend.md §30.6.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class ScrapeMediaParityTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenScrapeMediaParity", Guid.NewGuid().ToString("N"));

        public ScrapeMediaParityTests()
        {
            Assert.True(MediaService.Ready, MediaService.Report);
            RuntimeCatalog.Forget();
        }

        public void Dispose()
        {
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        private static void Put(string path, string text = "x")
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllText(path, text);
        }

        private static readonly string[] Types = ["cover", "screenshot", "marquee", "titlescreen", "miximage", "backcover", "3dbox", "physicalmedia", "fanart", "video", "manual"];

        [Fact]
        public void Every_game_s_pictures_present_types_and_metadata_resolve_alike()
        {
            int fromPackages = 0;
            for (int seed = 1; seed <= 3; seed++)
            {
                var r = new Random(seed);
                string root = Path.Combine(_root, $"s{seed}"), roms = Path.Combine(root, "Roms"), store = Path.Combine(root, "Media"), art = Path.Combine(root, "Art"), esde = Path.Combine(root, "Esde");
                string[] folders = ["", "", "RPG", "RPG/Old"];
                var games = new List<(string Rom, string System)>();
                var packages = new Dictionary<string, string>(StringComparer.Ordinal);
                for (int i = 0; i < 18; i++)
                {
                    string system = new[] { "snes", "nes", "gb", "gbc" }[r.Next(4)];
                    string ext = system switch { "snes" => ".sfc", "nes" => ".nes", "gb" => ".gb", _ => ".gbc" };
                    string folder = folders[r.Next(folders.Length)];
                    string stem = $"Game {i} ({new[] { "USA", "Europe", "Japan" }[r.Next(3)]})";
                    string rom = Path.Combine(roms, system, folder, stem + ext);
                    Directory.CreateDirectory(Path.GetDirectoryName(rom)!);
                    File.WriteAllBytes(rom, system is "gb" or "gbc" ? new byte[0x150] : system == "nes" ? SyntheticNesRom.Build() : SyntheticRom.BuildBlank());
                    games.Add((rom, system));
                    string named = folder.Length == 0 ? stem : folder + "/" + stem;
                    foreach (string type in Types.Where(_ => r.Next(3) == 0))
                    {
                        string kindFolder = ScrapeRules.KindOf(type)!.Folder, extn = type == "video" ? ".mp4" : type == "manual" ? ".pdf" : new[] { ".png", ".jpg", ".webp" }[r.Next(3)];
                        int where = r.Next(3);
                        if (where != 1) Put(Path.Combine(store, system, kindFolder, (r.Next(2) == 0 ? named : stem) + extn));
                        if (where != 0) Put(Path.Combine(esde, system, kindFolder, (r.Next(2) == 0 ? named : stem) + extn));
                    }
                    if (r.Next(4) == 0) Put(Path.Combine(art, new[] { "SNES", "Nintendo - Game Boy", "", "GB" }[r.Next(4)], ArtworkIndex.Key(r.Next(2) == 0 ? stem : ArtworkIndex.Untagged(stem)) + ".png"));
                    if (r.Next(4) == 0) Put(Path.Combine(store, "openemu", new[] { "SNES", "NES", "GB" }[r.Next(3)], CoverFetcher.Safe(stem) + ".png"));
                    // A packaged game: its package named otherwise than the file, some of its art in the package and some only in the store (§18.3).
                    if (r.Next(3) == 0)
                    {
                        string package = Path.Combine(root, "Packages", system, $"Package {i}{ext}");
                        packages[rom] = package;
                        foreach (string type in Types.Where(_ => r.Next(2) == 0))
                            Put(EmuSen.Galaxia.Media.GamePackage.MediaPathFor(package, ScrapeRules.KindOf(type)!.Folder, type == "video" ? ".mp4" : type == "manual" ? ".pdf" : new[] { ".png", ".jpg", ".webp" }[r.Next(3)]));
                    }
                }

                string db = Path.Combine(root, "games.db");
                using GameRecords records = GameRecords.OpenNative(db);
                foreach (var (rom, _) in games.Where(_ => r.Next(3) == 0))
                {
                    string from = games[r.Next(games.Count)].Rom;
                    records.SaveEdits(rom, new Dictionary<string, string?> { [GameMetadata.CoverFrom] = from, [GameMetadata.Developer] = r.Next(2) == 0 ? "Edited" : null, [GameMetadata.Hidden] = "1" }, new DateTime(2026, 10, 1));
                }
                using (MediaStore scraped = MediaStore.Open(store))
                    foreach (var (rom, _) in games.Where(_ => r.Next(2) == 0))
                    {
                        string md5 = Path.GetFileName(rom).GetHashCode().ToString("x8");
                        scraped.RememberFile(rom, 1, 1, md5);
                        scraped.Record(new ScrapedRecord(md5, 1, ScrapeState.Found)
                        {
                            Name = "Scraped " + Path.GetFileNameWithoutExtension(rom), Description = r.Next(2) == 0 ? "Words." : null, Rating = r.Next(2) == 0 ? ScrapeRules.Rating(r.Next(21)) : (float)r.NextDouble(),
                            ReleaseDate = new DateTime(1990 + r.Next(10), 1 + r.Next(12), 1 + r.Next(28)), Genre = "Racing", Players = "1-2", FetchedAt = new DateTime(2026, 9, 1, 0, 0, 0, DateTimeKind.Utc),
                        });
                    }

                ArtworkIndex index = ArtworkIndex.Scan(art, RuntimeCatalog.Consoles);
                var edits = records.AllEdits();
                string? CoverFrom(string p) => edits.TryGetValue(p, out var e) && e.TryGetValue(GameMetadata.CoverFrom, out string? f) && f.Length > 0 ? f : null;
                var csharp = new MediaSources((console, title) => index.Find(console, title), store, esde, Path.Combine(store, "openemu"), CoverFrom, "", roms, packages);

                string settings = JsonSerializer.Serialize(new Dictionary<string, object?>
                {
                    ["roms"] = roms, ["art"] = art, ["esde_media"] = esde, ["openemu"] = true, ["developer_files"] = false,
                    ["cores"] = RuntimeCatalog.Consoles.Select(c => c.Systems).ToArray(), ["packages"] = packages,
                });
                using var library = EmuSen.Crystal.Client.Library.GameLibrary.Open(db);
                using MediaService rust = MediaService.Open(store, settings, library);
                using MediaStore read = MediaStore.Open(store);
                foreach (var (rom, system) in games)
                {
                    string esdeSystem = RuntimeCatalog.ShelfByName(RuntimeCatalog.ShelfFor(rom))!.EsdeSystem;
                    foreach (string type in Types)
                    {
                        var expected = csharp.Locate(esdeSystem, rom, type);
                        if (expected.Source == MediaSource.Package) fromPackages++;
                        using JsonDocument got = JsonDocument.Parse(rust.Get(JsonSerializer.Serialize(new { what = "media", path = rom, system = esdeSystem, type })));
                        string actual = got.RootElement.ValueKind == JsonValueKind.Null ? "None " : $"{got.RootElement.GetProperty("source").GetString()} {got.RootElement.GetProperty("path").GetString()}";
                        string want = expected.Path is null ? "None " : $"{expected.Source switch { MediaSource.OtherGame => "other_game", MediaSource.HandPlaced => "hand", MediaSource.ScreenScraper => "screenscraper", MediaSource.EsdeFolder => "esde", MediaSource.Package => "package", _ => "openemu" }} {expected.Path}";
                        Assert.True(want == actual, $"seed {seed}, {rom}, {type}: C# {want}, Rust {actual}");
                    }
                    var metadata = GameMetadata.Resolve(rom, read.FoundFor(rom), records.Edits(rom), scrapedNames: seed % 2 == 0);
                    using JsonDocument m = JsonDocument.Parse(rust.Get(JsonSerializer.Serialize(new { what = "metadata", path = rom, scraped_names = seed % 2 == 0 })));
                    foreach (MetadataField field in GameMetadata.Fields)
                    {
                        JsonElement value = m.RootElement.GetProperty(field.Key);
                        string? text = value.GetProperty("value").ValueKind == JsonValueKind.Null ? null : value.GetProperty("value").GetString();
                        Assert.Equal((metadata[field.Key].Value, metadata[field.Key].Source.ToString().ToLowerInvariant()), (text, value.GetProperty("source").GetString()));
                    }
                }
                foreach (var shelf in games.GroupBy(g => RuntimeCatalog.ShelfByName(RuntimeCatalog.ShelfFor(g.Rom))!.EsdeSystem))
                {
                    var scene = shelf.Select(g => new SceneGame(Path.GetFileNameWithoutExtension(g.Rom), g.Rom) { FolderPath = GameFolders.Of(roms, g.Rom) }).ToList();
                    var expected = csharp.Present(new ThemeSystem(shelf.Key, shelf.Key, shelf.Key), scene).Order(StringComparer.Ordinal);
                    var actual = JsonSerializer.Deserialize<string[]>(rust.Get(JsonSerializer.Serialize(new { what = "present", system = shelf.Key, games = scene.Select(s => new { path = s.File, folder_path = s.FolderPath }) })))!.Order(StringComparer.Ordinal);
                    Assert.Equal(expected, actual);
                }
                string PackagePart(string stamp) => stamp[(stamp.LastIndexOf('|') + 1)..];
                Assert.Equal(PackagePart(csharp.Stamp(new ThemeSystem("snes", "snes", "snes"))!), PackagePart(JsonSerializer.Deserialize<string>(rust.Get(JsonSerializer.Serialize(new { what = "stamp", system = "snes", choice = "" })))!));
            }
            Assert.True(fromPackages > 0, "no picture came from a package");
        }
    }
}
