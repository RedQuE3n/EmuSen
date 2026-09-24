using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia;
using Avalonia.Automation;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Layout;
using Avalonia.Markup.Xaml.MarkupExtensions;
using Avalonia.Media;
using Avalonia.VisualTree;
using EmuSen.Common;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Views
{
    // One tile of the reel: a held moment with its picture, or the present - see EmuSen_Settings_Reference.md §4.49.
    public sealed class ReelMoment
    {
        private byte[]? _rgba;

        public ReelMoment(long frame, double secondsBack, RewindThumbnail picture, bool isNow)
        {
            Frame = frame;
            SecondsBack = secondsBack;
            Picture = picture;
            IsNow = isNow;
        }

        public long Frame { get; }
        public double SecondsBack { get; }
        public RewindThumbnail Picture { get; }
        public bool IsNow { get; }

        // Made when a tile first shows it, and kept while the reel is open.
        public byte[] Rgba => _rgba ??= Picture.ToRgba();

        public string Label => IsNow ? "Now" : Ago(SecondsBack);

        public static string Ago(double seconds) =>
            seconds < 10 ? $"{seconds:F2} s ago" : seconds < 60 ? $"{seconds:F1} s ago" : $"{(int)(seconds / 60)}:{(int)(seconds % 60):00} ago";

        // The held moments that have pictures, oldest first, then the present - see §4.49.
        public static List<ReelMoment> From(IReadOnlyList<RewindMoment> moments, long now, double frameRateHz, RewindThumbnail? present)
        {
            double hz = frameRateHz > 0 ? frameRateHz : 60;
            var reel = moments.Where(m => m.Thumbnail is not null && m.Frame <= now)
                .Select(m => new ReelMoment(m.Frame, (now - m.Frame) / hz, m.Thumbnail!, isNow: false))
                .ToList();
            if (present is not null) reel.Add(new ReelMoment(now, 0, present, isNow: true));
            return reel;
        }

        public override string ToString() => Label;
    }

    // The rewind history as a strip of pictures to choose from; closes with the chosen moment, or with nothing to cancel - see EmuSen_Settings_Reference.md §4.49.
    public sealed class RewindReelWindow : ToolWindow, IPadDriven
    {
        public const double StrideSeconds = 5;
        public const string PadHint = "Left Right  Step      L1 R1  Five seconds      L2 R2  Oldest, now      A  Rewind here      B  Cancel";

        private readonly IReadOnlyList<ReelMoment> _moments;
        private readonly RgbaImageView _preview = new() { Name = "ReelPreview", Stretch = Stretch.Uniform, HorizontalAlignment = HorizontalAlignment.Center };
        private readonly HintText _hint = Ui.Hint(PadHint);
        private readonly TextBlock _caption = new() { Name = "ReelCaption", FontWeight = FontWeight.SemiBold, HorizontalAlignment = HorizontalAlignment.Center };

        public TileStrip<ReelMoment> Strip { get; } = new() { Name = "ReelStrip", TileWidth = 150, TileHeight = 136, Spacing = 12, Height = 176 };

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public RewindReelWindow() : this(Array.Empty<ReelMoment>()) { }

        public RewindReelWindow(IReadOnlyList<ReelMoment> moments)
        {
            _moments = moments;
            Title = "Rewind";
            Width = 960;
            SizeToContent = SizeToContent.Height;
            WindowStartupLocation = WindowStartupLocation.CenterOwner;
            ClosesOnEscape = true;

            Strip.Label = m => m.Label;
            Strip.Key = m => m.Frame * 2 + (m.IsNow ? 1 : 0);
            Strip.CreateTile = Tile;
            Strip.BindTile = Bind;
            Strip.Chose += Show;
            Strip.Activated += Choose;
            AutomationProperties.SetName(Strip, "Rewind history");
            Strip.Refresh(moments);
            if (moments.Count > 0) Strip.Select(moments[^1]);

            var rewind = new Button { Name = "RewindHereButton", Content = "Rewind Here" };
            rewind.Click += (_, _) => { if (Strip.Selected is { } chosen) Choose(chosen); };
            var cancel = new Button { Name = "CancelRewindButton", Content = "Cancel" };
            cancel.Click += (_, _) => Close(null);

            var stage = new Border { Height = 280, Child = _preview, Margin = new Thickness(0, 0, 0, 8) };
            stage[!Border.BackgroundProperty] = new DynamicResourceExtension("LunaVoid");

            var body = Ui.Stack(8,
                stage,
                _caption,
                Strip,
                _hint,
                new ButtonBar { ItemsSource = new[] { rewind, cancel }, HorizontalAlignment = HorizontalAlignment.Right });
            Content = body.Margin(16);

            if (Strip.Selected is { } first) Show(first);
            Opened += (_, _) => Strip.Focus();
        }

        // Off where the sheet's own footer names the buttons.
        public bool ShowHint
        {
            get => _hint.IsVisible;
            set => _hint.IsVisible = value;
        }

        private static Control Tile()
        {
            var picture = new RgbaImageView { Stretch = Stretch.Uniform, Height = 108, HorizontalAlignment = HorizontalAlignment.Center };
            var label = new TextBlock { HorizontalAlignment = HorizontalAlignment.Center, Margin = new Thickness(0, 6, 0, 0) };
            return new StackPanel { Children = { picture, label } };
        }

        private static void Bind(Control tile, ReelMoment moment)
        {
            var panel = (StackPanel)tile;
            ((RgbaImageView)panel.Children[0]).SetFrame(moment.Rgba, moment.Picture.Width, moment.Picture.Height);
            ((TextBlock)panel.Children[1]).Text = moment.Label;
        }

        private void Show(ReelMoment moment)
        {
            _preview.SetFrame(moment.Rgba, moment.Picture.Width, moment.Picture.Height);
            _caption.Text = moment.IsNow ? "Now - where the game is paused" : moment.Label;
        }

        // The present is no rewind at all, so choosing it resumes as cancelling does.
        private void Choose(ReelMoment moment) => Close(moment.IsNow ? null : moment);

        // The nearest moment at least StrideSeconds further in that direction, else the end - see §4.49.
        public void Stride(int direction)
        {
            int at = Strip.SelectedIndex;
            if (at < 0 || _moments.Count == 0) { Strip.Move(direction); return; }
            double from = _moments[at].SecondsBack;
            int target = direction < 0 ? 0 : _moments.Count - 1;
            if (direction < 0)
            {
                for (int i = at - 1; i >= 0; i--) if (_moments[i].SecondsBack >= from + StrideSeconds) { target = i; break; }
            }
            else
            {
                for (int i = at + 1; i < _moments.Count; i++) if (_moments[i].SecondsBack <= from - StrideSeconds) { target = i; break; }
            }
            Strip.Move(target - at);
        }

        private bool StripFocused => TopLevel.GetTopLevel(Strip)?.FocusManager?.GetFocusedElement() is not Visual focused
            || ReferenceEquals(focused, Strip) || Strip.IsVisualAncestorOf(focused);

        // The strip takes left, right and accept; the shoulders and triggers work anywhere; the rest is the router's - see §4.49.
        public bool OnPad(UiButton button)
        {
            switch (button)
            {
                case UiButton.PageUp: Stride(-1); return true;
                case UiButton.PageDown: Stride(1); return true;
                case UiButton.First: Strip.Move(int.MinValue / 2); return true;
                case UiButton.Last: Strip.Move(int.MaxValue / 2); return true;
                case UiButton.Left when StripFocused: Strip.Move(-1); return true;
                case UiButton.Right when StripFocused: Strip.Move(1); return true;
                case UiButton.Accept when StripFocused:
                    if (Strip.Selected is { } chosen) Choose(chosen);
                    return true;
                default: return false;
            }
        }
    }
}
