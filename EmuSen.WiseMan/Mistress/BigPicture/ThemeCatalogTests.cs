using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Every documented element type and property, set, typed, defaulted and clamped, on synthetic themes - see EmuSen_BigPicture.md §12.2.
    public class ThemeCatalogTests : IDisposable
    {
        private readonly SyntheticTheme _theme = new();

        public void Dispose() => _theme.Dispose();

        [Fact]
        public void The_catalogue_holds_THEMES_md_s_fifteen_reference_elements_and_sound_with_468_properties()
        {
            var expected = new Dictionary<string, int>
            {
                ["carousel"] = 71, ["grid"] = 64, ["textlist"] = 38, ["image"] = 35, ["video"] = 41, ["animation"] = 22,
                ["badges"] = 32, ["text"] = 35, ["datetime"] = 25, ["gamelistinfo"] = 15, ["rating"] = 17,
                ["gameselector"] = 3, ["helpsystem"] = 31, ["systemstatus"] = 19, ["clock"] = 19, ["sound"] = 1,
            };
            Assert.Equal(expected, ThemeCatalog.Elements.ToDictionary(e => e.Type, e => e.Properties.Count));
            Assert.Equal(468, ThemeCatalog.PropertyCount);
        }

        // Spot checks read from THEMES.md by hand, independent of how the catalogue was drafted.
        [Theory]
        [InlineData("carousel", "pos", "NormalizedPair", "0 0.38378", null, null)]
        [InlineData("carousel", "size", "NormalizedPair", "1 0.2324", "0.05", "2")]
        [InlineData("carousel", "maxItemCount", "Float", "3", "0.5", "30")]
        [InlineData("carousel", "itemScale", "Float", "1.2", "0.2", "3")]
        [InlineData("carousel", "color", "Color", "FFFFFFD8", null, null)]
        [InlineData("carousel", "unfocusedItemOpacity", "Float", "0.5", "0.1", "1")]
        [InlineData("carousel", "fontSize", "Float", "0.085", "0.001", "1.5")]
        [InlineData("grid", "itemSize", "NormalizedPair", "0.15 0.25", "0.05", "1")]
        [InlineData("grid", "itemScale", "Float", "1.05", "0.5", "2")]
        [InlineData("grid", "unfocusedItemOpacity", "Float", "1", "0.1", "1")]
        [InlineData("textlist", "selectorColor", "Color", "333333FF", null, null)]
        [InlineData("textlist", "primaryColor", "Color", "0000FFFF", null, null)]
        [InlineData("textlist", "secondaryColor", "Color", "00FF00FF", null, null)]
        [InlineData("textlist", "lineSpacing", "Float", "1.5", "0.5", "3")]
        [InlineData("image", "zIndex", "Float", "30", null, null)]
        [InlineData("image", "size", "NormalizedPair", null, "0.001", "3")]
        [InlineData("image", "cornerRadius", "Float", "0", "0", "0.5")]
        [InlineData("video", "delay", "Float", "1.5", "0", "15")]
        [InlineData("video", "pillarboxThreshold", "NormalizedPair", "0.85 0.90", "0.2", "1")]
        [InlineData("video", "fadeInTime", "Float", "1", "0", "8")]
        [InlineData("animation", "speed", "Float", "1", "0.2", "3")]
        [InlineData("animation", "zIndex", "Float", "35", null, null)]
        [InlineData("badges", "lines", "UnsignedInteger", "3", null, null)]
        [InlineData("badges", "itemsPerLine", "UnsignedInteger", "4", null, null)]
        [InlineData("badges", "itemMargin", "NormalizedPair", "0.01 0.01", "0", "0.2")]
        [InlineData("text", "color", "Color", "000000FF", null, null)]
        [InlineData("text", "containerResetDelay", "Float", "7", "0", "20")]
        [InlineData("datetime", "format", "String", "%Y-%m-%d", null, null)]
        [InlineData("gamelistinfo", "zIndex", "Float", "45", null, null)]
        [InlineData("rating", "size", "NormalizedPair", "0 0.06", "0.01", "1")]
        [InlineData("gameselector", "gameCount", "UnsignedInteger", "1", "1", "30")]
        [InlineData("helpsystem", "textColor", "Color", "777777FF", null, null)]
        [InlineData("helpsystem", "entrySpacing", "Float", "0.00833", "0", "0.04")]
        [InlineData("helpsystem", "opacity", "Float", "1", "0.2", "1")]
        [InlineData("systemstatus", "pos", "NormalizedPair", "0.982 0.016", null, null)]
        [InlineData("systemstatus", "origin", "NormalizedPair", "1 0", "0", "1")]
        [InlineData("systemstatus", "height", "Float", "0.035", "0.01", "0.5")]
        [InlineData("clock", "format", "String", "%H:%M", null, null)]
        [InlineData("clock", "pos", "NormalizedPair", "0.018 0.016", null, null)]
        public void Catalogue_entries_match_THEMES_md(string element, string property, string type, string? def, string? min, string? max)
        {
            ThemePropertySpec spec = ThemeCatalog.Find(element)!.Find(property)!;
            Assert.Equal(type, spec.Type.ToString());
            Assert.Equal(def, spec.Default);
            Assert.Equal(min is null ? null : float.Parse(min, CultureInfo.InvariantCulture), spec.Min);
            Assert.Equal(max is null ? null : float.Parse(max, CultureInfo.InvariantCulture), spec.Max);
        }

        [Fact]
        public void Views_groups_and_default_zIndex_match_THEMES_md()
        {
            Assert.Equal(["carousel", "grid", "textlist"], ThemeCatalog.Elements.Where(e => e.Group == ThemeElementGroup.Primary).Select(e => e.Type));
            Assert.Equal(["badges", "gamelistinfo"], ThemeCatalog.Elements.Where(e => e.Views == ThemeElementViews.Gamelist).Select(e => e.Type));
            Assert.Equal(["gameselector"], ThemeCatalog.Elements.Where(e => e.Views == ThemeElementViews.System).Select(e => e.Type));
            var z = ThemeCatalog.Elements.ToDictionary(e => e.Type, e => e.DefaultZIndex);
            Assert.Equal((30f, 30f, 35f, 35f, 40f, 40f, 45f, 45f, 50f, 50f, 50f),
                (z["image"], z["video"], z["animation"], z["badges"], z["text"], z["datetime"], z["gamelistinfo"], z["rating"], z["carousel"], z["grid"], z["textlist"]));
            Assert.Null(z["helpsystem"]);
            Assert.Null(z["clock"]);
            Assert.Null(z["systemstatus"]);
        }

        private static string Sample(ThemePropertySpec spec, string element, out ThemeValue expected, SyntheticTheme theme)
        {
            float Mid(float fraction) => spec.Min is { } lo && spec.Max is { } hi ? lo + (hi - lo) * fraction : fraction;
            string F(float v) => v.ToString("0.####", CultureInfo.InvariantCulture);
            switch (spec.Type)
            {
                case ThemePropertyType.NormalizedPair:
                    var pair = spec.Rule == PairRule.OneAxisOnly ? new NormalizedPair(0, 0.1f) : new NormalizedPair(float.Parse(F(Mid(0.25f)), CultureInfo.InvariantCulture), float.Parse(F(Mid(0.75f)), CultureInfo.InvariantCulture));
                    expected = new PairValue(pair);
                    return $"{F(pair.X)} {F(pair.Y)}";
                case ThemePropertyType.Path when spec.Shape != ThemeValueShape.Keyed:
                    string relative = $"assets/{element}-{spec.Name}.png";
                    theme.Asset(relative);
                    expected = new PathValue(new ThemePath("./" + relative, theme.PathOf(relative), false, true));
                    return "./" + relative;
                case ThemePropertyType.Boolean:
                    bool b = spec.Default != "true";
                    expected = new BoolValue(b);
                    return b ? "true" : "false";
                case ThemePropertyType.Color:
                    expected = new ColorValue(new ThemeColor(0x12345678));
                    return "12345678";
                case ThemePropertyType.UnsignedInteger:
                    uint u = spec.Min is { } min ? (uint)min + 1 : 2;
                    expected = new UIntValue(u);
                    return u.ToString(CultureInfo.InvariantCulture);
                case ThemePropertyType.Float:
                    float f = float.Parse(F(Mid(0.25f)), CultureInfo.InvariantCulture);
                    expected = new FloatValue(f);
                    return F(f);
            }
            switch (spec.Shape)
            {
                case ThemeValueShape.Enum:
                    string value = spec.Values.Last(v => v != spec.Default);
                    expected = new StringValue(value);
                    return value;
                case ThemeValueShape.List:
                    string item = spec.Values.Contains("cover") ? "cover" : spec.Values[0];
                    expected = new ListValue([item]);
                    return item;
                case ThemeValueShape.Keyed:
                    string key = spec.Values[0];
                    string keyed = $"assets/{element}-{spec.Name}-{key}.svg";
                    theme.Asset(keyed);
                    expected = new KeyedPathsValue(new Dictionary<string, ThemePath> { [key] = new ThemePath("./" + keyed, theme.PathOf(keyed), false, true) });
                    return "./" + keyed;
                default:
                    expected = new StringValue("sample text");
                    return "sample text";
            }
        }

        [Fact]
        public void Every_documented_property_of_every_element_is_read_into_its_type()
        {
            var expectations = new List<(string View, string Element, string Property, ThemeValue Value)>();
            var views = new Dictionary<string, StringBuilder> { ["system"] = new(), ["gamelist"] = new(), ["all"] = new() };

            foreach (ThemeElementSpec element in ThemeCatalog.Elements)
            {
                string[] targets = element.Views == ThemeElementViews.All ? ["all"]
                    : element.Views == ThemeElementViews.SystemAndGamelist ? ["system", "gamelist"]
                    : [element.Views == ThemeElementViews.System ? "system" : "gamelist"];
                var byView = targets.ToDictionary(v => v, _ => new StringBuilder());
                foreach (ThemePropertySpec property in element.Properties)
                {
                    string view = property.OnlyIn switch
                    {
                        ThemeViewScope.System => "system",
                        ThemeViewScope.Gamelist => "gamelist",
                        _ => targets.Contains("gamelist") ? "gamelist" : targets[0],
                    };
                    string text = Sample(property, element.Type, out ThemeValue value, _theme);
                    string key = property.KeyAttribute is { } attribute ? $" {attribute}=\"{property.Values[0]}\"" : "";
                    byView[view].Append($"<{property.Name}{key}>{text}</{property.Name}>");
                    expectations.Add((view, element.Type, property.Name, value));
                }
                foreach ((string view, StringBuilder props) in byView)
                    views[view].Append($"<{element.Type} name=\"every\">{props}</{element.Type}>");
            }

            // One primary per view, so each primary type gets a view of its own through a variant.
            _theme.Capabilities("<variant name=\"v\"/>").Theme(string.Join("\n", views.Select(v => $"<view name=\"{v.Key}\">{v.Value}</view>")));
            string[] primaries = ["carousel", "grid", "textlist"];
            var seenPairs = new HashSet<(string, string)>();
            foreach (string keep in primaries)
            {
                string text = string.Join("\n", views.Select(v => $"<view name=\"{v.Key}\">{StripOtherPrimaries(v.Value.ToString(), keep, primaries)}</view>"));
                _theme.Theme(text);
                ResolvedTheme theme = _theme.Load();
                Assert.True(theme.IsThemed, string.Join("\n", theme.Diagnostics));
                Assert.DoesNotContain(theme.Diagnostics, d => d.Severity != ThemeSeverity.Debug);

                foreach ((string view, string element, string property, ThemeValue value) in expectations.Where(e => !primaries.Contains(e.Element) || e.Element == keep))
                {
                    ResolvedElement? resolved = view == "all" ? theme.SoundElements.Single() : theme.View(view).Find(element, "every");
                    Assert.True(resolved is not null, $"{element} missing from the {view} view");
                    Assert.True(resolved!.Explicit.TryGetValue(property, out ThemeValue? actual), $"{element}.{property} was not read");
                    Assert.Equal(value, actual);
                    seenPairs.Add((element, property));
                }
            }
            Assert.Equal(468, seenPairs.Count);
        }

        private static string StripOtherPrimaries(string xml, string keep, string[] primaries)
        {
            foreach (string other in primaries.Where(p => p != keep))
            {
                int start;
                while ((start = xml.IndexOf($"<{other} name=\"every\">", StringComparison.Ordinal)) >= 0)
                {
                    string close = $"</{other}>";
                    int end = xml.IndexOf(close, start, StringComparison.Ordinal) + close.Length;
                    xml = xml.Remove(start, end - start);
                }
            }
            return xml;
        }

        [Fact]
        public void Every_literal_default_is_in_the_effective_values_of_an_element_that_sets_nothing()
        {
            var covered = new HashSet<(string, string)>();
            foreach (string primary in new[] { "textlist", "carousel", "grid" })
            {
                string Bare(ThemeElementViews view) => string.Concat(ThemeCatalog.Elements
                    .Where(e => e.Views.HasFlag(view) && (e.Group != ThemeElementGroup.Primary || e.Type == primary))
                    .Select(e => $"<{e.Type} name=\"bare\"/>"));
                _theme.Capabilities("").Theme($"<view name=\"gamelist\">{Bare(ThemeElementViews.Gamelist)}</view><view name=\"system\">{Bare(ThemeElementViews.System)}</view><view name=\"all\">{Bare(ThemeElementViews.All)}</view>");
                ResolvedTheme theme = _theme.Load();
                Assert.True(theme.IsThemed, string.Join("\n", theme.Diagnostics));

                foreach ((ResolvedElement element, string view) in theme.GamelistView.Elements.Select(e => (e, "gamelist")).Concat(theme.SystemView.Elements.Select(e => (e, "system"))))
                    foreach (ThemePropertySpec spec in element.Spec.Properties.Where(p => p.Default is not null && (p.OnlyIn == ThemeViewScope.Any || p.OnlyIn.ToString().ToLowerInvariant() == view)))
                    {
                        ThemeValue? value = element.Get(spec.Name);
                        Assert.True(value is not null, $"{element.Type}.{spec.Name} has no default value");
                        string shown = value is ListValue l ? string.Join(" ", l.Items) : value!.ToString()!;
                        Assert.Equal(Normalise(spec.Default!, spec.Type), shown);
                        covered.Add((element.Type, spec.Name));
                    }
            }
            var literal = ThemeCatalog.Elements.SelectMany(e => e.Properties.Where(p => p.Default is not null).Select(p => (e.Type, p.Name))).ToHashSet();
            literal.ExceptWith(covered);
            Assert.Empty(literal);
            Assert.Equal(336, covered.Count);
        }

        private static string Normalise(string text, ThemePropertyType type) => type switch
        {
            ThemePropertyType.Color => text.Length == 6 ? text.ToUpperInvariant() + "FF" : text.ToUpperInvariant(),
            ThemePropertyType.Float => float.Parse(text, CultureInfo.InvariantCulture).ToString("0.######", CultureInfo.InvariantCulture),
            ThemePropertyType.NormalizedPair => new NormalizedPair(float.Parse(text.Split(' ')[0], CultureInfo.InvariantCulture), float.Parse(text.Split(' ')[1], CultureInfo.InvariantCulture)).ToString(),
            _ => text,
        };

        [Fact]
        public void Defaults_stated_as_another_property_follow_that_property()
        {
            _theme.Capabilities("").Theme("""
                <view name="gamelist">
                    <textlist name="l"><primaryColor>112233</primaryColor><selectedBackgroundColor>44556677</selectedBackgroundColor></textlist>
                    <image name="i"><color>AABBCC</color></image>
                    <helpsystem name="h"><pos>0.3 0.4</pos><opacity>0.5</opacity></helpsystem>
                    <video name="v"><maxSize>0.5 0.5</maxSize></video>
                </view>
                """);
            ResolvedView view = _theme.Load().GamelistView;
            ResolvedElement list = view.Find("textlist", "l")!;
            Assert.Equal("112233FF", list.Color("selectedColor")!.Value.ToString());
            Assert.Equal("112233FF", list.Color("selectedSecondaryColor")!.Value.ToString());
            Assert.Equal("44556677", list.Color("selectedSecondaryBackgroundColor")!.Value.ToString());
            Assert.Equal("AABBCCFF", view.Find("image", "i")!.Color("colorEnd")!.Value.ToString());
            Assert.Equal(new NormalizedPair(0.3f, 0.4f), view.Find("helpsystem", "h")!.Pair("posDimmed"));
            Assert.Equal(0.5f, view.Find("helpsystem", "h")!.Float("opacityDimmed"));
            Assert.Equal(new NormalizedPair(0.5f, 0.5f), view.Find("video", "v")!.Pair("imageMaxSize"));
        }

        [Fact]
        public void Defaults_stated_in_words_are_computed()
        {
            _theme.Capabilities("<aspectRatio>16:9</aspectRatio><aspectRatio>16:9_vertical</aspectRatio>").Theme("""
                <view name="system"><carousel name="c"><pos>0 0</pos></carousel></view>
                <view name="gamelist">
                    <textlist name="l"><size>0.4 0.8</size><fontSize>0.04</fontSize></textlist>
                    <helpsystem name="h"><scope>view</scope></helpsystem>
                    <image name="straight"><rotation>-90</rotation></image>
                    <image name="tilted"><rotation>45</rotation></image>
                    <text name="description"><metadata>description</metadata></text>
                    <text name="name"><metadata>name</metadata></text>
                    <text name="scroller"><container>true</container><containerType>horizontal</containerType></text>
                    <datetime name="last"><metadata>lastplayed</metadata></datetime>
                    <datetime name="released"><metadata>releasedate</metadata></datetime>
                </view>
                """);
            ResolvedTheme horizontal = _theme.Load(new ThemeChoices { AspectRatio = "16:9" });
            ResolvedView g = horizontal.GamelistView;
            Assert.Equal("Super Nintendo", horizontal.SystemView.Find("carousel", "c")!.String("text"));
            Assert.Equal(0.4f, g.Find("textlist", "l")!.Float("selectorWidth"));
            Assert.Equal(0.06f, g.Find("textlist", "l")!.Float("selectorHeight")!.Value, 5);
            Assert.Equal(new NormalizedPair(0.012f, 0.9515f), g.Find("helpsystem", "h")!.Pair("pos"));
            Assert.Equal(0.035f, g.Find("helpsystem", "h")!.Float("fontSize"));
            Assert.Equal("nearest", g.Find("image", "straight")!.String("interpolation"));
            Assert.Equal("linear", g.Find("image", "tilted")!.String("interpolation"));
            Assert.True(g.Find("text", "description")!.Bool("container"));
            Assert.False(g.Find("text", "name")!.Bool("container"));
            Assert.Equal(4.5f, g.Find("text", "description")!.Float("containerStartDelay"));
            Assert.Equal(1.5f, g.Find("text", "scroller")!.Float("containerStartDelay"));
            Assert.True(g.Find("datetime", "last")!.Bool("displayRelative"));
            Assert.False(g.Find("datetime", "released")!.Bool("displayRelative"));

            ResolvedView vertical = _theme.Load(new ThemeChoices { AspectRatio = "16:9_vertical" }).GamelistView;
            Assert.Equal(new NormalizedPair(0.012f, 0.975f), vertical.Find("helpsystem", "h")!.Pair("pos"));
            Assert.Equal(0.025f, vertical.Find("helpsystem", "h")!.Float("fontSize"));
        }

        [Fact]
        public void Colours_of_six_digits_are_opaque_and_of_eight_carry_their_alpha()
        {
            _theme.Capabilities("").Theme("<view name=\"gamelist\"><text name=\"a\"><color>0a0B0c</color></text><text name=\"b\"><color>0A0B0C80</color></text></view>");
            ResolvedView view = _theme.Load().GamelistView;
            ThemeColor a = view.Find("text", "a")!.Color("color")!.Value;
            ThemeColor b = view.Find("text", "b")!.Color("color")!.Value;
            Assert.Equal((10, 11, 12, 255), ((int)a.R, (int)a.G, (int)a.B, (int)a.A));
            Assert.Equal(0x80, b.A);
        }

        [Fact]
        public void Booleans_accept_true_false_one_and_zero()
        {
            _theme.Capabilities("").Theme("<view name=\"gamelist\"><image name=\"a\"><tile>1</tile><mipmap>TRUE</mipmap><visible>0</visible><flipVertical>false</flipVertical></image></view>");
            ResolvedElement a = _theme.Load().GamelistView.Find("image", "a")!;
            Assert.Equal((true, true, false, false), (a.Bool("tile"), a.Bool("mipmap"), a.Bool("visible"), a.Bool("flipVertical")));
        }

        [Fact]
        public void Numbers_outside_their_range_are_clamped_silently()
        {
            _theme.Capabilities("").Theme("""
                <view name="gamelist">
                    <carousel name="c"><maxItemCount>99</maxItemCount><itemScale>0.01</itemScale><size>5 0.001</size><itemsBeforeCenter>40</itemsBeforeCenter></carousel>
                    <image name="i"><opacity>1.5</opacity><brightness>-3</brightness><origin>-1 2</origin><pos>-0.5 1.5</pos></image>
                    <rating name="r"><size>0 0.9</size></rating>
                    <rating name="rx"><size>3 0</size></rating>
                </view>
                """);
            ResolvedTheme theme = _theme.Load();
            Assert.True(theme.IsThemed);
            Assert.DoesNotContain(theme.Diagnostics, d => d.Severity != ThemeSeverity.Debug);
            ResolvedElement c = theme.GamelistView.Find("carousel", "c")!;
            Assert.Equal(30f, c.Float("maxItemCount"));
            Assert.Equal(0.2f, c.Float("itemScale"));
            Assert.Equal(new NormalizedPair(2, 0.05f), c.Pair("size"));
            Assert.Equal(20u, c.UInt("itemsBeforeCenter"));
            ResolvedElement i = theme.GamelistView.Find("image", "i")!;
            Assert.Equal((1f, -2f), (i.Float("opacity")!.Value, i.Float("brightness")!.Value));
            Assert.Equal(new NormalizedPair(0, 1), i.Pair("origin"));
            Assert.Equal(new NormalizedPair(-0.5f, 1.5f), i.Pair("pos"));
            Assert.Equal(new NormalizedPair(0, 0.5f), theme.GamelistView.Find("rating", "r")!.Pair("size"));
            Assert.Equal(new NormalizedPair(1, 0), theme.GamelistView.Find("rating", "rx")!.Pair("size"));
        }

        [Fact]
        public void Sizes_with_a_zero_axis_are_automatic_and_both_zero_is_clamped_to_the_minimum()
        {
            _theme.Capabilities("").Theme("""
                <view name="gamelist">
                    <image name="auto"><size>0 0.5</size></image>
                    <image name="none"><size>0 0</size></image>
                    <video name="v"><size>0 0</size></video>
                    <text name="t"><size>0 0</size></text>
                </view>
                """);
            ResolvedView view = _theme.Load().GamelistView;
            Assert.Equal(new NormalizedPair(0, 0.5f), view.Find("image", "auto")!.Pair("size"));
            Assert.Equal(new NormalizedPair(0.001f, 0.001f), view.Find("image", "none")!.Pair("size"));
            Assert.Equal(new NormalizedPair(0.01f, 0.01f), view.Find("video", "v")!.Pair("size"));
            Assert.Equal(new NormalizedPair(0, 0), view.Find("text", "t")!.Pair("size"));
        }

        [Fact]
        public void Minus_one_means_match_the_other_axis_where_THEMES_md_allows_it()
        {
            _theme.Capabilities("").Theme("""
                <view name="gamelist">
                    <grid name="g"><itemSize>-1 0.2</itemSize><itemSpacing>0.02 -1</itemSpacing></grid>
                    <badges name="b"><itemMargin>-1 0.05</itemMargin></badges>
                </view>
                """);
            ResolvedView view = _theme.Load().GamelistView;
            Assert.Equal(new NormalizedPair(-1, 0.2f), view.Find("grid", "g")!.Pair("itemSize"));
            Assert.Equal(new NormalizedPair(0.02f, -1), view.Find("grid", "g")!.Pair("itemSpacing"));
            Assert.Equal(new NormalizedPair(-1, 0.05f), view.Find("badges", "b")!.Pair("itemMargin"));
        }

        [Fact]
        public void Lists_split_on_commas_and_whitespace_and_the_image_shortcut_binds_four_media()
        {
            _theme.Capabilities("").Theme("""
                <view name="gamelist">
                    <badges name="b"><slots>favorite,
                        completed	kidgame , broken</slots></badges>
                    <image name="i"><imageType>image</imageType></image>
                    <video name="v"><imageType>none</imageType></video>
                </view>
                """);
            ResolvedView view = _theme.Load().GamelistView;
            Assert.Equal(["favorite", "completed", "kidgame", "broken"], view.Find("badges", "b")!.List("slots"));
            Assert.Equal(new ElementBinding("media", ["miximage", "screenshot", "titlescreen", "cover"]), view.Find("image", "i")!.Bindings.Single(), new BindingComparer());
            Assert.Empty(view.Find("video", "v")!.Bindings.Single().Names);
        }

        private sealed class BindingComparer : IEqualityComparer<ElementBinding>
        {
            public bool Equals(ElementBinding? x, ElementBinding? y) => x!.Kind == y!.Kind && x.Names.SequenceEqual(y.Names);
            public int GetHashCode(ElementBinding obj) => obj.Kind.GetHashCode();
        }

        [Fact]
        public void Media_and_data_the_theme_does_not_supply_are_named_bindings()
        {
            _theme.Capabilities("").Theme("""
                <view name="system">
                    <carousel name="c"><pos>0 0</pos></carousel>
                    <text name="count"><systemdata>gamecount</systemdata></text>
                    <gameselector name="g"><selection>lastplayed</selection></gameselector>
                </view>
                <view name="gamelist">
                    <textlist name="l"><pos>0 0</pos></textlist>
                    <video name="art"><imageType>cover, screenshot</imageType></video>
                    <text name="d"><metadata>description</metadata></text>
                    <datetime name="r"><metadata>releasedate</metadata></datetime>
                    <rating name="s"><pos>0 0</pos></rating>
                    <badges name="b"><slots>favorite</slots></badges>
                    <gamelistinfo name="gi"><pos>0 0</pos></gamelistinfo>
                    <helpsystem name="h"><entries>a b</entries></helpsystem>
                    <clock name="k"><pos>0 0</pos></clock>
                    <systemstatus name="st"><entries>battery</entries></systemstatus>
                </view>
                """);
            ResolvedTheme theme = _theme.Load();
            string Show(ResolvedView v, string type, string name) => string.Join(" ", v.Find(type, name)!.Bindings);
            Assert.Equal("systems", Show(theme.SystemView, "carousel", "c"));
            Assert.Equal("systemdata:gamecount", Show(theme.SystemView, "text", "count"));
            Assert.Equal("gameselector:lastplayed", Show(theme.SystemView, "gameselector", "g"));
            Assert.Equal("games", Show(theme.GamelistView, "textlist", "l"));
            Assert.Equal("videoFallbackImage:cover,screenshot", Show(theme.GamelistView, "video", "art"));
            Assert.Equal("metadata:description", Show(theme.GamelistView, "text", "d"));
            Assert.Equal("metadata:releasedate", Show(theme.GamelistView, "datetime", "r"));
            Assert.Equal("metadata:rating", Show(theme.GamelistView, "rating", "s"));
            Assert.Equal("badges:favorite", Show(theme.GamelistView, "badges", "b"));
            Assert.Equal("gamelistinfo", Show(theme.GamelistView, "gamelistinfo", "gi"));
            Assert.Equal("help:a,b", Show(theme.GamelistView, "helpsystem", "h"));
            Assert.Equal("clock", Show(theme.GamelistView, "clock", "k"));
            Assert.Equal("systemstatus:battery", Show(theme.GamelistView, "systemstatus", "st"));
        }

        [Fact]
        public void Only_video_playback_properties_are_marked_unsupported()
        {
            var deferred = ThemeCatalog.Elements.SelectMany(e => e.Properties.Where(p => p.Deferred is not null).Select(p => $"{e.Type}.{p.Name}")).ToArray();
            Assert.Equal(["video.path", "video.default", "video.iterationCount", "video.onIterationsDone", "video.audio", "video.videoCornerRadius",
                "video.pillarboxes", "video.pillarboxThreshold", "video.scanlines", "video.delay", "video.fadeInType", "video.fadeInTime"], deferred);
        }
    }
}
