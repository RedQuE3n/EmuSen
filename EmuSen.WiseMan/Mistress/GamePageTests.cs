using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;
using Xunit;

namespace EmuSen.WiseMan.Mistress
{
    // The game's page as a plain class, without a window: its rows, tiles, hints and what each press asks of the host - see EmuSen_BigPicture.md §50.
    public class GamePageTests
    {
        private static readonly DateTime Now = new(2026, 11, 1, 21, 42, 0);

        // A host that keeps its game in memory and writes down what the page asks.
        internal sealed class FakeHost : IGamePageHost
        {
            public readonly List<string> Asked = new();
            public bool Favourite, Running, Answer = true;
            public string? EngineValue, StandardValue;
            public string? Typed = "typed";
            public List<GameState> StateFiles = new()
            {
                new("/s/slot1", "Slot 1", "/p/1.png", GamePageTests.Now.AddHours(-2)),
                new("/s/slot3", "Slot 3", null, GamePageTests.Now.AddDays(-3)),
                new("/s/resume", GamePage.AutoSlot, "/p/r.png", GamePageTests.Now.AddMinutes(-12)),
            };
            public List<GameCheat> CheatList = new() { new(0, "Infinite lives", true), new(1, "All doors open", false), new(2, "Start with the choir", false) };
            public Dictionary<string, string?> Fields = new() { [GameMetadata.Name] = "Granite Choir", [GameMetadata.Developer] = "Studio A", [GameMetadata.Hidden] = null };
            public GameSearch Search = new([new("1", "Granite Choir (1994, Super Nintendo)"), new("2", "Granite Choir II (1996, Super Nintendo)")], null);

            public DateTime Now => GamePageTests.Now;
            public bool ScrapeRunning { get; set; }
            public int StateSlots => 4;
            public GameFacts Facts(string path) => new(path, "Granite Choir", "Super Nintendo", "SNES", "1994", "Studio A", "Publisher P", TimeSpan.FromMinutes(860), Now.AddDays(-1), null, Favourite, 4 / 3.0);
            public GameArt Art(string path) => new("/a/cover.png", "/a/fanart.png", "/a/shot.png", "/a/clip.mp4");
            public IReadOnlyList<GameState> States(string path) => StateFiles.OrderByDescending(s => s.When).ToList();
            public IReadOnlyList<GameMedia> Media(string path) => [new("/m/1.png", GameMediaKind.Taken, Now), new("/m/2.png", GameMediaKind.Taken, Now), new("/a/shot.png", GameMediaKind.Screenshot, Now), new("/a/clip.mp4", GameMediaKind.Clip, Now)];
            public (int Width, int Height)? PictureSize(string path) => (512, 448);
            public IReadOnlyList<GameChoice> Engines(string path) => [new(null, "VenusRT (default)"), new("VenusRT", "VenusRT (the console's)"), new("Snes9x", "Snes9x")];
            public string? Engine(string path) => EngineValue;
            public void SetEngine(string path, string? engine) { Asked.Add($"engine {engine}"); EngineValue = engine; }
            public IReadOnlyList<GameChoice>? Standards(string path) => [new(null, "The game's"), new("ntsc", "NTSC"), new("pal", "PAL")];
            public string? Standard(string path) => StandardValue;
            public void SetStandard(string path, string? standard) { Asked.Add($"standard {standard}"); StandardValue = standard; }
            public string? Crop(string path) => "Console's";
            public void OpenCrop(string path, string title) => Asked.Add("crop");
            public IReadOnlyList<GameCheat>? Cheats(string path) => CheatList;
            public void SetCheat(string path, int id, bool on) { Asked.Add($"cheat {id} {on}"); CheatList[id] = CheatList[id] with { Enabled = on }; }
            public bool CheatsTakeCodes(string path) => Running;
            public string AddCode(string path, string code) { Asked.Add("code " + code); return "Added."; }
            public GamePackageFacts Package(string path) => new(true, "Linked · 1 layer", [("Widescreen patch.ips", "IPS")], ["/r/retranslation.ips"], false, true, false, true, null);
            public void PackageMake(string path) => Asked.Add("make");
            public void PackageAdd(string path, string file) => Asked.Add("add " + file);
            public void PackageRemove(string path, int layer) => Asked.Add($"remove {layer}");
            public void PackageIdentity(string path, bool patched) => Asked.Add($"identity {patched}");
            public void PackageMigrate(string path) => Asked.Add("migrate");
            public IReadOnlyList<GameCover> Covers(string path) => [new("own", "Its own", "/a/cover.png", true), new("/r/aurora.sfc", "Borrow from Aurora Drift", "/a/aurora.png", false)];
            public void ChooseCover(string path, string key) => Asked.Add("cover " + key);
            IReadOnlyList<GameField> IGamePageHost.Fields(string path) => GameMetadata.Fields.Where(f => Fields.ContainsKey(f.Key) || f.Kind == MetadataKind.Flag)
                .Select(f => new GameField(f.Key, f.Label, f.Kind, Fields.GetValueOrDefault(f.Key), f.Kind == MetadataKind.Flag ? [new GameChoice("0", "Off"), new GameChoice("1", "On")] : null)).ToList();
            public void EditField(string path, string key, string? value) { Asked.Add($"edit {key}={value}"); Fields[key] = value; }
            public void ResetField(string path, string key) => Asked.Add("reset " + key);
            public void ClearMetadata(string path) => Asked.Add("clear");
            public string? WhyNoSearch { get; set; }
            public Task<GameSearch> SearchAsync(string path, string name) { Asked.Add("search " + name); return Task.FromResult(Search); }
            public string SearchName(string path) => "Granite Choir";
            public bool ChooseMatch(string path, string key) { Asked.Add("match " + key); return true; }
            public Task<bool> ScrapeByFileAsync(string path) { Asked.Add("by file"); return Task.FromResult(true); }
            public void Play(string path, string? state) => Asked.Add($"play {state}");
            public void ToggleFavourite(string path) { Asked.Add("favourite"); Favourite = !Favourite; }
            public void OpenFolder(string path) => Asked.Add("folder");
            public void Hide(string path) => Asked.Add("hide");
            public bool DeleteState(string path, string state) { Asked.Add("delete " + state); StateFiles.RemoveAll(s => s.Path == state); return true; }
            public bool DeleteMedia(string path, string media) { Asked.Add("delete " + media); return true; }
            public Task<bool> ConfirmAsync(string title, string message, string yes) { Asked.Add("ask " + title); return Task.FromResult(Answer); }
            public Task<string?> TypeAsync(string title, string? text) { Asked.Add("type " + title); return Task.FromResult(Typed); }
            public void Close() => Asked.Add("close");
        }

        private static (GamePage Page, FakeHost Host) Page(bool desktop = false)
        {
            var host = new FakeHost();
            return (new GamePage(host, "/r/Granite Choir.sfc", desktop), host);
        }

        private static PageContent.Rows Rows(GamePage p) => ((GameContent.Rows)p.Content).Value;

        [Fact]
        public void The_hero_s_facts_and_actions_are_the_designs()
        {
            var (page, _) = Page();
            Assert.Equal("Super Nintendo · 1994 · Studio A · Publisher P · 14 h 20 min · Last played yesterday", page.FactsLine);
            Assert.Equal(["Play", "Resume", "Favourite"], page.Actions.Select(a => a.Text));
            Assert.Equal(PageMark.Heart, page.Actions[2].Mark);
            Assert.Equal("Auto · 12 min ago", page.Actions[1].Value);
            Assert.Equal(["Overview", "States", "Options", "Media", "Details"], page.SectionList().Select(s => s.Label));
            Assert.Equal("3", page.SectionList()[1].Status);
            Assert.Equal(GamePage.NoDescription, ((GameContent.Text)page.Content).Words);
        }

        [Fact]
        public void Play_resume_with_its_chosen_slot_and_favourite_ask_the_host()
        {
            var (page, host) = Page();
            page.Act(GamePage.PlayKey);
            page.StepAction(GamePage.ResumeKey, 1);
            Assert.Equal("Slot 1 · 2 h ago", page.Actions[1].Value);
            page.Act(GamePage.ResumeKey);
            page.Act(GamePage.FavouriteKey);
            Assert.Equal(["play ", "play /s/slot1", "favourite"], host.Asked);
            Assert.Equal(PageMark.FilledHeart, page.Actions[2].Mark);
        }

        [Fact]
        public void States_are_the_slots_with_auto_after_the_last_used_and_the_empty_ones_after()
        {
            var (page, host) = Page();
            page.ChooseSection(GamePage.States);
            var tiles = ((GameContent.Tiles)page.Content).Items;
            Assert.Equal(["Slot 1", "Slot 2", "Slot 3", "Auto", "Slot 4"], tiles.Select(t => t.Label));
            Assert.Equal("2 h ago", tiles[0].Status);
            Assert.True(tiles[1].IsEmpty);
            Assert.True(tiles[3].IsMarked);
            Assert.Equal(200, ((GameContent.Tiles)page.Content).Width);
            Assert.Contains(page.Hints(new PageFocus(false, "/s/slot3")), h => h.Button == PadGlyphButton.West && h.Key == GamePage.DeleteHint);
            Assert.Equal("Load", page.Hints(new PageFocus(false, "/s/slot3"))[0].Text);
            page.Hint(GamePage.DeleteHint);
            Assert.Contains("delete /s/slot3", host.Asked);
            page.ChooseTile("/s/slot1");
            Assert.Equal("play /s/slot1", host.Asked[^1]);
        }

        [Fact]
        public void Options_step_the_engine_and_the_standard_and_open_cheats_crop_and_package()
        {
            var (page, host) = Page();
            page.ChooseSection(GamePage.Options);
            PageContent.Rows rows = Rows(page);
            Assert.Equal(["Engine", "Cheats", "Display crop", "Television standard", "Package"], rows.Items.Select(r => r.Label));
            Assert.Equal("VenusRT (default)", rows.Items[0].Status);
            Assert.Equal("1 of 3 on", rows.Items[1].Status);
            Assert.Equal("Value", page.Hints(new PageFocus(false, "engine")).Single(h => h.Button == PadGlyphButton.DPadLeftRight).Text);
            page.StepRow("engine", 1);
            page.StepRow("standard", -1);
            Assert.Equal(["engine VenusRT", "standard pal"], host.Asked);
            page.ChooseRow("crop");
            Assert.Equal("crop", host.Asked[^1]);
            page.ChooseRow("cheats");
            Assert.Equal(("Options", GamePage.CheatsPage), page.Trail);
            page.StepRow("1", 1);
            Assert.Equal("cheat 1 True", host.Asked[^1]);
            Assert.DoesNotContain(Rows(page).Items, r => r.Key == "add");
            Assert.Equal("Options", page.Hints(new PageFocus(false, "0")).Single(h => h.Button == PadGlyphButton.East).Text);
            Assert.True(page.Back());
            Assert.Null(page.Trail);
            Assert.Equal(GamePage.CheatsPage, page.ReturnedFrom);
            Assert.DoesNotContain("close", host.Asked);
        }

        [Fact]
        public void The_package_page_lists_its_layers_and_asks_before_each_change()
        {
            var (page, host) = Page();
            page.ChooseSection(GamePage.Options);
            page.ChooseRow("package");
            Assert.Equal(["Mode", "Layer: Widescreen patch.ips", "Add retranslation.ips", "Scraped as", "Copy Saves and Art In"], Rows(page).Items.Select(r => r.Label));
            page.ChooseRow("layer.1");
            page.StepRow("identity", 1);
            Assert.Equal(["ask Take out Widescreen patch.ips?", "remove 1", "identity True"], host.Asked);
        }

        [Fact]
        public void Details_scrape_asks_by_name_and_a_match_or_the_file_scrapes_it()
        {
            var (page, host) = Page();
            page.ChooseSection(GamePage.Details);
            Assert.Equal(["Scrape This Game", "Edit Details", "Cover art", "Open Folder", "Hide from Library"], Rows(page).Items.Select(r => r.Label));
            Assert.Equal("Its own · 2 to choose from", Rows(page).Items[2].Status);
            page.ChooseRow("scrape");
            Assert.Equal(["Granite Choir (1994, Super Nintendo)", "Granite Choir II (1996, Super Nintendo)", "Search by another name…", "By the file's checksums…", "No"], Rows(page).Items.Select(r => r.Label));
            Assert.Equal("No", page.Hints(new PageFocus(false, "no")).Single(h => h.Button == PadGlyphButton.East).Text);
            page.ChooseRow("match.2");
            Assert.Equal(["search Granite Choir", "match 2"], host.Asked);
            Assert.Null(page.Trail);
            page.ChooseRow("scrape");
            page.ChooseRow("file");
            Assert.Equal("by file", host.Asked[^1]);
        }

        [Fact]
        public void Without_screen_scraper_the_question_says_why_and_sends_nothing()
        {
            var (page, host) = Page();
            host.WhyNoSearch = "there is no developer file";
            page.ChooseSection(GamePage.Details);
            page.ChooseRow("scrape");
            Assert.Equal("ScreenScraper cannot be asked: there is no developer file.", Rows(page).Sentence);
            Assert.DoesNotContain(host.Asked, a => a.StartsWith("search", StringComparison.Ordinal));
            Assert.DoesNotContain(Rows(page).Items, r => r.Key == "other");
        }

        [Fact]
        public void Edit_details_types_and_steps_fields_saved_at_once_and_resets_on_y()
        {
            var (page, host) = Page();
            page.ChooseSection(GamePage.Details);
            page.ChooseRow("edit");
            Assert.Equal(("Details", GamePage.EditPage), page.Trail);
            Assert.Contains(Rows(page).Items, r => r.Key == GameMetadata.Name && r.Kind == PageRowKind.Text && r.Status == "Granite Choir");
            Assert.DoesNotContain(Rows(page).Items, r => r.Key == GameMetadata.AltEmulator);
            page.ChooseRow(GameMetadata.Developer);
            Assert.Equal("edit developer=typed", host.Asked[^1]);
            page.StepRow(GameMetadata.Hidden, 1);
            Assert.Equal("edit hidden=1", host.Asked[^1]);
            Assert.Equal("Reset to scraped", page.Hints(new PageFocus(false, GameMetadata.Name)).Single(h => h.Button == PadGlyphButton.North).Text);
            page.Hint(GamePage.ResetHint);
            Assert.Equal("reset name", host.Asked[^1]);
            page.ChooseRow(GamePage.ClearRow);
            Assert.Equal("clear", host.Asked[^1]);
        }

        [Fact]
        public void Cover_art_and_media_the_viewer_steps_and_deletes_only_a_taken_screenshot()
        {
            var (page, host) = Page();
            page.ChooseSection(GamePage.Details);
            page.ChooseRow("cover");
            Assert.Equal(150, ((GameContent.Tiles)page.Content).Width);
            page.ChooseTile("/r/aurora.sfc");
            Assert.Equal("cover /r/aurora.sfc", host.Asked[^1]);
            page.Back();
            page.ChooseSection(GamePage.Media);
            Assert.Equal(["Screenshot 1", "Screenshot 2", "Scraped screenshot", "Clip"], ((GameContent.Tiles)page.Content).Items.Select(t => t.Label));
            page.ChooseTile("/m/2.png");
            Assert.StartsWith("Screenshot 2 (2 of 4) · 1 November 2026 · 512 × 448", page.Viewer!.Value.Caption);
            Assert.Contains(page.Hints(default), h => h.Key == GamePage.DeleteHint);
            page.StepViewer(1);
            Assert.DoesNotContain(page.Hints(default), h => h.Key == GamePage.DeleteHint);
            page.StepViewer(1);
            Assert.Equal("/a/fanart.png", page.Viewer!.Value.Picture);
            Assert.True(page.Back());
            Assert.Null(page.Viewer);
        }

        [Fact]
        public void B_at_the_top_and_start_close_the_page_and_the_desktop_shows_details_as_a_form()
        {
            var (page, host) = Page();
            page.Back();
            page.Hint(GamePage.HomeHint);
            Assert.Equal(["close", "close"], host.Asked);
            var (desk, _) = Page(desktop: true);
            desk.ChooseSection(GamePage.Details);
            Assert.All(Rows(desk).Items, r => Assert.False(r.IsEnabled));
            Assert.Equal(["Scrape This Game…", "Edit", "Cover Art", "Open Folder", "Hide from Library…"], Rows(desk).Buttons!.Select(b => b.Text));
            desk.ChooseSection(GamePage.States);
            Assert.Equal(["Load", "Delete"], ((GameContent.Tiles)desk.Content).Items[0].Buttons!.Select(b => b.Text));
        }

        [Theory]
        [InlineData(0.5, "Just now")]
        [InlineData(12, "12 min ago")]
        [InlineData(120, "2 h ago")]
        [InlineData(60 * 24, "Yesterday")]
        [InlineData(60 * 24 * 3, "3 days ago")]
        public void How_long_ago_is_said(double minutes, string said) => Assert.Equal(said, GamePage.Ago(Now.AddMinutes(-minutes), Now));
    }
}
