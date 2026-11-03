using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Pages
{
    // What a game's page shows of one game, read fresh each time it changes; pictures are files, never loaded here - see EmuSen_BigPicture.md §50.
    public sealed record GameFacts(string Path, string Title, string Console, string? Mark, string? Year, string? Developer, string? Publisher,
        TimeSpan Played, DateTime? LastPlayed, string? Description, bool Favourite, double CoverAspect);

    // The game's pictures by kind: the cover, the hero's best art, a screenshot for the overview, a clip for the poster slot.
    public sealed record GameArt(string? Cover, string? Hero, string? Screenshot, string? Clip);

    // One saved state: Slot is its place ("Slot 3", "Auto"), its picture and when it was saved.
    public sealed record GameState(string Path, string Slot, string? Picture, DateTime When);

    public enum GameMediaKind { Taken, Screenshot, TitleScreen, Marquee, Clip, Fanart }

    // One picture or clip of the game: one the player took, or one the scraper fetched.
    public sealed record GameMedia(string Path, GameMediaKind Kind, DateTime When);

    // One value of a stepped option: what is stored, and its words.
    public sealed record GameChoice(string? Value, string Text);

    // One cheat of the game, live in the running game or in its saved list.
    public sealed record GameCheat(int Id, string Description, bool Enabled);

    // A cover the game can show: its own, or another game's borrowed.
    public sealed record GameCover(string Key, string Label, string? Picture, bool InUse);

    // The game's package as its controls show it, read from its manifest.
    public sealed record GamePackageFacts(bool IsPackage, string Mode, IReadOnlyList<(string Name, string Format)> Layers, IReadOnlyList<string> Beside,
        bool CanMake, bool CanChange, bool Patched, bool TakesPatches, string? Status);

    // One game ScreenScraper found by name.
    public sealed record GameMatch(string Key, string Label);

    // The answer to a name search: the matches, or why there are none.
    public sealed record GameSearch(IReadOnlyList<GameMatch> Matches, string? Why);

    // One field of the game's metadata as the editor rows show it.
    public sealed record GameField(string Key, string Label, MetadataKind Kind, string? Value, IReadOnlyList<GameChoice>? Choices);

    // Everything the page reads of the window it lives in and asks it to do; MainWindow is the one host, a test another.
    public interface IGamePageHost
    {
        DateTime Now { get; }
        bool ScrapeRunning { get; }
        int StateSlots { get; }

        GameFacts Facts(string path);
        GameArt Art(string path);
        IReadOnlyList<GameState> States(string path);
        IReadOnlyList<GameMedia> Media(string path);
        (int Width, int Height)? PictureSize(string path);

        IReadOnlyList<GameChoice> Engines(string path);
        string? Engine(string path);
        void SetEngine(string path, string? engine);

        // The game's controller profile, kept in Endymion's store; null where its console has no profile but Default, so the row is left out (EmuSen_Input.md §10.5).
        IReadOnlyList<GameChoice>? Profiles(string path) => null;
        string? Profile(string path) => null;
        void SetProfile(string path, string? profile) { }
        IReadOnlyList<GameChoice>? Standards(string path);
        string? Standard(string path);
        void SetStandard(string path, string? standard);
        string? Crop(string path);
        void OpenCrop(string path, string title);

        // Null where the game has no cheats and is not running, so the row is left out.
        IReadOnlyList<GameCheat>? Cheats(string path);
        void SetCheat(string path, int id, bool on);
        bool CheatsTakeCodes(string path);
        string AddCode(string path, string code);

        GamePackageFacts Package(string path);
        void PackageMake(string path);
        void PackageAdd(string path, string file);
        void PackageRemove(string path, int layer);
        void PackageIdentity(string path, bool patched);
        void PackageMigrate(string path);

        IReadOnlyList<GameCover> Covers(string path);
        void ChooseCover(string path, string key);

        IReadOnlyList<GameField> Fields(string path);
        void EditField(string path, string key, string? value);
        void ResetField(string path, string key);
        void ClearMetadata(string path);

        string? WhyNoSearch { get; }
        Task<GameSearch> SearchAsync(string path, string name);
        string SearchName(string path);
        bool ChooseMatch(string path, string key);
        Task<bool> ScrapeByFileAsync(string path);

        void Play(string path, string? state);
        void ToggleFavourite(string path);
        void OpenFolder(string path);
        void Hide(string path);
        bool DeleteState(string path, string state);
        bool DeleteMedia(string path, string media);
        Task<bool> ConfirmAsync(string title, string message, string yes);
        Task<string?> TypeAsync(string title, string? text);
        void Close();
    }
}
