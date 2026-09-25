using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Media;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Every property the scene claims to map changes the rendered pixels: one synthetic theme per pair, rendered with two values - see EmuSen_BigPicture.md §13.5.
    public class SceneMappingTests
    {
        private readonly ITestOutputHelper _output;

        public SceneMappingTests(ITestOutputHelper output) => _output = output;

        private const int W = 320, H = 200;

        // What each element needs to be visible at all; a case's own property replaces its entry here.
        private static readonly Dictionary<string, string> Base = new()
        {
            ["image"] = "pos=0.1 0.1;size=0.5 0.6;path={A}",
            ["video"] = "pos=0.1 0.1;size=0.5 0.6;imageType=cover",
            ["text"] = "pos=0.05 0.05;size=0.9 0.6;text=Themed words here and more;fontSize=0.1;color=FFFFFF",
            ["datetime"] = "pos=0.05 0.05;size=0.9 0.6;metadata=releasedate;fontSize=0.1;color=FFFFFF",
            ["carousel"] = "pos=0 0;size=1 1;imageType=cover;maxItemCount=3;itemSize=0.3 0.6",
            ["textlist"] = "pos=0 0;size=1 1;fontSize=0.1;primaryColor=FFFFFF",
            ["rating"] = "pos=0.1 0.1;size=0 0.2;filledPath={S1};unfilledPath={S2}",
            ["badges"] = "pos=0 0;size=1 0.5;slots=favorite,completed;lines=1;itemsPerLine=4;itemMargin=0 0;customBadgeIcon:favorite=<customBadgeIcon badge=\"favorite\">{S1}</customBadgeIcon>;customBadgeIcon:completed=<customBadgeIcon badge=\"completed\">{S2}</customBadgeIcon>",
            ["helpsystem"] = "pos=0 0;fontSize=0.1;textColor=FFFFFF;iconColor=FFFFFF",
            ["clock"] = "pos=0 0;fontSize=0.1;color=FFFFFF",
            ["systemstatus"] = "pos=1 0;origin=1 0;height=0.15;entries=all",
        };

        private sealed record Case(string Type, string Property, string A, string B)
        {
            public string Context { get; init; } = "";
            public string Remove { get; init; } = "";
            public string? View { get; init; }
            public int Game { get; init; } = 4;
            public int System { get; init; } = 1;
            public bool HideMetadata { get; init; }
            public override string ToString() => $"{Type}.{Property}";
        }

        private static Case C(string type, string property, string a, string b) => new(type, property, a, b);


        // The two values of each pair, with the context that lets the property show; common properties are generated below.
        private static readonly Case[] Specific =
        [
            C("image", "path", "{A}", "{B}"),
            C("image", "default", "{A}", "{B}") with { Remove = "path", Context = "path={X}" },
            C("image", "imageType", "cover", "screenshot") with { Remove = "path" },
            C("image", "maxSize", "0.5 0.6", "0.2 0.2") with { Remove = "size" },
            C("image", "cropSize", "0.5 0.6", "0.2 0.6") with { Remove = "size" },
            C("image", "cropPos", "0 0", "1 1") with { Remove = "size", Context = "cropSize=0.2 0.6" },
            C("image", "tile", "false", "true"),
            C("image", "tileSize", "0.1 0.1", "0.2 0") with { Context = "tile=true" },
            C("image", "tileHorizontalAlignment", "left", "right") with { Context = "tile=true;tileSize=0.13 0.13" },
            C("image", "tileVerticalAlignment", "top", "bottom") with { Context = "tile=true;tileSize=0.13 0.13" },
            C("image", "color", "FFFFFF", "FF0000"),
            C("image", "colorEnd", "FFFFFF", "0000FF") with { Context = "color=FFFFFF" },
            C("image", "gradientType", "horizontal", "vertical") with { Context = "color=FF0000;colorEnd=0000FF" },
            C("image", "saturation", "1", "0"),
            C("image", "cornerRadius", "0", "0.05"),
            C("image", "interpolation", "nearest", "linear"),

            C("video", "imageType", "cover", "screenshot"),
            C("video", "defaultImage", "{A}", "{B}") with { Context = "imageType=none" },
            C("video", "imageSize", "0.5 0.6", "0.2 0.3"),
            C("video", "imageMaxSize", "0.5 0.6", "0.2 0.2") with { Remove = "size" },
            C("video", "imageCropSize", "0.5 0.6", "0.2 0.6") with { Remove = "size" },
            C("video", "imageCropPos", "0 0", "1 1") with { Remove = "size", Context = "imageCropSize=0.2 0.6" },
            C("video", "maxSize", "0.5 0.6", "0.2 0.2") with { Remove = "size" },
            C("video", "cropSize", "0.5 0.6", "0.2 0.6") with { Remove = "size" },
            C("video", "imageCornerRadius", "0", "0.05"),
            C("video", "color", "FFFFFF", "FF0000"),
            C("video", "colorEnd", "FFFFFF", "0000FF") with { Context = "color=FFFFFF" },
            C("video", "gradientType", "horizontal", "vertical") with { Context = "color=FF0000;colorEnd=0000FF" },
            C("video", "saturation", "1", "0"),

            C("text", "text", "One thing", "Another"),
            C("text", "metadata", "name", "developer") with { Remove = "text" },
            C("text", "systemdata", "name", "fullname") with { Remove = "text", View = "system" },
            C("text", "defaultValue", "one", "two") with { Remove = "text", Context = "metadata=emulator" },
            C("text", "systemNameSuffix", "false", "true") with { Remove = "text", Context = "metadata=name", System = 5 },
            C("text", "letterCaseSystemNameSuffix", "uppercase", "lowercase") with { Remove = "text", Context = "metadata=name;systemNameSuffix=true", System = 5 },
            C("text", "container", "false", "true") with { Context = "size=0.9 0.2;text=A long description that cannot fit in the small box it is given here" },
            C("text", "containerType", "vertical", "horizontal") with { Context = "container=true;size=0.9 0.3;text=A long description that cannot fit in the small box it is given here" },
            C("text", "fontPath", "{FT}", "{FB}"),
            C("text", "fontSize", "0.1", "0.05"),
            C("text", "horizontalAlignment", "left", "right"),
            C("text", "verticalAlignment", "top", "bottom"),
            C("text", "color", "FFFFFF", "FF0000"),
            C("text", "backgroundColor", "00000000", "FF0000"),
            C("text", "backgroundMargins", "0 0", "0.05 0.05") with { Context = "backgroundColor=FF0000;pos=0.2 0.2;size=0.5 0.3" },
            C("text", "backgroundCornerRadius", "0", "0.05") with { Context = "backgroundColor=FF0000" },
            C("text", "letterCase", "none", "uppercase"),
            C("text", "lineSpacing", "1", "2") with { Context = "verticalAlignment=top;text=A long description that wraps over several lines of the box" },

            C("datetime", "metadata", "releasedate", "lastplayed"),
            C("datetime", "defaultValue", "one", "two") with { Context = "metadata=lastplayed", Game = 0 },
            C("datetime", "fontPath", "{FT}", "{FB}"),
            C("datetime", "fontSize", "0.1", "0.05"),
            C("datetime", "horizontalAlignment", "left", "right"),
            C("datetime", "verticalAlignment", "top", "bottom"),
            C("datetime", "color", "FFFFFF", "FF0000"),
            C("datetime", "backgroundColor", "00000000", "FF0000"),
            C("datetime", "backgroundMargins", "0 0", "0.05 0.05") with { Context = "backgroundColor=FF0000;pos=0.2 0.2;size=0.5 0.3" },
            C("datetime", "backgroundCornerRadius", "0", "0.05") with { Context = "backgroundColor=FF0000" },
            C("datetime", "letterCase", "none", "uppercase") with { Context = "format=%b %Y" },
            C("datetime", "lineSpacing", "1", "2") with { Context = "verticalAlignment=top" },
            C("datetime", "format", "%Y-%m-%d", "%d/%m"),
            C("datetime", "displayRelative", "false", "true") with { Context = "metadata=lastplayed" },

            C("carousel", "type", "horizontal", "vertical"),
            C("carousel", "staticImage", "{A}", "{B}") with { View = "system", Remove = "imageType" },
            C("carousel", "defaultImage", "{A}", "{B}") with { View = "system", Remove = "imageType", Context = "staticImage={X}" },
            C("carousel", "imageType", "cover", "screenshot"),
            C("carousel", "maxItemCount", "3", "5"),
            C("carousel", "itemSize", "0.3 0.6", "0.2 0.3"),
            C("carousel", "itemScale", "1", "1.5"),
            C("carousel", "imageFit", "contain", "fill"),
            C("carousel", "imageColor", "FFFFFF", "FF0000"),
            C("carousel", "imageSelectedColor", "FFFFFF", "FF0000"),
            C("carousel", "imageSaturation", "1", "0"),
            C("carousel", "itemHorizontalAlignment", "left", "right") with { Context = "type=vertical;itemSize=0.3 0.2" },
            C("carousel", "itemVerticalAlignment", "top", "bottom") with { Context = "itemSize=0.3 0.3" },
            C("carousel", "unfocusedItemOpacity", "0.5", "1"),
            C("carousel", "unfocusedItemSaturation", "1", "0"),
            C("carousel", "unfocusedItemDimming", "1", "0.3"),
            C("carousel", "color", "00000000", "FF0000"),
            C("carousel", "colorEnd", "FF0000", "0000FF") with { Context = "color=FF0000" },
            C("carousel", "gradientType", "horizontal", "vertical") with { Context = "color=FF0000;colorEnd=0000FF" },
            C("carousel", "text", "One", "Two") with { View = "system", Remove = "imageType", Context = "staticImage={X};textColor=FFFFFF;fontSize=0.1" },
            C("carousel", "textRelativeScale", "1", "0.5") with { View = "system", Remove = "imageType", Context = "staticImage={X};textColor=FFFFFF;fontSize=0.1" },
            C("carousel", "textColor", "FFFFFF", "FF0000") with { View = "system", Remove = "imageType", Context = "staticImage={X};fontSize=0.1" },
            C("carousel", "textBackgroundColor", "FFFFFF00", "FF0000") with { View = "system", Remove = "imageType", Context = "staticImage={X};textColor=FFFFFF;fontSize=0.1" },
            C("carousel", "fontPath", "{FT}", "{FB}") with { View = "system", Remove = "imageType", Context = "staticImage={X};textColor=FFFFFF;fontSize=0.1" },
            C("carousel", "fontSize", "0.1", "0.05") with { View = "system", Remove = "imageType", Context = "staticImage={X};textColor=FFFFFF" },
            C("carousel", "letterCase", "none", "uppercase") with { View = "system", Remove = "imageType", Context = "staticImage={X};textColor=FFFFFF;fontSize=0.1" },

            C("textlist", "selectorHeight", "0.1", "0.2") with { Context = "selectorColor=FF0000" },
            C("textlist", "selectorVerticalOffset", "0", "0.1") with { Context = "selectorColor=FF0000" },
            C("textlist", "selectorColor", "333333", "FF0000"),
            C("textlist", "primaryColor", "FFFFFF", "FF0000"),
            C("textlist", "secondaryColor", "FFFFFF", "FF0000") with { System = 6 },
            C("textlist", "selectedColor", "FFFFFF", "FF0000"),
            C("textlist", "selectedSecondaryColor", "FFFFFF", "FF0000") with { System = 6, Game = 5 },
            C("textlist", "selectedBackgroundColor", "00000000", "FF0000"),
            C("textlist", "selectedBackgroundMargins", "0 0", "0.1 0.1") with { Context = "selectedBackgroundColor=FF0000;selectorColor=00000000;pos=0.2 0;size=0.5 1" },
            C("textlist", "selectedBackgroundCornerRadius", "0", "0.05") with { Context = "selectedBackgroundColor=FF0000;selectorColor=00000000" },
            C("textlist", "fontPath", "{FT}", "{FB}"),
            C("textlist", "fontSize", "0.1", "0.05"),
            C("textlist", "horizontalAlignment", "left", "right"),
            C("textlist", "horizontalMargin", "0", "0.1"),
            C("textlist", "letterCase", "none", "uppercase"),
            C("textlist", "lineSpacing", "1.5", "2.5"),
            C("textlist", "indicators", "none", "symbols"),
            C("textlist", "systemNameSuffix", "false", "true") with { System = 5 },
            C("textlist", "letterCaseSystemNameSuffix", "uppercase", "lowercase") with { System = 5, Context = "systemNameSuffix=true" },

            C("rating", "hideIfZero", "false", "true") with { Game = 3 },
            C("rating", "color", "FFFFFF", "FF0000"),
            C("rating", "filledPath", "{S1}", "{S3}"),
            C("rating", "unfilledPath", "{S2}", "{S3}"),
            C("rating", "overlay", "true", "false"),

            C("badges", "horizontalAlignment", "left", "right"),
            C("badges", "direction", "row", "column") with { Context = "lines=2;itemsPerLine=2" },
            C("badges", "lines", "1", "2"),
            C("badges", "itemsPerLine", "4", "2"),
            C("badges", "itemMargin", "0 0", "0.05 0.05"),
            C("badges", "slots", "favorite,completed", "favorite"),
            C("badges", "customBadgeIcon", "<customBadgeIcon badge=\"favorite\">{S1}</customBadgeIcon>", "<customBadgeIcon badge=\"favorite\">{S3}</customBadgeIcon>"),
            C("badges", "badgeIconColor", "FFFFFF", "FF0000"),

            C("helpsystem", "textColor", "FFFFFF", "FF0000"),
            C("helpsystem", "iconColor", "FFFFFF", "FF0000"),
            C("helpsystem", "fontPath", "{FT}", "{FB}"),
            C("helpsystem", "fontSize", "0.1", "0.05"),
            C("helpsystem", "entries", "all", "a,b"),
            C("helpsystem", "entryRelativeScale", "1", "0.5"),
            C("helpsystem", "entrySpacing", "0.01", "0.04"),
            C("helpsystem", "iconTextSpacing", "0", "0.04"),
            C("helpsystem", "letterCase", "uppercase", "lowercase"),
            C("helpsystem", "backgroundColor", "00000000", "FF0000"),
            C("helpsystem", "backgroundHorizontalPadding", "0 0", "0.05 0.05") with { Context = "backgroundColor=FF0000;entries=a" },
            C("helpsystem", "backgroundVerticalPadding", "0 0", "0.05 0.05") with { Context = "backgroundColor=FF0000" },
            C("helpsystem", "backgroundCornerRadius", "0", "0.05") with { Context = "backgroundColor=FF0000;backgroundHorizontalPadding=0.03 0.03;backgroundVerticalPadding=0.03 0.03" },
            C("helpsystem", "customButtonIcon", "<customButtonIcon button=\"button_a_XBOX\">{S1}</customButtonIcon>", "<customButtonIcon button=\"button_a_XBOX\">{S3}</customButtonIcon>") with { Context = "entries=a" },

            C("clock", "fontPath", "{FT}", "{FB}"),
            C("clock", "fontSize", "0.1", "0.05"),
            C("clock", "horizontalAlignment", "left", "right") with { Context = "size=0.8 0.3" },
            C("clock", "verticalAlignment", "top", "bottom") with { Context = "size=0.8 0.5" },
            C("clock", "color", "FFFFFF", "FF0000"),
            C("clock", "backgroundColor", "00000000", "FF0000"),
            C("clock", "backgroundColorEnd", "FF0000", "0000FF") with { Context = "backgroundColor=FF0000" },
            C("clock", "backgroundGradientType", "horizontal", "vertical") with { Context = "backgroundColor=FF0000;backgroundColorEnd=0000FF" },
            C("clock", "backgroundHorizontalPadding", "0 0", "0.05 0.05") with { Context = "backgroundColor=FF0000" },
            C("clock", "backgroundVerticalPadding", "0 0", "0.05 0.05") with { Context = "backgroundColor=FF0000" },
            C("clock", "backgroundCornerRadius", "0", "0.05") with { Context = "backgroundColor=FF0000;backgroundHorizontalPadding=0.03 0.03;backgroundVerticalPadding=0.03 0.03" },
            C("clock", "format", "%H:%M", "%Y"),

            C("systemstatus", "height", "0.1", "0.2"),
            C("systemstatus", "fontPath", "{FT}", "{FB}"),
            C("systemstatus", "textRelativeScale", "1", "0.5"),
            C("systemstatus", "color", "FFFFFF", "FF0000"),
            C("systemstatus", "backgroundColor", "00000000", "FF0000"),
            C("systemstatus", "backgroundHorizontalPadding", "0 0", "0.05 0.05") with { Context = "backgroundColor=FF0000" },
            C("systemstatus", "backgroundVerticalPadding", "0 0", "0.05 0.05") with { Context = "backgroundColor=FF0000" },
            C("systemstatus", "backgroundCornerRadius", "0", "0.05") with { Context = "backgroundColor=FF0000;backgroundHorizontalPadding=0.03 0.03;backgroundVerticalPadding=0.03 0.03" },
            C("systemstatus", "entries", "all", "wifi"),
            C("systemstatus", "entrySpacing", "0", "0.04"),
            C("systemstatus", "customIcon", "<customIcon icon=\"icon_wifi\">{S1}</customIcon>", "<customIcon icon=\"icon_wifi\">{S3}</customIcon>"),
        ];

        // The properties every element shares, where its type declares them.
        private static IEnumerable<Case> Common(string type)
        {
            ThemeElementSpec spec = ThemeCatalog.Find(type)!;
            bool has(string p) => spec.Find(p) is not null;
            string size = type is "rating" ? "0 0.3" : "0.3 0.3";
            if (has("pos")) yield return C(type, "pos", "0.1 0.1", "0.3 0.2");
            if (has("origin")) yield return C(type, "origin", "0 0", "1 1") with { Context = "pos=0.5 0.5" };
            if (has("size")) yield return C(type, "size", Base[type].Split(';').FirstOrDefault(p => p.StartsWith("size="))?[5..] ?? "0.5 0.5", size);
            if (has("rotation")) yield return C(type, "rotation", "0", "30");
            if (has("rotationOrigin")) yield return C(type, "rotationOrigin", "0 0", "1 1") with { Context = "rotation=30" };
            if (has("zIndex")) yield return C(type, "zIndex", "30", "45") with { Context = "cover" };
            if (has("opacity")) yield return C(type, "opacity", "1", "0.5");
            if (has("visible")) yield return C(type, "visible", "true", "false");
            if (has("metadataElement")) yield return C(type, "metadataElement", "false", "true") with { HideMetadata = true };
            if (has("scope")) yield return C(type, "scope", "shared", "none");
        }

        private static IEnumerable<Case> AllCases() => Specific.Concat(Base.Keys.SelectMany(Common));

        private static string Fill(string text) => text
            .Replace("{A}", SceneAssets.Halves("theme-a", 60, 80, Colors.DarkOrange, Colors.Teal))
            .Replace("{B}", SceneAssets.Halves("theme-b", 80, 50, Colors.Purple, Colors.Gold))
            .Replace("{X}", "/nonexistent/emusen/scene/missing.png")
            .Replace("{S1}", SceneAssets.Svg("icon-1", "<circle cx='10' cy='10' r='9' fill='#fff' fill-opacity='0.5'/>"))
            .Replace("{S2}", SceneAssets.Svg("icon-2", "<rect x='2' y='2' width='16' height='16' fill='#888'/>"))
            .Replace("{S3}", SceneAssets.Svg("icon-3", "<path d='M10 1L19 19H1Z' fill='#fff'/>"))
            .Replace("{FT}", SceneAssets.Font("Inter-Thin"))
            .Replace("{FB}", SceneAssets.Font("Inter-Bold"));

        private static string Element(Case c, string value)
        {
            var props = new List<KeyValuePair<string, string>>();
            void Set(string key, string v)
            {
                props.RemoveAll(p => p.Key == key);
                props.Add(new(key, v));
            }

            foreach (string part in Base[c.Type].Split(';')) { int eq = part.IndexOf('='); Set(part[..eq], part[(eq + 1)..]); }
            foreach (string r in c.Remove.Split(',', StringSplitOptions.RemoveEmptyEntries)) props.RemoveAll(p => p.Key == r);
            foreach (string part in c.Context.Split(';', StringSplitOptions.RemoveEmptyEntries).Where(p => p.Contains('='))) { int eq = part.IndexOf('='); Set(part[..eq], part[(eq + 1)..]); }
            if (value.StartsWith('<')) props.Add(new(c.Property + ":case", value));
            else Set(c.Property, value);
            string body = string.Concat(props.Select(p => p.Value.StartsWith('<') ? p.Value : $"<{p.Key}>{p.Value}</{p.Key}>"));
            string cover = c.Context.Split(';').Contains("cover") ? "<image name=\"cover\"><pos>0 0</pos><size>1 1</size><path>{B}</path><zIndex>35</zIndex></image>" : "";
            string view = c.View ?? (ThemeCatalog.Find(c.Type)!.Find(c.Property)?.OnlyIn == ThemeViewScope.System ? "system" : "gamelist");
            return Fill($"<view name=\"{view}\">{cover}<{c.Type} name=\"x\">{body}</{c.Type}></view>");
        }

        // Five regular systems, then a collection and a system whose sixth game is a folder.
        private static IReadOnlyList<SceneSystem> Systems(SyntheticTheme theme)
        {
            var systems = SyntheticLibrary.Systems.Select(s => (s.System, Games: SyntheticLibrary.Games(s.System, s.Extension))).ToList();
            systems.Add((new ThemeSystem("all", "All Games", "auto-allgames", ThemeSystemKind.AutoCollection), SyntheticLibrary.Games(new ThemeSystem("all", "All Games", "all"), ".nes")));
            systems.Add((new ThemeSystem("snes", "Super Nintendo", "snes"), SyntheticLibrary.Games(SyntheticTheme.Snes, ".sfc").Select((g, i) => i == 5 ? g with { Folder = true } : g).ToList()));
            return systems.Select(s => new SceneSystem(s.System, theme.Load(new ThemeChoices { ScreenWidth = W, ScreenHeight = H }, s.System), s.Games)).ToList();
        }

        private static (RenderedFrame Frame, ResolvedTheme Theme) Render(Case c, string value)
        {
            using var theme = new SyntheticTheme();
            theme.Capabilities("").Theme(Element(c, value));
            IReadOnlyList<SceneSystem> systems = Systems(theme);
            var data = new SceneData(systems, new Size(W, H))
            {
                SystemIndex = c.System, GameIndex = c.Game, Media = new SceneAssets.Media(), HideMetadata = c.HideMetadata,
                Status = new EmuSen.LunaP.Controls.DeviceStatus(Wifi: true, BatteryPercent: 70),
            };
            string view = Element(c, value).Contains("<view name=\"system\"") ? "system" : "gamelist";
            return (SceneAssets.Render(SceneBuilder.Build(data.System.Theme.View(view), data)), data.System.Theme);
        }

        [Fact]
        public Task Every_mapped_property_changes_the_rendered_pixels() => UiTest.Run(() =>
        {
            var failures = new List<string>();
            int cases = 0;
            foreach (Case c in AllCases())
            {
                cases++;
                (RenderedFrame a, ResolvedTheme ta) = Render(c, c.A);
                (RenderedFrame b, ResolvedTheme tb) = Render(c, c.B);
                string errors = string.Join("; ", ta.Errors.Concat(tb.Errors).Select(e => e.Message).Distinct());
                if (errors.Length > 0) failures.Add($"{c}: theme errors: {errors}");
                else if (SceneAssets.Differing(a, b) == 0) failures.Add($"{c}: {c.A} and {c.B} render the same pixels");
            }

            _output.WriteLine($"{cases} cases, {failures.Count} failing");
            Assert.True(failures.Count == 0, string.Join("\n", failures));
        });

        // The case list and the mapping's list are the same set, so the mapping cannot claim what no case proves.
        [Fact]
        public void Every_pair_the_mapping_names_has_a_case_and_every_case_is_mapped()
        {
            var claimed = SceneMapping.Specific.SelectMany(p => p.Value.Select(v => (p.Key, v)))
                .Concat(SceneMapping.Specific.Keys.SelectMany(t => SceneMapping.Common.Where(p => ThemeCatalog.Find(t)!.Find(p) is not null).Select(p => (t, p))))
                .ToHashSet();
            var proved = AllCases().Select(c => (c.Type, c.Property)).ToHashSet();
            Assert.Empty(claimed.Except(proved).Select(p => $"{p.Item1}.{p.Item2}"));
            Assert.Empty(proved.Except(claimed).Select(p => $"{p.Item1}.{p.Item2}"));
        }
    }
}
