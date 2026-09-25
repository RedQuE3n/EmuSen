using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // rating, badges, helpsystem, clock and systemstatus as LunaP's indicators - see EmuSen_BigPicture.md §13.2.
    internal static class IndicatorElements
    {
        internal static readonly string[] BadgeSlots = ["collection", "folder", "favorite", "completed", "kidgame", "broken", "controller", "altemulator", "manual"];

        internal static Control? Rating(SceneBuilder b, ResolvedElement e)
        {
            float value = b.Data.Game?.Rating ?? 0;
            if (value <= 0 && e.Bool("hideIfZero") == true) return null;
            var rating = new StarRating
            {
                Value = value,
                FilledPath = ImageElements.Existing(e.Path("filledPath")),
                UnfilledPath = ImageElements.Existing(e.Path("unfilledPath")),
                Tint = SceneUnits.ToColor(e.Color("color"), Colors.White),
                Overlay = e.Bool("overlay") ?? true,
            };
            NormalizedCanvas.SetSize(rating, SceneUnits.ToSize(e.Pair("size") ?? new NormalizedPair(0, 0.06f)));
            return rating;
        }

        internal static bool Has(SceneGame game, string slot) => slot switch
        {
            "collection" => game.InCollection, "folder" => game.Folder, "favorite" => game.Favorite, "completed" => game.Completed,
            "kidgame" => game.KidGame, "broken" => game.Broken, "altemulator" => game.AltEmulator, _ => false,
        };

        internal static Control? Badges(SceneBuilder b, ResolvedElement e)
        {
            if (b.Data.Game is not { } game) return null;
            IReadOnlyList<string> slots = e.List("slots") is { Count: > 0 } s ? (s.Contains("all") ? BadgeSlots : s) : [];
            IReadOnlyDictionary<string, ThemePath> icons = e.Keyed("customBadgeIcon");
            string[] shown = slots.Where(slot => Has(game, slot) && icons.GetValueOrDefault(slot) is { Exists: true }).Select(slot => icons[slot].Absolute).ToArray();
            NormalizedPair margin = e.Pair("itemMargin") ?? new NormalizedPair(0.01f, 0.01f);
            double mx = margin.X < 0 ? SceneUnits.Px(margin.Y, b.H) : SceneUnits.Px(margin.X, b.W), my = margin.Y < 0 ? mx : SceneUnits.Px(margin.Y, b.H);
            var badges = new BadgeStrip
            {
                Icons = shown,
                Direction = e.String("direction") == "column" ? Avalonia.Layout.Orientation.Vertical : Avalonia.Layout.Orientation.Horizontal,
                Lines = (int)(e.UInt("lines") ?? 3),
                ItemsPerLine = (int)(e.UInt("itemsPerLine") ?? 4),
                ItemMargin = new Size(mx, my),
                ContentHorizontalAlignment = SceneUnits.HorizontalLayout(e.String("horizontalAlignment")),
                Tint = SceneUnits.ToColor(e.Color("badgeIconColor"), Colors.White),
            };
            NormalizedCanvas.SetSize(badges, SceneUnits.ToSize(e.Pair("size") ?? new NormalizedPair(0.15f, 0.2f)));
            return badges;
        }

        internal static Control? Help(SceneBuilder b, ResolvedElement e)
        {
            IReadOnlyList<string> entries = e.List("entries") is { Count: > 0 } l ? l : ["all"];
            IReadOnlyDictionary<string, ThemePath> icons = e.Keyed("customButtonIcon");
            IReadOnlyList<HintEntry> hints = HelpPrompts.For(b.View.Name, entries, icons);
            if (hints.Count == 0) return null;
            return Outward(new HintBar
            {
                Entries = hints,
                FontPath = ImageElements.Existing(e.Path("fontPath")),
                FontSize = SceneUnits.Px(e.Float("fontSize") ?? 0.035f, b.H),
                EntryScale = e.Float("entryRelativeScale") ?? 1,
                IconColor = SceneUnits.ToColor(e.Color("iconColor"), Color.FromRgb(0x77, 0x77, 0x77)),
                TextColor = SceneUnits.ToColor(e.Color("textColor"), Color.FromRgb(0x77, 0x77, 0x77)),
                BackgroundColor = SceneUnits.ToColor(e.Color("backgroundColor"), Colors.Transparent),
                BackgroundCornerRadius = SceneUnits.Px(e.Float("backgroundCornerRadius") ?? 0, b.W),
                Padding = Padding(b, e),
                EntrySpacing = SceneUnits.Px(e.Float("entrySpacing") ?? 0.00833f, b.W),
                IconTextSpacing = SceneUnits.Px(e.Float("iconTextSpacing") ?? 0.00416f, b.W),
                LetterCase = SceneUnits.Case(e.String("letterCase") ?? "uppercase"),
            });
        }

        internal static Control? Clock(SceneBuilder b, ResolvedElement e)
        {
            var clock = new ClockLabel
            {
                Time = b.Data.Now,
                Format = SceneUnits.DotNetFormat(e.String("format") ?? "%H:%M"),
                FontPath = ImageElements.Existing(e.Path("fontPath")),
                FontSize = SceneUnits.Px(e.Float("fontSize") ?? 0.035f, b.H),
                Foreground = new SolidColorBrush(SceneUnits.ToColor(e.Color("color"), Colors.White)),
                Background = SceneUnits.Fill(e.Color("backgroundColor"), e.Color("backgroundColorEnd"), e.String("backgroundGradientType")),
                BackgroundCornerRadius = SceneUnits.Px(e.Float("backgroundCornerRadius") ?? 0, b.W),
                Padding = Padding(b, e),
                TextAlignment = SceneUnits.Horizontal(e.String("horizontalAlignment")),
                TextVerticalAlignment = SceneUnits.Vertical(e.String("verticalAlignment")),
                LineSpacing = 1,
                Ellipsis = null,
            };
            return Outward(clock);
        }

        internal static Control? Status(SceneBuilder b, ResolvedElement e)
        {
            IReadOnlyList<string> entries = e.List("entries") is { Count: > 0 } l ? l : ["all"];
            DeviceIndicators on = DeviceIndicators.None;
            foreach (string entry in entries)
            {
                on |= entry switch
                {
                    "all" => DeviceIndicators.All,
                    "bluetooth" => DeviceIndicators.Bluetooth,
                    "wifi" => DeviceIndicators.Wifi,
                    "cellular" => DeviceIndicators.Cellular,
                    "battery" => DeviceIndicators.Battery | DeviceIndicators.BatteryPercentage,
                    _ => DeviceIndicators.None,
                };
            }

            var icons = e.Keyed("customIcon").Where(p => p.Value.Exists).ToDictionary(p => p.Key.StartsWith("icon_", StringComparison.Ordinal) ? p.Key[5..] : p.Key, p => p.Value.Absolute);
            return Outward(new DeviceStatusBar
            {
                Status = b.Data.Status,
                Indicators = on,
                Icons = icons,
                IconHeight = SceneUnits.Px(e.Float("height") ?? 0.035f, b.H),
                FontPath = ImageElements.Existing(e.Path("fontPath")),
                TextScale = e.Float("textRelativeScale") ?? 0.9f,
                Color = SceneUnits.ToColor(e.Color("color"), Colors.White),
                BackgroundColor = SceneUnits.ToColor(e.Color("backgroundColor"), Colors.Transparent),
                BackgroundCornerRadius = SceneUnits.Px(e.Float("backgroundCornerRadius") ?? 0, b.W),
                Padding = Padding(b, e),
                EntrySpacing = SceneUnits.Px(e.Float("entrySpacing") ?? 0.005f, b.W),
            });
        }

        // The background grows outward from the positioned box, as ES-DE draws it (§13.8), so the padding is matched by a negative margin.
        internal static T Outward<T>(T control) where T : Control
        {
            Thickness p = control switch { HintBar h => h.Padding, FontText f => f.Padding, DeviceStatusBar d => d.Padding, _ => default };
            control.Margin = new Thickness(-p.Left, -p.Top, -p.Right, -p.Bottom);
            return control;
        }

        // backgroundHorizontalPadding is left and right by the width, backgroundVerticalPadding top and bottom by the height, in whole pixels so the outward margin matches.
        private static Thickness Padding(SceneBuilder b, ResolvedElement e)
        {
            NormalizedPair h = e.Pair("backgroundHorizontalPadding") ?? default, v = e.Pair("backgroundVerticalPadding") ?? default;
            return new Thickness(Math.Round(h.X * b.W), Math.Round(v.X * b.H), Math.Round(h.Y * b.W), Math.Round(v.Y * b.H));
        }
    }
}
