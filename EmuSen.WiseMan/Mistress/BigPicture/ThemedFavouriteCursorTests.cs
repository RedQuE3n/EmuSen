using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // A favourite's toggle moves the game, never the highlight: the row, the scroll and what the details show - see EmuSen_BigPicture.md §44.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedFavouriteCursorTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedFavouriteCursorTests).GetTypeInfo().Assembly);

        // Thirty SNES games, longer than the list's window, so a jump to the top would scroll it.
        private static readonly string[] Extra = Enumerable.Range(1, 25).Select(i => $"Lark {i:00} (Synthetic)").ToArray();

        private const string Title = "<text name=\"title\"><pos>0.55 0.85</pos><size>0.4 0.05</size><metadata>name</metadata><fontSize>0.03</fontSize><color>FFFFFF</color></text>";

        private static void Roms(string directory)
        {
            foreach (string g in Extra) File.WriteAllBytes(Path.Combine(directory, g + ".sfc"), SyntheticRom.BuildBlank());
        }

        private static Task Run(Action<ThemedSession> test, Action<AppSettings>? settings = null, string? theme = null) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(settings: settings, extraGamelist: Title, roms: Roms, themeDirectory: theme);
            test(s);
        }, default);

        private static SceneView View(ThemedSession s) => s.Themed.Stage!.Current;

        private static string[] Listed(ThemedSession s) => View(s).Data.System.Games.Select(g => g.Name).ToArray();

        private static TextRowList Rows(ThemedSession s) => View(s).Scene.Entries.Select(e => e.Control).OfType<TextRowList>().Single();

        // The details follow the highlight: the name the theme's metadata text shows is the selected game's, with a collection's system suffix.
        private static void DetailsFollow(ThemedSession s)
        {
            s.Settle();
            string? shown = ((FontText)View(s).Scene.Find("text", "title")!.Control!).Text;
            Assert.True(shown == s.Game || shown == s.Game + " [SNES]", $"the details show {shown} under {s.Game}");
        }

        [Fact]
        public Task A_favourite_moves_to_the_top_while_the_highlight_and_the_scroll_stay_where_they_were() => Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Down(20);
            string[] before = Listed(s);
            Assert.Equal(30, before.Length);
            int first = Rows(s).FirstVisible;
            Assert.True(first > 0, "the list must be scrolled for the test to mean anything");
            string chosen = before[20];

            s.Pad.Y();
            Assert.Equal(chosen, Listed(s)[0]);
            Assert.True(View(s).Data.System.Games[0].Favorite);
            Assert.Equal(20, View(s).Index);
            Assert.Equal(20, Rows(s).SelectedIndex);
            Assert.Equal(first, Rows(s).FirstVisible);
            Assert.Equal(before[19], s.Game);
            DetailsFollow(s);

            // One press down reaches the game that came after the favourite, as it would have before.
            s.Pad.Down();
            Assert.Equal(before[21], s.Game);
            DetailsFollow(s);
        });

        [Fact]
        public Task Unfavouriting_the_game_at_the_top_keeps_the_highlight_at_the_top() => Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            string[] before = Listed(s);
            s.Pad.Down(7);
            s.Pad.Y();
            s.Pad.L2();
            Assert.Equal(0, View(s).Index);
            Assert.Equal(before[7], s.Game);

            s.Pad.Y();
            Assert.Equal(before, Listed(s));
            Assert.Equal(0, View(s).Index);
            Assert.Equal(0, Rows(s).FirstVisible);
            Assert.Equal(before[0], s.Game);
            Assert.False(View(s).Data.System.Games.Any(g => g.Favorite));
            DetailsFollow(s);
        });

        [Fact]
        public Task Favouriting_the_last_game_keeps_the_highlight_on_the_last_row() => Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            string[] before = Listed(s);
            s.Pad.R2();
            int last = before.Length - 1, first = Rows(s).FirstVisible;
            Assert.Equal(last, View(s).Index);

            s.Pad.Y();
            Assert.Equal(before[last], Listed(s)[0]);
            Assert.Equal(last, View(s).Index);
            Assert.Equal(first, Rows(s).FirstVisible);
            Assert.Equal(before[last - 1], s.Game);
            DetailsFollow(s);
        });

        [Fact]
        public Task In_all_games_a_favourite_moves_to_the_top_and_the_highlight_stays() => Run(s =>
        {
            ThemedCollectionsTests.Enter(s, "all");
            string[] before = Listed(s);
            s.Pad.Down(12);
            s.Pad.Y();
            Assert.Equal(before[12], Listed(s)[0]);
            Assert.Equal(12, View(s).Index);
            Assert.Equal(before[11], s.Game);
            DetailsFollow(s);
        }, ThemedCollectionsTests.AllAuto);

        [Fact]
        public Task In_the_favorites_collection_an_unfavourited_game_leaves_and_the_highlight_keeps_its_row_within_the_shorter_list() => Run(s =>
        {
            foreach (string g in Extra.Take(4)) ThemedCollectionsTests.Records(s).ToggleFavourite(Path.Combine(s.RomDirectory, g + ".sfc"));
            ThemedCollectionsTests.Refresh(s);
            ThemedCollectionsTests.Enter(s, "favorites");
            Assert.Equal(Extra.Take(4), Listed(s));

            s.Pad.Down(1);
            s.Pad.Y();
            Assert.Equal(new[] { Extra[0], Extra[2], Extra[3] }, Listed(s));
            Assert.Equal(1, View(s).Index);
            Assert.Equal(Extra[2], s.Game);
            DetailsFollow(s);

            // The last row's game leaves: the highlight takes the new last row.
            s.Pad.R2();
            s.Pad.Y();
            Assert.Equal(new[] { Extra[0], Extra[2] }, Listed(s));
            Assert.Equal(1, View(s).Index);
            Assert.Equal(Extra[2], s.Game);
            DetailsFollow(s);
        }, ThemedCollectionsTests.AllAuto);

        [Fact]
        public Task Unfavouriting_the_last_game_of_favorites_returns_to_the_system_view_at_the_collection_s_place() => Run(s =>
        {
            ThemedCollectionsTests.Records(s).ToggleFavourite(Path.Combine(s.RomDirectory, Extra[5] + ".sfc"));
            ThemedCollectionsTests.Refresh(s);
            Assert.Equal(["nes", "gb", "snes", "all", "favorites"], ThemedCollectionsTests.Systems(s));
            ThemedCollectionsTests.Enter(s, "favorites");

            s.Pad.Y();
            Assert.Equal(["nes", "gb", "snes", "all"], ThemedCollectionsTests.Systems(s));
            Assert.Equal(("system", "all"), (s.View, s.System));
            Assert.Null(s.Themed.SelectedGame);
        }, ThemedCollectionsTests.AllAuto);

        [Fact]
        public Task The_game_options_favourite_entry_keeps_the_highlight_too() => Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            string[] before = Listed(s);
            s.Pad.Down(9);
            ThemedGameOptionsTests.Choose(s, "Add to Favourites");
            Assert.Equal(before[9], Listed(s)[0]);
            Assert.Equal(9, View(s).Index);
            Assert.Equal(before[8], s.Game);
            DetailsFollow(s);
        });

        // The session's theme with its list replaced by another primary element.
        private static SyntheticTheme Primary(string element)
        {
            var theme = new SyntheticTheme();
            ThemedSession.Write(theme, Title);
            string xml = File.ReadAllText(theme.PathOf("theme.xml"));
            int a = xml.IndexOf("<textlist name=\"gamelist\">"), b = xml.IndexOf("</textlist>") + "</textlist>".Length;
            File.WriteAllText(theme.PathOf("theme.xml"), xml[..a] + element + xml[b..]);
            return theme;
        }

        private static double? GridRow(ThemedSession s) => View(s).Scene.Entries.Select(e => e.Control).OfType<ImageGrid>().SingleOrDefault()?.ScrollRow;

        [Theory]
        [InlineData("grid")]
        [InlineData("carousel")]
        public Task A_grid_and_a_carousel_keep_the_highlight_where_it_was(string type) => Session.Dispatch(() =>
        {
            using SyntheticTheme theme = Primary(type == "grid"
                ? "<grid name=\"gamelist\"><pos>0.05 0.08</pos><size>0.45 0.7</size><itemSize>0.1 0.2</itemSize><itemSpacing>0.01 0.01</itemSpacing><itemScale>1</itemScale><imageType>cover</imageType><textColor>FFFFFF</textColor></grid>"
                : "<carousel name=\"gamelist\"><pos>0 0.2</pos><size>1 0.4</size><type>horizontal</type><maxItemCount>5</maxItemCount><itemTransitions>instant</itemTransitions><imageType>cover</imageType></carousel>");
            using var s = new ThemedSession(extraGamelist: Title, roms: Roms, themeDirectory: theme.Root);
            ThemedLibraryPadTests.Enter(s, "snes");
            string[] before = Listed(s);
            s.Pad.Right(13);
            s.Run(300);
            Assert.Equal(13, View(s).Index);
            double? scroll = GridRow(s);
            double position = View(s).Position;
            Assert.True(type != "grid" || scroll > 0, "the grid must be scrolled for the test to mean anything");

            s.Pad.Y();
            s.Run(300);
            Assert.Equal(before[13], Listed(s)[0]);
            Assert.Equal(13, View(s).Index);
            Assert.Equal(before[12], s.Game);
            Assert.Equal(scroll, GridRow(s));
            if (type == "carousel") Assert.Equal(position, View(s).Position);
            DetailsFollow(s);
        }, default);
    }
}
