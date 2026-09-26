using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Mistress.Scraping;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // §5.4's region, language and rating rules, each by itself - see EmuSen_BigPicture.md §17.
    public class ScrapeRulesTests
    {
        [Theory]
        [InlineData("Super Mario World (USA).sfc", "us")]
        [InlineData("Super Mario 64 (Europe) (En,Fr,De).z64", "eu")]
        [InlineData("Mother 3 (Japan).gba", "jp")]
        [InlineData("Tetris (World) (Rev 1).gb", "wor")]
        [InlineData("Pac-Man (USA, Europe).nes", "us")]
        [InlineData("Sutte Hakkun (Japan, USA).sfc", "jp")]
        [InlineData("Game (En,Fr,De) (Germany).sfc", "de")]
        [InlineData("My Hack.sfc", null)]
        [InlineData("Game (Proto).sfc", null)]
        public void The_region_is_the_first_tag_whose_every_name_is_a_country(string file, string? region)
        {
            Assert.Equal(region, ScrapeRules.RegionOfFileName(file));
        }

        [Fact]
        public void The_order_is_the_preferred_region_then_es_de_s_fallback_list_only_with_the_fallback_on()
        {
            Assert.Equal(["eu", "wor", "us", "jp", "ss"], ScrapeRules.RegionOrder("Game (Europe).sfc", ScrapeRules.AutomaticRegion, fallback: true));
            Assert.Equal(["eu"], ScrapeRules.RegionOrder("Game (Europe).sfc", ScrapeRules.AutomaticRegion, fallback: false));
            Assert.Equal(["jp", "wor", "us", "eu", "ss"], ScrapeRules.RegionOrder("Game (Europe).sfc", "jp", fallback: true));
            Assert.Equal(["wor", "us", "eu", "jp", "ss"], ScrapeRules.RegionOrder("Untagged.sfc", ScrapeRules.AutomaticRegion, fallback: true));
        }

        private static readonly List<ScrapedMedia> Boxes =
        [
            new("box-2D", "jp", "u/jp", "png"), new("box-2D", "us", "u/us", "png"), new("box-2D", "wor", "u/wor", "jpg"),
            new("wheel", "us", "u/wheel", "png"), new("wheel-hd", "eu", "u/wheel-hd", "png"), new("fanart", null, "u/fanart", "jpg"),
        ];

        [Fact]
        public void A_media_kind_takes_the_first_region_in_the_order_and_its_first_screen_scraper_type()
        {
            Assert.Equal("u/us", ScrapeRules.ChooseMedia(Boxes, ScrapeRules.Cover, ["us", "wor", "eu", "jp", "ss"], true)!.Url);
            Assert.Equal("u/wor", ScrapeRules.ChooseMedia(Boxes, ScrapeRules.Cover, ["eu", "wor", "us", "jp", "ss"], true)!.Url);
            Assert.Equal("u/jp", ScrapeRules.ChooseMedia(Boxes, ScrapeRules.Cover, ["jp"], false)!.Url);

            // wheel-hd before wheel, whatever the region order says of wheel
            Assert.Equal("u/wheel-hd", ScrapeRules.ChooseMedia(Boxes, ScrapeRules.Marquee, ["us", "eu"], true)!.Url);
        }

        [Fact]
        public void Without_the_fallback_a_kind_with_no_file_in_the_preferred_region_is_left_out()
        {
            Assert.Null(ScrapeRules.ChooseMedia(Boxes, ScrapeRules.Cover, ["eu"], fallback: false));
            Assert.Equal("u/jp", ScrapeRules.ChooseMedia(Boxes, ScrapeRules.Cover, ["eu"], fallback: true)!.Url);
            Assert.Null(ScrapeRules.ChooseMedia(Boxes, ScrapeRules.Screenshot, ["us"], fallback: true));
        }

        [Fact]
        public void A_file_with_no_region_is_taken_when_no_region_in_the_order_has_one()
        {
            var kind = new ScrapeMediaKind("fanart", "fanart", ["fanart"]);
            Assert.Equal("u/fanart", ScrapeRules.ChooseMedia(Boxes, kind, ["eu"], fallback: false)!.Url);
        }

        [Fact]
        public void Text_is_chosen_by_the_order_then_any_only_with_the_fallback()
        {
            var synopses = new List<Localised> { new("fr", "Français"), new("de", "Deutsch") };
            Assert.Equal("Deutsch", ScrapeRules.ChooseText(synopses, ScrapeRules.LanguageOrder("de"), true)!.Text);
            Assert.Null(ScrapeRules.ChooseText(synopses, ScrapeRules.LanguageOrder("en"), false));
            Assert.Equal("Français", ScrapeRules.ChooseText(synopses, ScrapeRules.LanguageOrder("en"), true)!.Text);
            Assert.Equal(["de", "en"], ScrapeRules.LanguageOrder("de"));
            Assert.Equal(["en"], ScrapeRules.LanguageOrder("en"));
        }

        [Fact]
        public void The_genre_is_the_main_one_in_the_language_order()
        {
            var genres = new List<ScrapedGenre>
            {
                new([new("en", "Other")], false),
                new([new("fr", "Course"), new("en", "Racing")], true),
            };
            Assert.Equal("Racing", ScrapeRules.Genre(genres, ["en"]));
            Assert.Equal("Course", ScrapeRules.Genre(genres, ["fr", "en"]));
            Assert.Equal("Other", ScrapeRules.Genre([genres[0]], ["en"]));
        }

        [Theory]
        [InlineData(16.0, 0.8f)]
        [InlineData(15.0, 0.8f)]
        [InlineData(13.0, 0.7f)]
        [InlineData(20.0, 1.0f)]
        [InlineData(0.0, 0.0f)]
        [InlineData(25.0, 1.0f)]
        public void The_rating_is_the_note_over_20_to_the_nearest_tenth(double note, float rating)
        {
            Assert.Equal(rating, ScrapeRules.Rating(note));
        }

        [Fact]
        public void No_note_is_no_rating_and_dates_are_read_whole_or_by_year()
        {
            Assert.Null(ScrapeRules.Rating(null));
            Assert.Equal(new DateTime(1991, 8, 23), ScrapeRules.Date("1991-08-23"));
            Assert.Equal(new DateTime(1991, 1, 1), ScrapeRules.Date("1991"));
            Assert.Null(ScrapeRules.Date("sometime"));
        }

        [Fact]
        public void What_is_fetched_by_default_is_the_cover_screenshot_marquee_and_miximage()
        {
            Assert.Equal(["cover", "screenshot", "marquee", "miximage"], new ScrapeChoices().Kinds().Select(k => k.EsdeType));
            Assert.Equal(["box-2D"], ScrapeRules.Cover.ScreenScraperTypes);
            Assert.Equal(["mixrbv2"], ScrapeRules.Miximage.ScreenScraperTypes);
            Assert.Equal("miximages", ScrapeRules.Miximage.Folder);
        }
    }
}
