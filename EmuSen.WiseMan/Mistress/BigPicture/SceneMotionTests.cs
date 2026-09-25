using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Animation.Easings;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Motion;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The scene's time model on synthetic themes, with a motion of round numbers so each rule is checked by itself - see EmuSen_BigPicture.md §14.6.
    public class SceneMotionTests
    {
        private const int W = 320, H = 200;

        private static TimeSpan Ms(double ms) => TimeSpan.FromMilliseconds(ms);

        public static readonly SceneMotion Round = new()
        {
            CarouselStep = Ms(200), CarouselEasing = new QuadraticEaseOut(),
            CarouselRepeat = new(Ms(500), Ms(200)),
            CarouselFastRepeat = new(Ms(500), Ms(200), Ms(1000), Ms(100)),
            ListRepeat = new(Ms(400), Ms(100), Ms(1000), Ms(50)),
            MetadataFadeOut = Ms(100), MetadataFadeIn = Ms(200), ScrollFadeIn = Ms(300), ScrollFadeInFrom = 0.5,
            MarqueeSpeedPerEm = 2, MarqueeGapSeconds = 1,
            HorizontalContainerSpeedPerEm = 2, HorizontalContainerGapSeconds = 1, VerticalContainerSpeedPerEm = 1.5, VerticalContainerFadeIn = Ms(400),
            ViewSlide = Ms(400), ViewSlideEasing = new CubicEaseInOut(),
        };

        private static string Red => SceneAssets.Halves("mot-red", 20, 40, Colors.Red, Colors.DarkRed);

        public static SceneData Data(string system, string gamelist, SceneMotion motion, string capabilities = "", int systemIndex = 1, int game = 2)
        {
            using var theme = new SyntheticTheme();
            theme.Capabilities(capabilities).Theme($"<view name=\"system\">{system}</view><view name=\"gamelist\">{gamelist}</view>");
            var systems = SyntheticLibrary.Systems.Select(s => new SceneSystem(s.System, theme.Load(new ThemeChoices { ScreenWidth = W, ScreenHeight = H }, s.System),
                SyntheticLibrary.Games(s.System, s.Extension).Select((g, i) => i == 3 ? g with { Name = "A name far too long to fit in the narrow list it is shown in" } : g).ToList())).ToList();
            return new SceneData(systems, new Size(W, H)) { SystemIndex = systemIndex, GameIndex = game, Media = new SceneAssets.Media(), Motion = motion };
        }

        private static string Carousel(string extra = "") =>
            $"<carousel name=\"c\"><pos>0 0</pos><size>1 1</size><maxItemCount>3</maxItemCount><itemScale>1.5</itemScale><unfocusedItemOpacity>0.3</unfocusedItemOpacity><staticImage>{Red}</staticImage>{extra}</carousel>";

        private static string List(string extra = "") =>
            $"<textlist name=\"l\"><pos>0 0</pos><size>0.4 1</size><fontSize>0.08</fontSize><primaryColor>FFFFFF</primaryColor><selectedColor>FFFF00</selectedColor>{extra}</textlist>";

        private static RenderedFrame Static(SceneData data, string view) => SceneAssets.Render(SceneBuilder.Build(data.System.Theme.View(view), data));

        [Fact]
        public Task A_carousel_step_eases_to_the_next_item_and_settles_on_the_static_picture() => UiTest.Run(() =>
        {
            SceneData data = Data(Carousel(), "", Round);
            using var host = new SceneMotionHost(new SceneView(data, "system", Ms(1000)));
            RenderedFrame before = host.Frame();
            host.View.Step(1, Ms(1000));
            Assert.Equal(2, host.View.Data.SystemIndex);
            Assert.Equal(1, host.View.Position, 9);
            host.View.Advance(Ms(1100));
            Assert.Equal(1 + new QuadraticEaseOut().Ease(0.5), host.View.Position, 9);
            var carousel = Assert.IsType<ImageCarousel>(host.View.Scene.Entries.Single(e => e.Element.Type == "carousel").Control);
            Assert.Equal(1.75, carousel.Position, 9);
            Assert.True(host.View.IsMoving);
            RenderedFrame middle = host.Frame();
            RenderedFrame settled = host.At(Ms(1200));
            Assert.False(host.View.IsMoving);
            Assert.Equal(0, SceneAssets.Differing(Static(data with { SystemIndex = 2 }, "system"), settled));
            Assert.NotEqual(0, SceneAssets.Differing(middle, settled));
            Assert.NotEqual(0, SceneAssets.Differing(middle, before));
        });

        [Fact]
        public Task A_step_past_the_last_item_keeps_moving_the_same_way_and_itemTransitions_instant_jumps() => UiTest.Run(() =>
        {
            SceneData data = Data(Carousel(), "", Round, systemIndex: 4);
            var view = new SceneView(data, "system", Ms(0));
            view.Step(1, Ms(0));
            Assert.Equal(0, view.Data.SystemIndex);
            view.Advance(Ms(50));
            Assert.True(view.Position > 4 && view.Position < 5);

            var instant = new SceneView(Data(Carousel("<itemTransitions>instant</itemTransitions>"), "", Round), "system", Ms(0));
            instant.Step(-1, Ms(0));
            instant.Advance(Ms(1));
            Assert.Equal(0, instant.Position);
            Assert.False(instant.IsMoving);
        });

        [Fact]
        public Task A_held_direction_repeats_after_its_delay_and_faster_after_a_while() => UiTest.Run(() =>
        {
            var list = new SceneView(Data("", List(), Round, game: 0), "gamelist", Ms(0));
            list.Press(1, Ms(0));
            Assert.Equal(1, list.Data.GameIndex);
            list.Advance(Ms(399));
            Assert.Equal(1, list.Data.GameIndex);
            list.Advance(Ms(400));
            Assert.Equal(2, list.Data.GameIndex);
            list.Advance(Ms(900));
            Assert.Equal(7, list.Data.GameIndex);
            list.Advance(Ms(1100));
            Assert.Equal(10, list.Data.GameIndex);
            list.Release(Ms(1100));
            list.Advance(Ms(5000));
            Assert.Equal(10, list.Data.GameIndex);
        });

        [Fact]
        public Task A_list_wraps_on_a_tap_stops_at_its_end_when_held_and_jumps_at_the_fast_tier() => UiTest.Run(() =>
        {
            SceneMotion jump = Round with { ListRepeat = new(Ms(400), Ms(100), Ms(1050), Ms(50), 4) };
            string elements = List() + "<text name=\"dev\"><pos>0.5 0</pos><size>0.5 0.2</size><metadata>developer</metadata><color>FFFFFF</color></text>";
            var tap = new SceneView(Data("", elements, jump, game: 11), "gamelist", Ms(0));
            tap.Step(1, Ms(0));
            Assert.Equal(0, tap.Data.GameIndex);

            var held = new SceneView(Data("", elements, jump, game: 0), "gamelist", Ms(0));
            held.Press(1, Ms(0));
            held.Advance(Ms(1000));
            Assert.Equal(8, held.Data.GameIndex);
            held.Advance(Ms(1050));
            Assert.Equal(11, held.Data.GameIndex);
            Assert.Equal(0, OpacityOf(held, "dev"), 6);
            held.Advance(Ms(1200));
            Assert.Equal(11, held.Data.GameIndex);
            Assert.True(OpacityOf(held, "dev") > 0);
        });

        [Fact]
        public Task Fast_scrolling_takes_the_carousels_faster_tier_only_when_the_theme_asks() => UiTest.Run(() =>
        {
            var slow = new SceneView(Data(Carousel("<fastScrolling>false</fastScrolling>"), "", Round, systemIndex: 0), "system", Ms(0));
            var fast = new SceneView(Data(Carousel("<fastScrolling>true</fastScrolling>"), "", Round, systemIndex: 0), "system", Ms(0));
            foreach (SceneView v in new[] { slow, fast })
            {
                v.Press(1, Ms(0));
                v.Advance(Ms(1500));
            }

            Assert.Equal(7, slow.Target);
            Assert.Equal(10, fast.Target);
        });

        [Fact]
        public Task The_selected_name_scrolls_after_its_delay_and_starts_again_on_each_selection() => UiTest.Run(() =>
        {
            var view = new SceneView(Data("", List("<textHorizontalScrollDelay>1</textHorizontalScrollDelay>"), Round, game: 3), "gamelist", Ms(0));
            TextRowList list() => Assert.IsType<TextRowList>(view.Scene.Entries.Single(e => e.Element.Type == "textlist").Control);
            view.Advance(Ms(900));
            Assert.Equal(0, list().MarqueeOffset);
            view.Advance(Ms(1500));
            Assert.Equal(Ms(1500), list().MarqueeTime);
            Assert.Equal(0.5 * 2 * 0.08 * H, list().MarqueeOffset, 6);
            view.Step(1, Ms(1600));
            view.Step(-1, Ms(1700));
            view.Advance(Ms(2000));
            Assert.Equal(Ms(300), list().MarqueeTime);
            Assert.Equal(0, list().MarqueeOffset);

            var off = new SceneView(Data("", List("<textHorizontalScrolling>false</textHorizontalScrolling>"), Round, game: 3), "gamelist", Ms(0));
            off.Advance(Ms(10000));
            Assert.Equal(0, Assert.IsType<TextRowList>(off.Scene.Entries.Single(e => e.Element.Type == "textlist").Control).MarqueeOffset);
        });

        [Fact]
        public Task A_description_container_scrolls_by_the_themes_delays_and_the_measured_rate() => UiTest.Run(() =>
        {
            string text = "<text name=\"d\"><pos>0.5 0</pos><size>0.5 0.3</size><metadata>description</metadata><fontSize>0.06</fontSize><lineSpacing>1.5</lineSpacing>"
                + "<containerStartDelay>2</containerStartDelay><containerResetDelay>3</containerResetDelay><containerScrollSpeed>2</containerScrollSpeed><color>FFFFFF</color></text>";
            var view = new SceneView(Data("", List() + text, Round, game: 1), "gamelist", Ms(0));
            FontText d() => Assert.IsType<FontText>(view.Scene.Entries.Single(e => e.Element.Name == "d").Control);
            Assert.Equal(TextScrollDirection.Vertical, d().ScrollDirection);
            Assert.Equal(Ms(2000), d().Scroll.Delay);
            Assert.Equal(Ms(3000), d().Scroll.EndPause);
            Assert.Equal(Ms(400), d().Scroll.FadeIn);
            Assert.Equal(1.5 * d().FontSize * 2, d().Scroll.Speed, 6);
            Assert.True(d().Scroll.WholePixels);
            Assert.True(d().ScrollWholeLines);
            view.Advance(Ms(2500));
            Assert.Equal(Ms(2500), d().ScrollTime);
        });

        private static double OpacityOf(SceneView view, string name) => view.Scene.Entries.Single(e => e.Element.Name == name).Control!.Opacity;

        [Fact]
        public Task A_scrollFadeIn_image_fades_in_on_each_new_game_but_not_when_the_view_opens() => UiTest.Run(() =>
        {
            string elements = List() + "<image name=\"art\"><pos>0.5 0</pos><size>0.4 0.5</size><imageType>cover</imageType><scrollFadeIn>true</scrollFadeIn><opacity>0.8</opacity></image>"
                + "<image name=\"plain\"><pos>0.5 0.5</pos><size>0.4 0.5</size><imageType>cover</imageType></image>";
            var view = new SceneView(Data("", elements, Round), "gamelist", Ms(0));
            Assert.Equal(0.8, OpacityOf(view, "art"), 6);
            view.Step(1, Ms(1000));
            Assert.Equal(0.4, OpacityOf(view, "art"), 6);
            view.Advance(Ms(1150));
            Assert.Equal(0.6, OpacityOf(view, "art"), 6);
            Assert.Equal(1, OpacityOf(view, "plain"), 6);
            view.Advance(Ms(1300));
            Assert.Equal(0.8, OpacityOf(view, "art"), 6);
        });

        [Fact]
        public Task Metadata_fades_out_while_the_list_scrolls_fast_and_back_in_when_it_stops() => UiTest.Run(() =>
        {
            string elements = List() + "<text name=\"dev\"><pos>0.5 0</pos><size>0.5 0.2</size><metadata>developer</metadata><color>FFFFFF</color></text>"
                + "<text name=\"sys\"><pos>0.5 0.5</pos><size>0.5 0.2</size><metadata>systemName</metadata><color>FFFFFF</color></text>";
            var view = new SceneView(Data("", elements, Round, game: 0), "gamelist", Ms(0));
            view.Press(1, Ms(0));
            view.Advance(Ms(399));
            Assert.Equal(1, OpacityOf(view, "dev"), 6);
            view.Advance(Ms(450));
            Assert.Equal(0.5, OpacityOf(view, "dev"), 6);
            Assert.Equal(1, OpacityOf(view, "sys"), 6);
            view.Advance(Ms(1050));
            Assert.Equal(0, OpacityOf(view, "dev"), 6);
            view.Release(Ms(1200));
            Assert.Equal(0, OpacityOf(view, "dev"), 6);
            view.Advance(Ms(1300));
            Assert.Equal(0.5, OpacityOf(view, "dev"), 6);
            view.Advance(Ms(1400));
            Assert.Equal(1, OpacityOf(view, "dev"), 6);
        });

        [Fact]
        public Task A_slide_moves_both_views_across_and_settles_on_the_gamelist_alone() => UiTest.Run(() =>
        {
            string capabilities = "<transitions name=\"slide\"><systemToGamelist>slide</systemToGamelist><gamelistToSystem>slide</gamelistToSystem></transitions>";
            SceneData data = Data(Carousel(), List(), Round, capabilities);
            var stage = new SceneStage(data, "system", Ms(0));
            using var window = new SceneMotionHost(stage.Root, W, H);
            stage.Switch(Ms(1000));
            Assert.Equal(2, stage.Root.Children.Count);
            stage.Advance(Ms(1200));
            var leaving = (TranslateTransform)stage.Root.Children[0].RenderTransform!;
            var coming = (TranslateTransform)stage.Root.Children[1].RenderTransform!;
            Assert.Equal(-H * new CubicEaseInOut().Ease(0.5), leaving.Y, 6);
            Assert.Equal(H * (1 - new CubicEaseInOut().Ease(0.5)), coming.Y, 6);
            Assert.Equal(0, leaving.X);
            stage.Advance(Ms(1400));
            Assert.Single(stage.Root.Children);
            Assert.Equal(0, SceneAssets.Differing(Static(data, "gamelist"), window.Frame()));

            var instant = new SceneStage(Data(Carousel(), List(), Round, "<transitions name=\"instant\"><systemToGamelist>instant</systemToGamelist></transitions>"), "system", Ms(0));
            instant.Switch(Ms(0));
            Assert.Single(instant.Root.Children);
            Assert.Equal("gamelist", instant.Current.ViewName);
        });
    }
}
