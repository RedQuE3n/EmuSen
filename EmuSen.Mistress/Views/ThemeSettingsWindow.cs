using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Layout;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.Views
{
    // The ES-DE theme's options, built from its capabilities.xml, and the themes downloaded on request - see EmuSen_Settings_Reference.md §4.53.
    public class ThemeSettingsWindow : ToolWindow
    {
        private readonly AppSettings _settings;
        private readonly Func<HttpClient> _http;
        private readonly Action<bool> _applied;
        private readonly StackPanel _options = Ui.Stack(12);
        private readonly StackPanel _themes = Ui.Stack(12);
        private readonly TextBlock _status = new() { Name = "ThemeStatus", TextWrapping = Avalonia.Media.TextWrapping.Wrap };
        private readonly Dictionary<string, string> _latest = new(StringComparer.Ordinal);
        private CancellationTokenSource? _download;
        private bool _closed;

        // Called with true when a theme folder was replaced or removed, so the view reads it afresh.
        public ThemeSettingsWindow(AppSettings settings, Func<HttpClient> http, Action<bool> applied)
        {
            _settings = settings;
            _http = http;
            _applied = applied;
            Title = "Theme Settings";
            Width = 720;
            CanResize = false;
            SizeToContent = SizeToContent.Height;
            var tabs = new Tabs { Name = "ThemeSettingsTabs" };
            tabs.Add("Options", Pane(_options));
            tabs.Add("Themes", Pane(Ui.Stack(12, _themes, _status)));
            Control buttons = Ui.Buttons(Ui.Button("Close", Close)).Margin(0, 12, 0, 0);
            DockPanel.SetDock(buttons, Dock.Bottom);
            Content = new DockPanel { LastChildFill = true, Children = { buttons, tabs } }.Margin(16);
            Closed += (_, _) => { _closed = true; StopDownload(); };
            Fill();
        }

        // The download in flight, for a test to await; null when none runs.
        public Task<ThemeStamp>? Downloading { get; private set; }

        // A closed sheet leaves nothing running behind it (§16, P55).
        public void StopDownload() => _download?.Cancel();

        private static Control Pane(Control content) => new ScrollViewer { Content = content.Margin(4, 12, 4, 4), MaxHeight = 560 };

        private string? Current => string.IsNullOrWhiteSpace(_settings.BigPictureTheme) ? null : _settings.BigPictureTheme;

        public static string Key(string themeDirectory) => Path.GetFullPath(themeDirectory).TrimEnd(Path.DirectorySeparatorChar);

        // The stored choices for a theme folder, as the loader takes them.
        public static ThemeChoices ChoicesFor(AppSettings settings, string? themeDirectory) =>
            themeDirectory is not null && settings.BigPicture.TryGetValue(Key(themeDirectory), out BigPictureChoices? c)
                ? new ThemeChoices { Variant = c.Variant, ColorScheme = c.ColorScheme, FontSize = c.FontSize, AspectRatio = c.AspectRatio, Language = c.Language, Transitions = c.Transitions }
                : new ThemeChoices();

        private void Fill()
        {
            FillOptions();
            FillThemes();
        }

        private void FillOptions()
        {
            _options.Children.Clear();
            if (Current is not { } dir || !Directory.Exists(dir))
            {
                _options.Children.Add(new EmptyState { Message = "No theme chosen", Detail = "Download Art Book Next on the Themes tab, or choose an ES-DE theme folder in Preferences." });
                return;
            }

            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(dir);
            ThemeChoices chosen = ChoicesFor(_settings, dir);
            ThemeSelection now = ThemeSelection.Resolve(caps, chosen with { ScreenWidth = 1280, ScreenHeight = 800 });
            _options.Children.Add(Ui.Header(caps.ThemeName));

            Row("Variant", "VariantDropdown", "The layout of the game lists. A theme may switch to another when your games have no media it needs.",
                caps.Variants.Where(v => v.Selectable).Select(v => (v.Name, Label(v.Labels, v.Name))), chosen.Variant ?? now.Variant, (c, v) => c.Variant = v);
            Row("Colour Scheme", "ColorSchemeDropdown", "The theme's colours and artwork.",
                caps.ColorSchemes.Select(s => (s.Name, Label(s.Labels, s.Name))), chosen.ColorScheme ?? now.ColorScheme, (c, v) => c.ColorScheme = v);
            Row("Font Size", "FontSizeDropdown", "The size of the theme's text.",
                caps.FontSizes.Select(f => (f, FontSizeLabel(f))), chosen.FontSize ?? now.FontSize, (c, v) => c.FontSize = v);
            Row("Aspect Ratio", "AspectRatioDropdown", "Automatic takes the ratio nearest the screen's.",
                caps.AspectRatios.Count == 0 ? [] : caps.AspectRatios.Select(a => (a, a.Replace("_vertical", " vertical"))).Prepend(("automatic", "Automatic")),
                chosen.AspectRatio ?? "automatic", (c, v) => c.AspectRatio = v == "automatic" ? null : v);
            Row("Language", "LanguageDropdown", "The language of the theme's own words.",
                caps.Languages.Select(l => (l, l)), chosen.Language ?? now.Language, (c, v) => c.Language = v);
            IEnumerable<(string, string)> transitions = caps.Transitions.Where(t => t.Selectable).Select(t => (t.Name, Label(t.Labels, t.Name)))
                .Concat(ThemeCapabilities.BuiltInTransitions.Where(b => !caps.SuppressedTransitions.Contains(b)).Select(b => (b, BuiltInLabel(b))));
            List<(string, string)> transitionList = transitions.ToList();
            Row("Transitions", "TransitionsDropdown", "How the view changes between the systems and a game list.",
                transitionList.Count == 0 ? [] : transitionList.Prepend(("automatic", "Automatic")), chosen.Transitions ?? "automatic", (c, v) => c.Transitions = v == "automatic" ? null : v);
        }

        // One dropdown row, left out when the theme declares nothing for it; a choice is stored for this theme and applied at once.
        private void Row(string label, string name, string hint, IEnumerable<(string Value, string Text)> entries, string? selected, Action<BigPictureChoices, string> store)
        {
            var list = entries.ToList();
            if (list.Count == 0) return;
            var dropdown = new Dropdown { Name = name, HorizontalAlignment = HorizontalAlignment.Stretch };
            string[] texts = list.Select(e => list.Count(o => o.Text == e.Text) > 1 ? $"{e.Text} ({e.Value})" : e.Text).ToArray();
            int at = list.FindIndex(e => e.Value == selected);
            dropdown.Fill(texts, at >= 0 ? texts[at] : texts[0]);
            dropdown.Chose += chosen =>
            {
                int i = Array.IndexOf(texts, chosen as string);
                if (i < 0 || Current is not { } dir) return;
                string key = Key(dir);
                if (!_settings.BigPicture.TryGetValue(key, out BigPictureChoices? choices)) _settings.BigPicture[key] = choices = new BigPictureChoices();
                store(choices, list[i].Value);
                _settings.Save();
                _applied(false);
            };
            _options.Children.Add(new FieldRow { Label = label, Hint = hint, Content = dropdown });
        }

        private static string Label(IReadOnlyDictionary<string, string> labels, string name) => labels.GetValueOrDefault("en_US") ?? labels.Values.FirstOrDefault() ?? name;

        private static string FontSizeLabel(string size) => size switch
        {
            "x-small" => "Extra Small", "small" => "Small", "medium" => "Medium", "large" => "Large", "x-large" => "Extra Large", _ => size,
        };

        private static string BuiltInLabel(string name) => name switch
        {
            "builtin-instant" => "Instant (built in)", "builtin-slide" => "Slide (built in)", "builtin-fade" => "Fade (built in)", _ => name,
        };

        private void FillThemes()
        {
            _themes.Children.Clear();
            IReadOnlyList<InstalledTheme> installed = ThemeDownloads.Installed(Current);
            if (installed.Count == 0)
                _themes.Children.Add(new EmptyState { Message = "No themes", Detail = "Download one below. Mistress ships no theme." });
            foreach (InstalledTheme theme in installed) _themes.Children.Add(ThemeRow(theme));

            ThemeSource artBook = ThemeSource.ArtBookNext;
            if (!ThemeDownloads.IsDownloaded(ThemeDownloads.DirectoryFor(artBook)))
            {
                Button download = Ui.Button("Download", () => _ = DownloadAsync(artBook));
                download.Name = "DownloadArtBookNext";
                download.IsEnabled = _download is null;
                _themes.Children.Add(new FieldRow
                {
                    Label = "Download Art Book Next",
                    Hint = $"The ES-DE edition, from {artBook.Url.Replace("https://", "")}, about 220 MB, into Mistress's own folder. It is someone else's work under its own licence, shown on its About sheet.",
                    Content = download,
                });
            }
        }

        private Control ThemeRow(InstalledTheme theme)
        {
            bool inUse = Current is { } c && ThemeDownloads.SamePath(c, theme.Directory);
            string where = theme.Stamp is { } s
                ? $"Downloaded {s.Downloaded:yyyy-MM-dd} from {s.Source.Url.Replace("https://", "")}" + (s.Commit is { Length: >= 7 } commit ? $", commit {commit[..7]}" : "")
                : $"Read in place from {theme.Directory}";
            var buttons = new List<Button>();
            string id = Path.GetFileName(theme.Directory);
            Button use = Ui.Button(inUse ? "In Use" : "Use", () => Use(theme.Directory));
            use.Name = $"ThemeUse.{id}";
            use.IsEnabled = !inUse;
            buttons.Add(use);
            if (theme.Stamp is { } stamp)
            {
                bool newer = _latest.TryGetValue(theme.Directory, out string? latest) && latest != stamp.Commit;
                Button update = Ui.Button(newer ? "Update" : "Check for Update", () => _ = newer ? DownloadAsync(stamp.Source) : CheckAsync(theme.Directory, stamp));
                update.IsEnabled = _download is null;
                update.Name = $"ThemeUpdate.{id}";
                buttons.Add(update);
                Button remove = Ui.Button("Remove", () => _ = RemoveAsync(theme));
                remove.IsEnabled = _download is null;
                remove.Name = $"ThemeRemove.{id}";
                buttons.Add(remove);
            }
            Button about = Ui.Button("About", () => ShowAbout(theme.Directory));
            about.Name = $"ThemeAbout.{id}";
            buttons.Add(about);
            return new FieldRow { Label = theme.Name, Hint = where, Content = Ui.Row(8, buttons.ToArray()) };
        }

        private void Use(string directory)
        {
            _settings.BigPictureTheme = directory;
            if (_settings.LibraryStyle != AppSettings.LibraryStyleTheme) _settings.LibraryStyle = AppSettings.LibraryStyleTheme;
            _settings.Save();
            _applied(true);
            Fill();
        }

        private async Task CheckAsync(string directory, ThemeStamp stamp)
        {
            _status.Text = "Asking GitHub for the newest commit…";
            try
            {
                using HttpClient http = _http();
                string? latest = await ThemeDownloads.LatestCommitAsync(http, stamp.Source);
                if (latest is null) { _status.Text = "GitHub did not say which commit is newest."; return; }
                _latest[directory] = latest;
                _status.Text = latest == stamp.Commit ? "Up to date." : $"Update available: {Short(stamp.Commit)} → {Short(latest)}.";
            }
            catch (Exception ex) when (ex is HttpRequestException or TaskCanceledException or System.Text.Json.JsonException)
            {
                _status.Text = $"Could not check for an update: {ex.Message}";
            }
            FillThemes();
        }

        private static string Short(string? commit) => commit is { Length: >= 7 } ? commit[..7] : commit ?? "unknown";

        // A download, or an update keeping theme-customizations; the About sheet opens once after a theme's first download.
        private async Task DownloadAsync(ThemeSource source)
        {
            if (_download is not null) return;
            var cancel = _download = new CancellationTokenSource();
            string directory = ThemeDownloads.DirectoryFor(source);
            bool first = !ThemeDownloads.IsDownloaded(directory);
            FillThemes();
            _status.Text = $"Downloading {source.Repository}…";
            var progress = new Progress<(long Read, long? Total)>(p =>
            {
                if (_download == cancel) _status.Text = $"Downloading {source.Repository}: {p.Read / 1e6:F1}" + (p.Total is { } t ? $" of {t / 1e6:F1} MB" : " MB");
            });
            try
            {
                using HttpClient http = _http();
                http.Timeout = TimeSpan.FromMinutes(30);
                var unpacking = new Progress<(int Done, int Total)>(p =>
                {
                    if (_download == cancel) _status.Text = $"Unpacking {source.Repository}: {p.Done} of {p.Total} files";
                });
                Downloading = ThemeDownloads.FetchAsync(http, source, progress, cancel.Token, unpacking);
                ThemeStamp stamp = await Downloading;
                _latest[directory] = stamp.Commit ?? "";
                _status.Text = $"Installed {ThemeDownloads.NameOf(directory)}" + (stamp.Commit is null ? "." : $" at commit {Short(stamp.Commit)}.");
                if (Current is null) _settings.BigPictureTheme = directory;
                _settings.Save();
                _applied(true);
                if (first && !_closed) ShowAbout(directory);
            }
            catch (OperationCanceledException)
            {
                _status.Text = "The download was stopped; the theme there before is unchanged.";
            }
            catch (Exception ex) when (ex is HttpRequestException or IOException or InvalidDataException or UnauthorizedAccessException)
            {
                _status.Text = $"The download failed, and the theme there before is unchanged: {ex.Message}";
            }
            finally
            {
                _download = null;
                cancel.Dispose();
            }
            Fill();
        }

        private async Task RemoveAsync(InstalledTheme theme)
        {
            if (!await Dialogs.ConfirmAsync(this, "Remove Theme", $"Remove {theme.Name}? Its folder is deleted, theme-customizations included.", "Remove", "Cancel")) return;
            try
            {
                ThemeDownloads.Remove(theme.Directory);
                if (Current is { } c && ThemeDownloads.SamePath(c, theme.Directory)) _settings.BigPictureTheme = null;
                _settings.BigPicture.Remove(Key(theme.Directory));
                _settings.Save();
                _status.Text = $"Removed {theme.Name}.";
                _applied(true);
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException or InvalidOperationException)
            {
                _status.Text = $"Could not remove {theme.Name}: {ex.Message}";
            }
            Fill();
        }

        private void ShowAbout(string directory) => _ = SheetLayer.Show(new ThemeAboutWindow(ThemeAttribution.Read(directory)), this);
    }
}
