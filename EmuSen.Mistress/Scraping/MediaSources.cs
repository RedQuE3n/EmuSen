using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.Scraping
{
    public enum MediaSource { None, HandPlaced, ScreenScraper, EsdeFolder, OpenEmu }

    // Where a game's picture is looked for, in order: what the player placed, ScreenScraper's, an ES-DE folder, OpenEmu's failover - see EmuSen_Settings_Reference.md §4.60.
    public sealed class MediaSources : ISceneMedia
    {
        private static readonly string[] Extensions = [".png", ".jpg", ".jpeg"];

        private readonly Func<string, string, string?> _handPlaced;
        private readonly EsdeMediaFolder _scraped;
        private readonly EsdeMediaFolder? _esde;
        private readonly string? _openEmu;

        // handPlaced takes a console and a ROM's title; openEmuRoot is null when the failover is not ticked, so what it fetched is not shown either.
        public MediaSources(Func<string, string, string?> handPlaced, string scrapedRoot, string? esdeRoot, string? openEmuRoot)
        {
            _handPlaced = handPlaced;
            _scraped = new EsdeMediaFolder(scrapedRoot);
            _esde = string.IsNullOrWhiteSpace(esdeRoot) ? null : new EsdeMediaFolder(esdeRoot);
            _openEmu = string.IsNullOrWhiteSpace(openEmuRoot) ? null : openEmuRoot;
        }

        public string? Find(ThemeSystem system, SceneGame game, string mediaType) => Locate(system.Name, game.File, mediaType).Path;

        // The folders' own listings for the variant triggers (§16), and a cover from the player's folder or OpenEmu's counts too.
        public IReadOnlySet<string> Present(ThemeSystem system, IReadOnlyList<SceneGame> games)
        {
            var found = new HashSet<string>(_scraped.Present(system, games), StringComparer.Ordinal);
            if (_esde is not null) found.UnionWith(_esde.Present(system, games));
            if (!found.Contains(ScrapeRules.Cover.EsdeType) && games.Any(g => Cover(system.Name, g.File) is not null)) found.Add(ScrapeRules.Cover.EsdeType);
            return found;
        }

        // Changes when a file is added to or taken from any folder a picture could come from; the player's folder is the caller's to key.
        public string? Stamp(ThemeSystem system)
        {
            string openEmu = _openEmu is not null && Directory.Exists(_openEmu)
                ? string.Join(",", Directory.EnumerateDirectories(_openEmu).Select(d => Directory.GetLastWriteTimeUtc(d).Ticks))
                : "";
            return $"{_scraped.Stamp(system)}|{_esde?.Stamp(system)}|{openEmu}";
        }

        // The library's cover for a ROM on the shelf ES-DE calls esdeSystem.
        public string? Cover(string esdeSystem, string romPath) => Locate(esdeSystem, romPath, ScrapeRules.Cover.EsdeType).Path;

        public (MediaSource Source, string? Path) Locate(string esdeSystem, string romPath, string mediaType)
        {
            bool cover = mediaType == ScrapeRules.Cover.EsdeType;
            string title = Path.GetFileNameWithoutExtension(romPath);
            string? console = EmuSen.Cores.CoreCatalog.ByExtension(Path.GetExtension(romPath))?.Console;
            var system = new ThemeSystem(esdeSystem, esdeSystem, esdeSystem);
            var game = new SceneGame(title, romPath);

            if (cover && console is not null && _handPlaced(console, title) is string hand) return (MediaSource.HandPlaced, hand);
            if (_scraped.Find(system, game, mediaType) is string scraped) return (MediaSource.ScreenScraper, scraped);
            if (_esde?.Find(system, game, mediaType) is string esde) return (MediaSource.EsdeFolder, esde);
            if (cover && console is not null && _openEmu is not null && OpenEmuCover(_openEmu, console, title) is string openEmu) return (MediaSource.OpenEmu, openEmu);
            return (MediaSource.None, null);
        }

        // Where CoverFetcher writes: <root>/<console>/<libretro-safe stem>.<ext>.
        public static string? OpenEmuCover(string root, string console, string title)
        {
            string stem = Library.CoverFetcher.Safe(title);
            foreach (string ext in Extensions)
            {
                string path = Path.Combine(root, console, stem + ext);
                if (File.Exists(path)) return path;
            }
            return null;
        }
    }
}
