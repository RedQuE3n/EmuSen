using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The badges Mistress draws itself for a theme that names no image, and Art Book Next's own left as they were - see EmuSen_BigPicture.md §29.
    public class BuiltInBadgesTests
    {
        private readonly ITestOutputHelper _out;

        public BuiltInBadgesTests(ITestOutputHelper output) => _out = output;

        private const int W = 1280, H = 800;

        private static readonly string[] Documented = ["collection", "folder", "favorite", "completed", "kidgame", "broken", "controller", "altemulator", "manual"];

        private static SceneGame Flagged(SceneGame g) => g with
        {
            InCollection = true, Folder = true, FolderLink = true, Favorite = true, Completed = true, KidGame = true, Broken = true,
            Controller = "gamepad_nintendo_64", AltEmulator = true, Manual = true,
        };

        private static SceneBuilder Scene(string badges, bool flagged)
        {
            using var theme = new SyntheticTheme();
            theme.Capabilities("").Theme("<view name=\"gamelist\"><text name=\"title\"><pos>0.05 0.6</pos><size>0.9 0.1</size><text>Badges below the line</text><fontSize>0.05</fontSize><color>FFFFFF</color></text>" +
                badges + "</view>");
            var system = new ThemeSystem("snes", "Super Nintendo", "snes");
            List<SceneGame> games = SyntheticLibrary.Games(system, ".sfc").ToList();
            games[0] = flagged ? Flagged(games[0]) : games[0] with { Favorite = false, Completed = false, KidGame = false, Broken = false, InCollection = false, Folder = false, AltEmulator = false, Controller = null, Manual = false };
            var data = new SceneData([new SceneSystem(system, theme.Load(new ThemeChoices { ScreenWidth = W, ScreenHeight = H }, system), games)], new Size(W, H)) { GameIndex = 0 };
            return SceneBuilder.Build(data.System.Theme.View("gamelist"), data);
        }

        private const string AllSlots = "<badges name=\"badges\"><pos>0.05 0.05</pos><size>0.9 0.2</size><slots>all</slots><lines>1</lines><itemsPerLine>9</itemsPerLine><itemMargin>0.01 0</itemMargin></badges>";

        // P105's second half: with no image named, each of the nine slots draws Mistress's own badge, in THEMES.md's order, and nothing outside the element changes.
        [Fact]
        public Task On_a_theme_that_names_no_image_each_of_the_nine_slots_draws_and_nothing_outside_the_badges_changes() => UiTest.Run(() =>
        {
            SceneBuilder plain = Scene(AllSlots, flagged: false), flagged = Scene(AllSlots, flagged: true);
            BadgeStrip strip = Assert.IsType<BadgeStrip>(flagged.Find("badges", "badges")!.Control);
            Assert.Equal(new[] { BadgeKind.Collection, BadgeKind.Folder, BadgeKind.Favorite, BadgeKind.Completed, BadgeKind.KidGame, BadgeKind.Broken, BadgeKind.Controller, BadgeKind.AltEmulator, BadgeKind.Manual },
                strip.Entries!.Select(e => e.Kind));
            Assert.All(strip.Entries!, e => Assert.Null(e.IconPath));
            Assert.Equal(ControllerShape.Nintendo64, strip.Entries!.Single(e => e.Kind == BadgeKind.Controller).Controller);
            Assert.True(strip.Entries!.Single(e => e.Kind == BadgeKind.Folder).Linked);
            Assert.Empty(((BadgeStrip)plain.Find("badges", "badges")!.Control!).Entries!);

            RenderedFrame a = SceneAssets.Render(plain), b = SceneAssets.Render(flagged);
            var box = new Rect(0.05 * W, 0.05 * H, 0.9 * W, 0.2 * H).Inflate(1);
            (int inside, int outside) = Split(a, b, box);
            _out.WriteLine($"{inside} pixels changed inside the badges' box, {outside} outside");
            Assert.Equal(0, outside);
            IReadOnlyList<Rect> cells = strip.Cells(new Size(0.9 * W, 0.2 * H));
            Assert.Equal(9, cells.Count);
            for (int i = 0; i < 9; i++)
            {
                Rect cell = cells[i].Translate(new Vector(0.05 * W, 0.05 * H));
                Assert.True(Split(a, b, cell).Inside > 150, $"{Documented[i]} drew {Split(a, b, cell).Inside} pixels");
            }
        });

        // A theme that names some images: those slots show its files, the rest Mistress's drawings; a named file that is missing falls back too.
        [Fact]
        public Task A_theme_s_named_image_wins_and_a_missing_one_falls_back_to_the_drawing() => UiTest.Run(() =>
        {
            string red = SceneAssets.Halves("badge-red", 20, 20, Colors.Red, Colors.Red);
            string xml = AllSlots.Replace("</badges>", $"<customBadgeIcon badge=\"favorite\">{red}</customBadgeIcon><customBadgeIcon badge=\"broken\">./missing-broken.svg</customBadgeIcon>" +
                $"<customControllerIcon controller=\"gamepad_nintendo_64\">{red}</customControllerIcon></badges>");
            BadgeStrip strip = (BadgeStrip)Scene(xml, flagged: true).Find("badges", "badges")!.Control!;
            Assert.Equal(red, strip.Entries!.Single(e => e.Kind == BadgeKind.Favorite).IconPath);
            Assert.Null(strip.Entries!.Single(e => e.Kind == BadgeKind.Broken).IconPath);
            Assert.Equal(red, strip.Entries!.Single(e => e.Kind == BadgeKind.Controller).ControllerIconPath);
            Assert.All(strip.Entries!.Where(e => e.Kind is not (BadgeKind.Favorite)), e => Assert.Null(e.IconPath));
        });

        // ES-DE's "all": the slots a theme names first in its order, then the rest in THEMES.md's; the controller shapes Mistress draws for its consoles, a generic pad for another pad, unknown otherwise.
        [Fact]
        public void All_keeps_the_named_slots_first_and_each_controller_type_has_its_shape()
        {
            Assert.Equal(new[] { "manual", "favorite", "collection", "folder", "completed", "kidgame", "broken", "controller", "altemulator" },
                SceneBadges.Slots(["manual", "favorite", "all"]));
            Assert.Equal(new[] { "favorite", "broken" }, SceneBadges.Slots(["favorite", "broken"]));
            Assert.Equal(ControllerShape.Nes, SceneBadges.ShapeOf("gamepad_nintendo_nes"));
            Assert.Equal(ControllerShape.Snes, SceneBadges.ShapeOf("gamepad_nintendo_snes"));
            Assert.Equal(ControllerShape.Nintendo64, SceneBadges.ShapeOf("gamepad_nintendo_64"));
            Assert.Equal(ControllerShape.Gamepad, SceneBadges.ShapeOf("gamepad_generic"));
            Assert.Equal(ControllerShape.Gamepad, SceneBadges.ShapeOf("gamepad_xbox"));
            Assert.Equal(ControllerShape.Unknown, SceneBadges.ShapeOf("unknown"));
            Assert.Equal(ControllerShape.Unknown, SceneBadges.ShapeOf("lightgun_generic"));
        }

        // P105's first half: Art Book Next names an image for each of its five slots, so drawing its badges through the entries changes no pixel of its gamelist against drawing its files as before.
        [ArtBookNextFact]
        public Task Built_in_badges_change_no_pixel_of_Art_Book_Next_s_gamelist() => UiTest.Run(() =>
        {
            SceneBuilder Build()
            {
                IReadOnlyList<SceneSystem> systems = SyntheticLibrary.Load(ArtBookNextFactAttribute.Folder, new ThemeChoices { ScreenWidth = W, ScreenHeight = H });
                var flagged = systems.Select(s => s with { Games = s.Games.Select((g, i) => i == 0 ? Flagged(g) with { Folder = false, FolderLink = false } : g).ToList() }).ToList();
                var data = new SceneData(flagged, new Size(W, H)) { SystemIndex = 1, GameIndex = 0, Media = new SceneAssets.Media() };
                return SceneBuilder.Build(data.System.Theme.View("gamelist"), data);
            }

            SceneBuilder now = Build();
            BadgeStrip strip = now.Entries.Select(e => e.Control).OfType<BadgeStrip>().Single();
            Assert.Equal(new[] { BadgeKind.Favorite, BadgeKind.Completed, BadgeKind.Collection, BadgeKind.AltEmulator }, strip.Entries!.Select(e => e.Kind));
            Assert.All(strip.Entries!, e => Assert.NotNull(e.IconPath));
            RenderedFrame entries = SceneAssets.Render(now);

            SceneBuilder before = Build();
            BadgeStrip old = before.Entries.Select(e => e.Control).OfType<BadgeStrip>().Single();
            old.Icons = old.Entries!.Select(e => e.IconPath!).ToArray();
            old.Entries = null;
            RenderedFrame icons = SceneAssets.Render(before);
            Assert.Equal(0, SceneAssets.Differing(entries, icons));
        });

        // The bounding box of the pixels that changed outside a box, for a failure's message.
        internal static Rect Changed(RenderedFrame a, RenderedFrame b, Rect box)
        {
            double l = double.MaxValue, t = double.MaxValue, r = 0, bottom = 0;
            for (int y = 0; y < a.Height; y++)
                for (int x = 0; x < a.Width; x++)
                {
                    int i = (y * a.Width + x) * 4;
                    if ((a.Rgba[i] == b.Rgba[i] && a.Rgba[i + 1] == b.Rgba[i + 1] && a.Rgba[i + 2] == b.Rgba[i + 2]) || box.Contains(new Point(x + 0.5, y + 0.5))) continue;
                    (l, t, r, bottom) = (Math.Min(l, x), Math.Min(t, y), Math.Max(r, x + 1), Math.Max(bottom, y + 1));
                }
            return r == 0 ? default : new Rect(l, t, r - l, bottom - t);
        }

        internal static (int Inside, int Outside) Split(RenderedFrame a, RenderedFrame b, Rect box)
        {
            int inside = 0, outside = 0;
            for (int y = 0; y < a.Height; y++)
                for (int x = 0; x < a.Width; x++)
                {
                    int i = (y * a.Width + x) * 4;
                    if (a.Rgba[i] == b.Rgba[i] && a.Rgba[i + 1] == b.Rgba[i + 1] && a.Rgba[i + 2] == b.Rgba[i + 2]) continue;
                    if (box.Contains(new Point(x + 0.5, y + 0.5))) inside++; else outside++;
                }
            return (inside, outside);
        }
    }
}
