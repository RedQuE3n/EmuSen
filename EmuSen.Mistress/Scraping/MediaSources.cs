using System.Collections.Generic;
using System.Linq;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.Scraping
{
    public enum MediaSource { None, HandPlaced, ScreenScraper, EsdeFolder, OpenEmu, OtherGame, Package }

    // Where a game's picture is looked for, in emusen-scrape's order: another game's cover the player chose, what the player placed, a packaged game's package, ScreenScraper's, an ES-DE folder, OpenEmu's failover - see EmuSen_Settings_Reference.md §4.60, §4.65, EmuSen_GamePackages.md §18.3 and EmuSen_Backend.md §40.5.
    public sealed class MediaSources : ISceneMedia
    {
        private readonly NativeScraping _native;
        private readonly string _choiceStamp;

        // The choice stamp is the window's, which changes when any cover choice does.
        public MediaSources(NativeScraping native, string choiceStamp)
        {
            _native = native;
            _choiceStamp = choiceStamp;
        }

        public string? Find(ThemeSystem system, SceneGame game, string mediaType) =>
            _native.Locate(system.Name, game.File, mediaType, game.FolderPath, game is { Folder: true, IsCollection: false }).Path;

        // The folders' own listings for the variant triggers (§16), a cover from the player's folder or OpenEmu's counting too.
        public IReadOnlySet<string> Present(ThemeSystem system, IReadOnlyList<SceneGame> games) =>
            _native.Present(system.Name, games.Select(g => (g.File, g.FolderPath, g is { Folder: true, IsCollection: false })));

        // Changes when a file is added to or taken from any folder a picture could come from; the player's folder is the caller's to key.
        public string? Stamp(ThemeSystem system) => _native.Stamp(system.Name, _choiceStamp);

        // The library's cover for a ROM on the shelf ES-DE calls esdeSystem.
        public string? Cover(string esdeSystem, string romPath) => Locate(esdeSystem, romPath, ScrapeRules.Cover.EsdeType).Path;

        public (MediaSource Source, string? Path) Locate(string esdeSystem, string romPath, string mediaType) => _native.Locate(esdeSystem, romPath, mediaType);
    }
}
