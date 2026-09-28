using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Layout;
using Avalonia.Media.Imaging;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Views
{
    // One game whose cover can be borrowed: its ROM, the name the library shows, its console and the picture it shows now.
    public sealed record CoverCandidate(string Path, string Title, string Console, string Cover);

    // The order the picker lists games in: those whose name starts with the most of this game's words first, then by name - see EmuSen_Settings_Reference.md §4.65.
    public static class CoverCandidates
    {
        public const int Shown = 60;

        private static string[] Words(string title) =>
            ArtworkIndex.Untagged(title).ToLowerInvariant().Split([' ', '-', ':', ',', '.', '\'', '!', '&'], StringSplitOptions.RemoveEmptyEntries);

        public static int SharedWords(string a, string b)
        {
            string[] x = Words(a), y = Words(b);
            int n = 0;
            while (n < x.Length && n < y.Length && x[n] == y[n]) n++;
            return n;
        }

        public static IReadOnlyList<CoverCandidate> Rank(string title, IEnumerable<CoverCandidate> all, string? search) =>
            all.Where(c => string.IsNullOrWhiteSpace(search) || FilterBar.Matches(search, c.Title))
               .OrderByDescending(c => SharedWords(title, c.Title))
               .ThenBy(c => c.Title, StringComparer.OrdinalIgnoreCase)
               .ThenBy(c => c.Path, StringComparer.Ordinal)
               .Take(Shown)
               .ToList();
    }

    // Use Another Game's Cover...: a game of the library chosen by name, whose cover this game then shows; a sheet in a big-screen session - see EmuSen_Settings_Reference.md §4.65.
    public sealed class CoverPickerWindow : ToolWindow, IPadDriven
    {
        private readonly string _title;
        private readonly IReadOnlyList<CoverCandidate> _all;
        private readonly Action<CoverCandidate> _chosen;
        private readonly StackPanel _list = Ui.Stack(4);
        private readonly TextBox _search = new() { Name = "CoverPickerSearch", PlaceholderText = "Search the library", HorizontalAlignment = HorizontalAlignment.Stretch };
        private readonly Dictionary<string, Bitmap?> _thumbs = new(StringComparer.Ordinal);

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public CoverPickerWindow() : this("", [], null, _ => { }, null) { }

        public CoverPickerWindow(string title, IReadOnlyList<CoverCandidate> all, string? borrowedFrom, Action<CoverCandidate> chosen, Action? useOwn)
        {
            _title = title;
            _all = all;
            _chosen = chosen;
            Title = "Use Another Game's Cover";
            Width = 640;
            CanResize = false;
            SizeToContent = SizeToContent.Height;
            WindowStartupLocation = WindowStartupLocation.CenterOwner;
            ClosesOnEscape = true;

            var header = Ui.Stack(6,
                new TextBlock { Name = "CoverPickerTitle", Text = title, FontSize = 20, FontWeight = Avalonia.Media.FontWeight.SemiBold, TextWrapping = Avalonia.Media.TextWrapping.Wrap },
                new HintText
                {
                    Name = "CoverPickerHint",
                    Text = borrowedFrom is null
                        ? "Choose a game of your library; this game then shows its cover. Nothing is copied or sent, and Use Its Own Cover undoes it."
                        : $"Shows the cover of {borrowedFrom} now. Choose another game, or use this game's own cover again.",
                });
            if (borrowedFrom is not null && useOwn is not null)
            {
                Button own = Ui.Button("Use Its Own Cover", () => { Close(); useOwn(); });
                own.Name = "CoverPickerUseOwn";
                own.HorizontalAlignment = HorizontalAlignment.Left;
                header.Children.Add(own);
            }
            header.Children.Add(_search);

            Button close = Ui.Button("Cancel", Close);
            close.Name = "CoverPickerCancel";
            Content = Ui.Stack(12, header, new ScrollViewer { Name = "CoverPickerScroll", Content = _list, MaxHeight = 460 }, Ui.Buttons(close)).Margin(16);
            _search.TextChanged += (_, _) => Fill();
            Fill();
            Opened += (_, _) => ((InputElement?)Entries.FirstOrDefault() ?? _search).Focus(NavigationMethod.Directional);
            Closed += (_, _) => { foreach (Bitmap? b in _thumbs.Values) b?.Dispose(); _thumbs.Clear(); };
        }

        public IEnumerable<Button> Entries => _list.Children.OfType<Button>();

        public IReadOnlyList<CoverCandidate> Listed { get; private set; } = [];

        private void Fill()
        {
            _list.Children.Clear();
            Listed = CoverCandidates.Rank(_title, _all, _search.Text);
            foreach (CoverCandidate c in Listed)
            {
                var row = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 12 };
                row.Children.Add(new Image { Source = Thumb(c.Cover), Width = 48, Height = 48, Stretch = Avalonia.Media.Stretch.Uniform });
                row.Children.Add(Ui.Stack(0,
                    new TextBlock { Text = c.Title, TextTrimming = Avalonia.Media.TextTrimming.CharacterEllipsis },
                    new HintText { Text = c.Console }).Margin(0, 2, 0, 0));
                var button = new Button { Content = row, Tag = c, HorizontalAlignment = HorizontalAlignment.Stretch, HorizontalContentAlignment = HorizontalAlignment.Left };
                button.Name = "CoverPickerGame_" + _list.Children.Count;
                button.Click += (_, _) => Choose(c);
                _list.Children.Add(button);
            }
            if (Listed.Count == 0)
                _list.Children.Add(new HintText { Name = "CoverPickerEmpty", Text = _all.Count == 0 ? "No other game in the library has a cover yet." : "No game with a cover matches the search." });
        }

        public void Choose(CoverCandidate candidate)
        {
            Close();
            _chosen(candidate);
        }

        // Small enough to decode sixty at once on opening; a picture that will not decode is left blank.
        private Bitmap? Thumb(string path)
        {
            if (_thumbs.TryGetValue(path, out Bitmap? kept)) return kept;
            try
            {
                using FileStream stream = File.OpenRead(path);
                return _thumbs[path] = Bitmap.DecodeToWidth(stream, 96);
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException or ArgumentException or InvalidOperationException or NotSupportedException)
            {
                return _thumbs[path] = null;
            }
        }

        // Select, as in the options menu, puts the picker away; B does too through the sheet.
        public bool OnPad(UiButton button)
        {
            if (button != UiButton.Options) return false;
            Close();
            return true;
        }
    }
}
