using System;
using System.Collections.Generic;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Layout;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Media;

namespace EmuSen.Mistress.Views
{
    // A card's context pane: the save slots, the reel, the paused frame or a cover, or the tools' status lines - see EmuSen_BigPicture.md §48.4 and §49.4.
    public sealed class PauseCardPane : Border
    {
        // A slot's picture in design pixels, the design's state picture, and the gaps between slots.
        private const double SlotWidth = 160, SlotHeight = 120, Gap = 16, Caption = 20, Words = 22, Ring = 3;

        public PauseCardPane()
        {
            Name = "PauseCardPane";
            HorizontalAlignment = HorizontalAlignment.Center;
            VerticalAlignment = VerticalAlignment.Center;
        }

        // What the pane shows now, for a test or the recorder: none, slots, reel, frame or lines.
        public string Showing { get; private set; } = "none";

        private double U => MenuPanel.GetScale(this) is > 0 and var s ? s : 1;

        public void Clear()
        {
            Showing = "none";
            Child = null;
        }

        // Every slot as a picture and its time, four across, the current one ringed in the accent.
        public void ShowSlots(IReadOnlyList<(string Name, string When, IImage? Picture)> slots, int current)
        {
            double u = U;
            var grid = new UniformGrid { Name = "PaneSlots", Columns = 4 };
            for (int i = 0; i < slots.Count; i++)
            {
                (string name, string when, IImage? picture) = slots[i];
                var tile = new StackPanel { Spacing = 4 * u, Margin = new Thickness(Gap * u / 2) };
                var frame = new Border
                {
                    Name = $"PaneSlot{i + 1}", Width = SlotWidth * u, Height = SlotHeight * u, CornerRadius = new CornerRadius(4 * u),
                    BorderThickness = new Thickness((i + 1 == current ? Ring : 1) * u), ClipToBounds = true,
                    Child = picture is null ? null : new Image { Source = picture, Stretch = Stretch.Uniform },
                };
                frame.Bind(BorderBrushProperty, frame.GetResourceObservable(i + 1 == current ? "LunaAccent" : "LunaBorder"));
                frame.Bind(BackgroundProperty, frame.GetResourceObservable("LunaInputSurface"));
                tile.Children.Add(frame);
                tile.Children.Add(Line(name, Caption * u, strong: i + 1 == current));
                tile.Children.Add(Line(when, Caption * u, strong: false));
                grid.Children.Add(tile);
            }
            Showing = "slots";
            Child = grid;
        }

        // The newest pictures of the reel, oldest first, and how far back it reaches.
        public void ShowReel(IReadOnlyList<IImage> pictures, string held)
        {
            double u = U;
            var row = new StackPanel { Name = "PaneReel", Orientation = Orientation.Horizontal, Spacing = Gap * u, HorizontalAlignment = HorizontalAlignment.Center };
            foreach (IImage picture in pictures)
                row.Children.Add(new Border { Width = SlotWidth * u, Height = SlotHeight * u, ClipToBounds = true, Child = new Image { Source = picture, Stretch = Stretch.Uniform } });
            var stack = new StackPanel { Spacing = 12 * u, Children = { row, Line(held, Words * u, strong: false, name: "PaneReelCaption") } };
            Showing = "reel";
            Child = stack;
        }

        // One picture, such as the paused frame or a cover, at most the width and height given, a name under it where one is given, and a line under that.
        public void ShowPicture(IImage? picture, string caption, double width, string? title = null, double? height = null)
        {
            double u = U;
            var stack = new StackPanel { Spacing = 8 * u, MaxWidth = Math.Max(width, 1) };
            if (picture is not null) stack.Children.Add(new Image { Name = "PanePicture", Source = picture, Stretch = Stretch.Uniform, MaxWidth = width, MaxHeight = height ?? double.PositiveInfinity });
            if (title is not null)
            {
                TextBlock name = Line(title, Words * u, strong: true, name: "PaneTitle");
                name.TextWrapping = TextWrapping.Wrap;
                name.MaxLines = 2;
                stack.Children.Add(name);
            }
            stack.Children.Add(Line(caption, Caption * u, strong: false, name: "PaneCaption"));
            Showing = "frame";
            Child = stack;
        }

        // Named lines with a status under each, such as the tools' (§49.4).
        public void ShowLines(IReadOnlyList<(string Name, string Status)> lines, double width)
        {
            double u = U;
            var stack = new StackPanel { Name = "PaneLines", Spacing = 12 * u, Width = width };
            foreach ((string name, string status) in lines)
            {
                var pair = new StackPanel { Spacing = 2 * u };
                TextBlock title = Line(name, Words * u, strong: true), under = Line(status, Caption * u, strong: false);
                title.TextAlignment = under.TextAlignment = TextAlignment.Left;
                under.TextWrapping = TextWrapping.Wrap;
                pair.Children.Add(title);
                pair.Children.Add(under);
                stack.Children.Add(pair);
            }
            Showing = "lines";
            Child = stack;
        }

        private static TextBlock Line(string text, double size, bool strong, string? name = null)
        {
            var line = new TextBlock { Text = text, FontFamily = LunaFonts.Body, FontSize = size, TextAlignment = TextAlignment.Center, TextWrapping = TextWrapping.NoWrap, TextTrimming = TextTrimming.CharacterEllipsis };
            if (name is not null) line.Name = name;
            line.Bind(TextBlock.ForegroundProperty, line.GetResourceObservable(strong ? "LunaTextStrong" : "LunaMuted"));
            return line;
        }
    }
}
