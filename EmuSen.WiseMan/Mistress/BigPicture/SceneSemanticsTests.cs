using System;
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

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // What each property is mapped to, not only that it has an effect: axes, precedence, formats and ES-DE's measured rules - see EmuSen_BigPicture.md §13.9.
    public class SceneSemanticsTests
    {
        private const int W = 320, H = 200;

        private static SceneBuilder Scene(string view, string elements, int system = 1, int game = 4)
        {
            using var theme = new SyntheticTheme();
            theme.Capabilities("").Theme($"<view name=\"{view}\">{elements}</view>");
            var systems = SyntheticLibrary.Systems.Select(s => new SceneSystem(s.System, theme.Load(new ThemeChoices { ScreenWidth = W, ScreenHeight = H }, s.System), SyntheticLibrary.Games(s.System, s.Extension))).ToList();
            var collection = new ThemeSystem("all", "All Games", "auto-allgames", ThemeSystemKind.AutoCollection);
            systems.Add(new SceneSystem(collection, theme.Load(new ThemeChoices { ScreenWidth = W, ScreenHeight = H }, collection), SyntheticLibrary.Games(collection, ".nes")));
            var data = new SceneData(systems, new Size(W, H)) { SystemIndex = system, GameIndex = game, Media = new SceneAssets.Media() };
            SceneBuilder scene = SceneBuilder.Build(data.System.Theme.View(view), data);
            scene.Canvas.Measure(new Size(W, H));
            scene.Canvas.Arrange(new Rect(0, 0, W, H));
            return scene;
        }

        private static T Control<T>(SceneBuilder scene, string type) where T : Control =>
            Assert.IsType<T>(scene.Entries.Single(e => e.Element.Type == type && e.Element.Name == "x").Control);

        private static void Near(Thickness expected, Thickness actual) =>
            Assert.True(Math.Abs(expected.Left - actual.Left) + Math.Abs(expected.Top - actual.Top) + Math.Abs(expected.Right - actual.Right) + Math.Abs(expected.Bottom - actual.Bottom) < 0.001, $"{expected} against {actual}");

        private static string Red => SceneAssets.Halves("sem-red", 20, 20, Colors.Red, Colors.Red);
        private static string Blue => SceneAssets.Halves("sem-blue", 20, 20, Colors.Blue, Colors.Blue);

        [Fact]
        public Task A_higher_zIndex_draws_on_top_whatever_the_order_of_definition() => UiTest.Run(() =>
        {
            SceneBuilder scene = Scene("gamelist", $"<image name=\"top\"><pos>0 0</pos><size>1 1</size><path>{Blue}</path><zIndex>40</zIndex></image>"
                + $"<image name=\"under\"><pos>0 0</pos><size>1 1</size><path>{Red}</path><zIndex>30</zIndex></image>");
            RenderedFrame f = SceneAssets.Render(scene);
            int i = (100 * W + 200) * 4;
            Assert.Equal((0, 0, 255), (f.Rgba[i], f.Rgba[i + 1], f.Rgba[i + 2]));
        });

        [Fact]
        public Task Corner_radii_and_horizontal_paddings_scale_by_the_width_font_sizes_and_vertical_paddings_by_the_height() => UiTest.Run(() =>
        {
            Assert.Equal(0.05 * W, Control<FittedImage>(Scene("gamelist", $"<image name=\"x\"><size>0.5 0.5</size><path>{Red}</path><cornerRadius>0.05</cornerRadius></image>"), "image").CornerRadius, 3);
            Assert.Equal(0.1 * H, Control<FontText>(Scene("gamelist", "<text name=\"x\"><text>Words</text><fontSize>0.1</fontSize></text>"), "text").FontSize, 3);
            var clock = Control<ClockLabel>(Scene("gamelist", "<clock name=\"x\"><backgroundColor>FF0000</backgroundColor><backgroundHorizontalPadding>0.05 0.1</backgroundHorizontalPadding><backgroundVerticalPadding>0.05 0.1</backgroundVerticalPadding></clock>"), "clock");
            Near(new Thickness(0.05 * W, 0.05 * H, 0.1 * W, 0.1 * H), clock.Padding);
        });

        // ES-DE 3.4.1 grows the clock's, the status's and the help bar's backgrounds outward from the positioned box (§13.8).
        [Fact]
        public Task Special_elements_grow_their_backgrounds_outward_from_the_positioned_box() => UiTest.Run(() =>
        {
            SceneBuilder scene = Scene("gamelist", "<clock name=\"x\"><pos>0.5 0.5</pos><backgroundColor>FF0000</backgroundColor><backgroundHorizontalPadding>0.05 0.05</backgroundHorizontalPadding><backgroundVerticalPadding>0.05 0.05</backgroundVerticalPadding></clock>");
            var clock = Control<ClockLabel>(scene, "clock");
            Near(new Thickness(-0.05 * W, -0.05 * H, -0.05 * W, -0.05 * H), clock.Margin);
            Assert.Equal(W / 2.0 - 0.05 * W, clock.Bounds.X, 3);
            Assert.Equal(H / 2.0 - 0.05 * H, clock.Bounds.Y, 3);
        });

        // THEMES.md: "If size is not defined then maxSize will be used."
        [Fact]
        public Task Size_wins_over_maxSize_and_a_videos_image_sizes_win_over_its_own() => UiTest.Run(() =>
        {
            var image = Control<FittedImage>(Scene("gamelist", $"<image name=\"x\"><size>0.5 0.5</size><maxSize>0.2 0.2</maxSize><path>{Red}</path></image>"), "image");
            Assert.Equal(new Size(0.5 * W, 0.5 * H), image.Bounds.Size);
            var video = Control<FittedImage>(Scene("gamelist", "<video name=\"x\"><size>0.5 0.6</size><imageMaxSize>0.2 0.2</imageMaxSize><imageType>cover</imageType></video>"), "video");
            Assert.True(video.Bounds.Width <= 0.2 * W + 0.01 && video.Bounds.Height <= 0.2 * H + 0.01, video.Bounds.ToString());
        });

        [Fact]
        public Task A_container_cuts_without_an_ellipsis_and_other_text_ends_in_one() => UiTest.Run(() =>
        {
            Assert.Null(Control<FontText>(Scene("gamelist", "<text name=\"x\"><size>0.5 0.1</size><text>Words</text><container>true</container></text>"), "text").Ellipsis);
            Assert.Equal("...", Control<FontText>(Scene("gamelist", "<text name=\"x\"><size>0.5 0.1</size><text>Words</text><container>false</container></text>"), "text").Ellipsis);
        });

        [Theory]
        [InlineData("%Y-%m-%d", "1994-05-05")]
        [InlineData("%d/%m/%y", "05/05/94")]
        [InlineData("%b %e, %Y", "May 5, 1994")]
        [InlineData("Released %Y", "Released 1994")]
        [InlineData("%%%Y", "%1994")]
        public Task A_datetime_formats_by_strftime(string format, string expected) => UiTest.Run(() =>
        {
            var text = Control<FontText>(Scene("gamelist", $"<datetime name=\"x\"><metadata>releasedate</metadata><format>{format}</format></datetime>"), "datetime");
            Assert.Equal(expected, text.Text);
        });

        [Fact]
        public Task Badges_show_only_the_slots_the_game_has() => UiTest.Run(() =>
        {
            string icon = SceneAssets.Svg("sem-badge", "<rect width='20' height='20' fill='#fff'/>");
            var badges = Control<BadgeStrip>(Scene("gamelist", $"<badges name=\"x\"><slots>favorite,completed,broken</slots><customBadgeIcon badge=\"favorite\">{icon}</customBadgeIcon><customBadgeIcon badge=\"completed\">{icon}</customBadgeIcon><customBadgeIcon badge=\"broken\">{icon}</customBadgeIcon></badges>", game: 0), "badges");
            Assert.Single(badges.Icons!);
        });

        [Theory]
        [InlineData("battery", DeviceIndicators.Battery | DeviceIndicators.BatteryPercentage)]
        [InlineData("wifi,bluetooth", DeviceIndicators.Wifi | DeviceIndicators.Bluetooth)]
        [InlineData("all", DeviceIndicators.All)]
        public Task Status_entries_choose_the_indicators(string entries, DeviceIndicators expected) => UiTest.Run(() =>
            Assert.Equal(expected, Control<DeviceStatusBar>(Scene("gamelist", $"<systemstatus name=\"x\"><entries>{entries}</entries></systemstatus>"), "systemstatus").Indicators));

        [Fact]
        public Task The_system_name_suffix_is_for_collections_only() => UiTest.Run(() =>
        {
            const string element = "<text name=\"x\"><metadata>name</metadata><systemNameSuffix>true</systemNameSuffix></text>";
            Assert.Equal("Ember Circuit", Control<FontText>(Scene("gamelist", element), "text").Text);
            Assert.Equal("Ember Circuit [ALL]", Control<FontText>(Scene("gamelist", element, system: 5), "text").Text);
            const string list = "<textlist name=\"x\"><systemNameSuffix>true</systemNameSuffix></textlist>";
            Assert.Equal("Aurora Drift", Control<TextRowList>(Scene("gamelist", list), "textlist").Items![0].Text);
            Assert.Equal("Aurora Drift [ALL]", Control<TextRowList>(Scene("gamelist", list, system: 5), "textlist").Items![0].Text);
        });
    }
}
