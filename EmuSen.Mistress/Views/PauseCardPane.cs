using System;
using System.Collections.Generic;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Controls.Shapes;
using Avalonia.Layout;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Media;

namespace EmuSen.Mistress.Views
{
    // A card's context pane: the save slots, the reel, the paused frame or a cover, or the tools' status lines - see EmuSen_BigPicture.md §48.4 and §49.4.
    public sealed class PauseCardPane : Border
    {
        // A slot's or a moment's picture in design pixels, three across the pane's 592, the gap between them, the type and the ring (§48.4).
        private const double SlotWidth = 184, SlotHeight = 138, Gap = 20, Caption = 20, Words = 22, Ring = 3, Radius = 8;

        public PauseCardPane()
        {
            Name = "PauseCardPane";
            HorizontalAlignment = HorizontalAlignment.Stretch;
            VerticalAlignment = VerticalAlignment.Top;
            IsVisible = false;
        }

        // An empty pane hides, and the card's raised surface with it.
        protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
        {
            base.OnPropertyChanged(change);
            if (change.Property == ChildProperty) IsVisible = Child is not null;
        }

        // What the pane shows now, for a test or the recorder: none, slots, reel, frame or lines.
        public string Showing { get; private set; } = "none";

        private double U => MenuPanel.GetScale(this) is > 0 and var s ? s : 1;

        public void Clear()
        {
            Showing = "none";
            Child = null;
        }

        // The current slot between its neighbours as pictures and times, the current one ringed in the accent, and what A does to it.
        public void ShowSlots(IReadOnlyList<(string Name, string When, IImage? Picture)> slots, int current, string sentence)
        {
            double u = U;
            int first = Math.Clamp(current - 2, 0, Math.Max(0, slots.Count - 3));
            var strip = new StackPanel { Name = "PaneSlots", Orientation = Orientation.Horizontal, Spacing = Gap * u };
            for (int i = first; i < Math.Min(slots.Count, first + 3); i++)
            {
                (string name, string when, IImage? picture) = slots[i];
                bool ringed = i + 1 == current;
                var frame = new Border
                {
                    Name = $"PaneSlot{i + 1}", Width = SlotWidth * u, Height = SlotHeight * u, CornerRadius = new CornerRadius(Radius * u),
                    BorderThickness = new Thickness((ringed ? Ring : 0) * u), ClipToBounds = true,
                    Child = picture is null ? Empty(u) : new Image { Source = picture, Stretch = Stretch.UniformToFill },
                };
                frame.Bind(BorderBrushProperty, frame.GetResourceObservable("LunaAccent"));
                if (picture is not null) frame.Bind(BackgroundProperty, frame.GetResourceObservable("LunaInputSurface"));
                strip.Children.Add(new StackPanel { Spacing = 4 * u, Children = { frame, Line(name, Caption * u, Ink.Strong), Line(when, Caption * u, Ink.Muted) } });
            }
            Showing = "slots";
            Child = new StackPanel { Spacing = 12 * u, Children = { strip, Line(sentence, Words * u, Ink.Text, name: "PaneSentence", wraps: true) } };
        }

        // An empty slot: a dashed outline in the Border token with the word in it.
        private static Control Empty(double u)
        {
            var outline = new Rectangle { StrokeThickness = Math.Max(1, u), StrokeDashArray = [4, 3], RadiusX = Radius * u, RadiusY = Radius * u };
            outline.Bind(Shape.StrokeProperty, outline.GetResourceObservable("LunaBorder"));
            return new Panel { Children = { outline, Line("Empty", Words * u, Ink.Muted, centred: true) } };
        }

        // The newest moments of the reel, oldest first, each with how long ago it was, and what the reel holds.
        public void ShowReel(IReadOnlyList<(IImage Picture, string Ago)> moments, string held)
        {
            double u = U;
            var row = new StackPanel { Name = "PaneReel", Orientation = Orientation.Horizontal, Spacing = Gap * u };
            foreach ((IImage picture, string ago) in moments)
            {
                var frame = new Border { Width = SlotWidth * u, Height = SlotHeight * u, CornerRadius = new CornerRadius(Radius * u), ClipToBounds = true, Child = new Image { Source = picture, Stretch = Stretch.UniformToFill } };
                row.Children.Add(new StackPanel { Spacing = 4 * u, Children = { frame, Line(ago, Caption * u, Ink.Text) } });
            }
            Showing = "reel";
            Child = new StackPanel { Spacing = 12 * u, Children = { row, Line(held, Words * u, Ink.Text, name: "PaneReelCaption", wraps: true) } };
        }

        // One picture, such as the paused frame or a cover, at most the width and height given, a name under it where one is given, and a line under that.
        public void ShowPicture(IImage? picture, string caption, double width, string? title = null, double? height = null)
        {
            double u = U;
            var stack = new StackPanel { Spacing = 8 * u, MaxWidth = Math.Max(width, 1) };
            if (picture is not null) stack.Children.Add(new Image { Name = "PanePicture", Source = picture, Stretch = Stretch.Uniform, MaxWidth = width, MaxHeight = height ?? double.PositiveInfinity });
            if (title is not null)
            {
                TextBlock name = Line(title, Words * u, Ink.Strong, name: "PaneTitle");
                name.TextWrapping = TextWrapping.Wrap;
                name.MaxLines = 2;
                stack.Children.Add(name);
            }
            stack.Children.Add(Line(caption, Caption * u, Ink.Muted, name: "PaneCaption"));
            Showing = "frame";
            Child = stack;
        }

        // A picture at the left, such as a game's cover, and beside it its name in the pane's title type and lines under it, the last a muted hint.
        public void ShowBeside(IImage? picture, string title, IReadOnlyList<string> lines, string hint, double width)
        {
            double u = U;
            var words = new StackPanel { Spacing = 8 * u };
            TextBlock name = Line(title, Words * u, Ink.Strong, name: "PaneTitle", wraps: true);
            name.FontFamily = LunaFonts.Condensed;
            name.FontSize = 32 * u;
            words.Children.Add(name);
            for (int i = 0; i < lines.Count; i++) words.Children.Add(Line(lines[i], Words * u, i == 0 ? Ink.Muted : Ink.Text, name: i == 0 ? "PaneCaption" : null, wraps: true));
            words.Children.Add(Line(hint, Words * u, Ink.Muted, name: "PaneHint", wraps: true));
            var grid = new Grid { ColumnDefinitions = new ColumnDefinitions("Auto,*"), ColumnSpacing = 24 * u };
            if (picture is not null)
            {
                var cover = new Border { CornerRadius = new CornerRadius(Radius * u), ClipToBounds = true, VerticalAlignment = VerticalAlignment.Top, Child = new Image { Name = "PanePicture", Source = picture, Stretch = Stretch.Uniform, MaxWidth = width, MaxHeight = width } };
                grid.Children.Add(cover);
            }
            Grid.SetColumn(words, picture is null ? 0 : 1);
            if (picture is null) Grid.SetColumnSpan(words, 2);
            grid.Children.Add(words);
            Showing = "frame";
            Child = grid;
        }

        // Named lines with a status under each, such as the tools' (§49.4).
        public void ShowLines(IReadOnlyList<(string Name, string Status)> lines, double width)
        {
            double u = U;
            var stack = new StackPanel { Name = "PaneLines", Spacing = 12 * u, Width = width };
            foreach ((string name, string status) in lines)
            {
                var pair = new StackPanel { Spacing = 2 * u };
                TextBlock title = Line(name, Words * u, Ink.Strong), under = Line(status, Caption * u, Ink.Muted);
                title.TextAlignment = under.TextAlignment = TextAlignment.Left;
                under.TextWrapping = TextWrapping.Wrap;
                pair.Children.Add(title);
                pair.Children.Add(under);
                stack.Children.Add(pair);
            }
            Showing = "lines";
            Child = stack;
        }

        // A tool's name, its state in a sentence, and a muted line of what A and B do (EmuSen_Interface.md §19.6).
        public void ShowSaid(string title, string sentence, string hint, double width)
        {
            double u = U;
            var stack = new StackPanel { Name = "PaneSaid", Spacing = 6 * u, Width = width };
            stack.Children.Add(Line(title, Words * u, Ink.Text, "PaneTitle"));
            stack.Children.Add(Line(sentence, Words * u, Ink.Text, "PaneSentence", wraps: true));
            stack.Children.Add(Line(hint, Words * u, Ink.Muted, "PaneHint", wraps: true));
            Showing = "said";
            Child = stack;
        }

        // What a row does in sentences, then a line in the success or error colour, such as whether firmware was found.
        public void ShowSentences(IReadOnlyList<string> sentences, (string Line, bool Good)? flag, double width)
        {
            double u = U;
            var stack = new StackPanel { Name = "PaneSentences", Spacing = 6 * u, Width = width };
            foreach (string sentence in sentences) stack.Children.Add(Line(sentence, Words * u, Ink.Text, wraps: true));
            if (flag is { } f)
            {
                TextBlock line = Line(f.Line, Words * u, Ink.Text, "PaneFlag");
                line.Bind(TextBlock.ForegroundProperty, line.GetResourceObservable(f.Good ? "LunaSuccess" : "LunaError"));
                stack.Children.Add(line);
            }
            Showing = "sentences";
            Child = stack;
        }

        private enum Ink { Strong, Text, Muted }

        private static TextBlock Line(string text, double size, Ink ink, string? name = null, bool wraps = false, bool centred = false)
        {
            var line = new TextBlock
            {
                Text = text, FontFamily = LunaFonts.Body, FontSize = size, TextAlignment = centred ? TextAlignment.Center : TextAlignment.Left,
                TextWrapping = wraps ? TextWrapping.Wrap : TextWrapping.NoWrap, TextTrimming = TextTrimming.CharacterEllipsis,
            };
            if (centred) line.VerticalAlignment = VerticalAlignment.Center;
            if (name is not null) line.Name = name;
            line.Bind(TextBlock.ForegroundProperty, line.GetResourceObservable(ink switch { Ink.Strong => "LunaTextStrong", Ink.Text => "LunaText", _ => "LunaMuted" }));
            return line;
        }
    }
}
