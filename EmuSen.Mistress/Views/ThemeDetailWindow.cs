using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture;
using EmuSen.Galaxia.Library;

namespace EmuSen.Mistress.Views
{
    // One theme of ES-DE's list: its screenshots, what it supports, its last update, its licence line before Download, and its install, update and removal - see EmuSen_Settings_Reference.md §4.62.
    public class ThemeDetailWindow : ToolWindow
    {
        private readonly ThemeBrowserWindow _browserWindow;
        private readonly ThemeSettingsWindow _owner;
        private readonly ThemeBrowser _browser;
        private ThemeBrowserEntry _entry;
        private ThemeRemote? _remote;
        private int _at;
        private CancellationTokenSource? _download, _shotCancel;

        private readonly FittedImage _shot = new() { Name = "ThemeDetailScreenshot", Width = 496, Height = 279, Fit = ImageFit.Contain };
        private readonly TextBlock _caption = new() { Name = "ThemeDetailCaption", TextWrapping = TextWrapping.Wrap, Opacity = 0.8 };
        private readonly TextBlock _position = new() { Name = "ThemeDetailPosition", VerticalAlignment = VerticalAlignment.Center };
        private readonly TextBlock _updated = new() { Name = "ThemeDetailUpdated", TextWrapping = TextWrapping.Wrap };
        private readonly TextBlock _state = new() { Name = "ThemeDetailState", TextWrapping = TextWrapping.Wrap };
        private readonly StackPanel _licence = Ui.Stack(4);
        private readonly TextBlock _status = new() { Name = "ThemeDetailStatus", TextWrapping = TextWrapping.Wrap };
        private readonly ProgressBar _bar = new() { Name = "ThemeDetailProgress", Minimum = 0, Maximum = 1, HorizontalAlignment = HorizontalAlignment.Stretch, IsVisible = false };
        private readonly Button _previous, _next, _download_, _update, _use, _remove, _about, _cancel;

        public ThemeDetailWindow(ThemeBrowserWindow browserWindow, ThemeSettingsWindow owner, ThemeBrowserEntry entry)
        {
            _browserWindow = browserWindow;
            _owner = owner;
            _browser = browserWindow.Browser;
            _entry = entry;
            _remote = entry.Remote;
            ThemeListEntry t = entry.Theme;
            Title = t.Name;
            Width = 1040;
            CanResize = false;
            SizeToContent = SizeToContent.Height;

            _previous = Named(Ui.Button("Previous", () => Step(-1)), "ThemeDetailPrevious");
            _next = Named(Ui.Button("Next", () => Step(1)), "ThemeDetailNext");
            _download_ = Named(Ui.Button("Download", () => _ = DownloadAsync()), "ThemeDetailDownload");
            _update = Named(Ui.Button("Update", () => _ = DownloadAsync()), "ThemeDetailUpdate");
            _use = Named(Ui.Button("Use", Use), "ThemeDetailUse");
            _remove = Named(Ui.Button("Remove", () => _ = RemoveAsync()), "ThemeDetailRemove");
            _about = Named(Ui.Button("About", () => { if (_entry.Installed is { } i) _owner.ShowAbout(i.Directory, this); }), "ThemeDetailAbout");
            _cancel = Named(Ui.Button("Cancel Download", () => _download?.Cancel()), "ThemeDetailCancel");

            var rows = Ui.Stack(8,
                Words("ThemeDetailAuthor", $"by {t.Author}   ·   {t.Source?.Url.Replace("https://", "") ?? t.Url}"),
                new FieldRow { Label = "Last Updated", Content = _updated },
                new FieldRow { Label = "Here", Content = _state },
                Field("Variants", "ThemeDetailVariants", t.Variants),
                Field("Colour Schemes", "ThemeDetailColorSchemes", t.ColorSchemes),
                Field("Aspect Ratios", "ThemeDetailAspectRatios", t.AspectRatios));
            if (t.FontSizes.Count > 0) rows.Children.Add(Field("Font Sizes", "ThemeDetailFontSizes", t.FontSizes));
            if (t.Transitions.Count > 0) rows.Children.Add(Field("Transitions", "ThemeDetailTransitions", t.Transitions));
            if (t.Languages.Count > 0) rows.Children.Add(Field("Languages", "ThemeDetailLanguages", t.Languages));
            // The licence row and the buttons sit below both columns, never scrolled away, so the licence is read before Download (Q27).
            var top = new Grid { ColumnDefinitions = new ColumnDefinitions("520,20,*") };
            StackPanel shots = Ui.Stack(6, _shot, _caption, Ui.Row(12, _previous, _position, _next));
            var facts = new ScrollViewer { Name = "ThemeDetailFacts", Content = rows.Margin(0, 0, 8, 0), MaxHeight = 380, Width = 520, Focusable = true };
            Grid.SetColumn(facts, 2);
            top.Children.Add(shots);
            top.Children.Add(facts);
            var bottom = Ui.Stack(8,
                new FieldRow { Name = "ThemeDetailLicenceRow", Label = "Licence", Hint = "From the theme's own README, else the licence file its host names. The theme is its author's work, under its own terms.", Content = _licence },
                Ui.Row(8, _download_, _update, _use, _remove, _about, _cancel), _bar, _status);
            Control buttons = Ui.Buttons(Ui.Button("Close", Close)).Margin(0, 12, 0, 0);
            DockPanel.SetDock(buttons, Dock.Bottom);
            DockPanel.SetDock(bottom, Dock.Bottom);
            var dock = new DockPanel { LastChildFill = true, Children = { buttons, bottom.Margin(0, 12, 0, 0), top } }.Margin(16);
            Content = dock;
            // On a big-screen sheet: the screenshot and its caption at the left, the facts beside them in a column of their own, the licence and every button in rows below, nothing over anything else (§4.83).
            MenuLook.SetWidthFraction(this, 0.8);
            MenuLook.WhenApplied(dock, () => InLook(dock, shots, facts, bottom, buttons));
            Closed += (_, _) => { _shotCancel?.Cancel(); StopDownload(); };
            ShowShot(0);
            Refill();
            DetailsLoading = LoadRemoteAsync();
        }

        private void InLook(DockPanel dock, StackPanel shots, ScrollViewer facts, StackPanel bottom, Control buttons)
        {
            dock.Children.Clear();
            dock.Margin = new Avalonia.Thickness(0, 0, 0, 12);
            if (shots.Parent is Panel oldTop) oldTop.Children.Clear();

            _shot.Width = double.NaN;
            _shot.Height = ShotHeightInLook;
            _shot.HorizontalAlignment = HorizontalAlignment.Stretch;
            facts.Width = double.NaN;
            facts.MaxHeight = ShotHeightInLook + 90;
            facts.VerticalAlignment = VerticalAlignment.Top;
            var top = new Grid { ColumnDefinitions = new ColumnDefinitions("5*,24,4*"), Children = { shots, facts } };
            Grid.SetColumn(facts, 2);

            // Every action and Close in one row that wraps, so none is ever pushed past the panel.
            if (bottom.Children.OfType<StackPanel>().FirstOrDefault(p => p.Children.Contains(_use)) is { } actions)
            {
                var wrap = new WrapPanel { ItemSpacing = 8, LineSpacing = 8 };
                foreach (Control b in actions.Children.ToList()) { actions.Children.Remove(b); wrap.Children.Add(b); }
                wrap.Children.Add(Named(Ui.Button("Close", Close), "ThemeDetailClose"));
                bottom.Children[bottom.Children.IndexOf(actions)] = wrap;
            }
            buttons.IsVisible = false;
            DockPanel.SetDock(top, Dock.Top);
            dock.Children.Add(top);
            dock.Children.Add(bottom);
            dock.Children.Add(buttons);
        }

        // The screenshot's height on a big-screen sheet, in the menu's design pixels.
        private const double ShotHeightInLook = 210;

        // The host's answer being fetched as the sheet opens, for a test to await.
        public Task DetailsLoading { get; }

        public Task? ShotLoading { get; private set; }

        // The download in flight, for a test to await; null when none has started.
        public Task<ThemeStamp>? Downloading { get; private set; }

        public ThemeBrowserEntry Entry => _entry;

        public void StopDownload() => _download?.Cancel();

        private static Button Named(Button b, string name)
        {
            b.Name = name;
            return b;
        }

        private static TextBlock Words(string name, string text) => new() { Name = name, Text = text, TextWrapping = TextWrapping.Wrap };

        private static Control Field(string label, string name, IReadOnlyList<string> values) =>
            new FieldRow { Label = $"{label} ({values.Count})", Content = Words(name, values.Count == 0 ? "The list states none." : string.Join(", ", values)) };

        private void Step(int by)
        {
            int count = _entry.Theme.Screenshots.Count;
            if (count > 0) ShowShot((_at + by + count) % count);
        }

        // A screenshot is fetched when it is shown, then kept in home/Themes/.list with its row in themes.db.
        private void ShowShot(int at)
        {
            IReadOnlyList<ThemeScreenshot> shots = _entry.Theme.Screenshots;
            _previous.IsEnabled = _next.IsEnabled = shots.Count > 1;
            _shotCancel?.Cancel();
            if (shots.Count == 0)
            {
                _position.Text = "no screenshots";
                _caption.Text = "The list gives no screenshot.";
                return;
            }
            _at = at;
            _position.Text = $"{at + 1} of {shots.Count}";
            _caption.Text = shots[at].Caption ?? "";
            _shot.Source = null;
            var cancel = _shotCancel = new CancellationTokenSource();
            ShotLoading = LoadShotAsync(shots[at], cancel.Token);
        }

        private async Task LoadShotAsync(ThemeScreenshot shot, CancellationToken cancel)
        {
            try
            {
                string? file = await _browser.ScreenshotAsync(shot, cancel);
                if (!cancel.IsCancellationRequested) _shot.Source = file;
                if (file is null && !cancel.IsCancellationRequested) _caption.Text = "The screenshot could not be fetched.";
            }
            catch (Exception ex) when (ex is OperationCanceledException or ObjectDisposedException) { }
            catch (Exception ex) when (ex is HttpRequestException or IOException or InvalidDataException)
            {
                ErrorLog.Error("themes", "A theme screenshot could not be fetched", ex, _entry.Theme.Name);
                if (!cancel.IsCancellationRequested) _caption.Text = $"The screenshot could not be fetched: {ex.Message}";
            }
        }

        private async Task LoadRemoteAsync()
        {
            if (_browser.Closed) return;
            try
            {
                _remote = await _browser.RemoteAsync(_entry.Theme);
            }
            catch (Exception ex) when (ex is OperationCanceledException or ObjectDisposedException) { return; }
            Reread();
        }

        // The entry again from themes.db and the folder, after anything that changed it.
        private void Reread()
        {
            if (ThemeBrowser.Cached() is { } list && ThemeBrowser.Entries(list).FirstOrDefault(e => e.Theme.Url == _entry.Theme.Url) is { } now) _entry = now;
            _remote = _entry.Remote ?? _remote;
            Refill();
        }

        private void Refill()
        {
            bool installed = _entry.Installed is not null, busy = _download is not null, known = _remote is not null;
            _updated.Text = _remote is null ? "Asking the host…"
                : _remote.HeadDate is { } d ? $"{d.ToLocalTime():yyyy-MM-dd}, commit {Short(_remote.Head)}, on branch {_remote.Branch}"
                : _remote.Error is { } e ? $"The host could not say: {e}" : "The host did not say.";
            _state.Text = StateLine();
            _licence.Children.Clear();
            IEnumerable<string> lines = _remote?.Licence is { Count: > 0 } l ? l : ["Reading the licence line from the theme's repository…"];
            int i = 0;
            foreach (string line in lines) _licence.Children.Add(Words($"ThemeDetailLicence{i++}", line));

            _download_.IsVisible = !installed;
            _download_.IsEnabled = known && !busy;
            _update.IsVisible = installed;
            _update.IsEnabled = installed && known && !busy && _entry.State.HasFlag(ThemeListState.UpdateAvailable);
            _use.IsVisible = _remove.IsVisible = _about.IsVisible = installed;
            _remove.IsEnabled = !busy;
            bool inUse = installed && _owner.IsInUse(_entry.Installed!.Directory);
            _use.Content = inUse ? "In Use" : "Use";
            _use.IsEnabled = installed && !inUse && !busy;
            _cancel.IsVisible = busy;
            _bar.IsVisible = busy;
        }

        private string StateLine()
        {
            if (_entry.Installed is not { } i) return "Not installed. Download fetches the repository's archive into Mistress's own folder, home/Themes.";
            string line = $"Installed in {i.Directory}, {i.Updated.ToLocalTime():yyyy-MM-dd}, commit {Short(i.Commit)}.";
            if (_entry.State.HasFlag(ThemeListState.UpdateAvailable)) line += $" An update is available: commit {Short(_remote?.Head)}.";
            if (_entry.Changes.Any)
                line += $" Local changes to {_entry.Changes.Modified.Count + _entry.Changes.Missing.Count} files ({string.Join(", ", _entry.Changes.Modified.Concat(_entry.Changes.Missing).Take(3))}" +
                        (_entry.Changes.Modified.Count + _entry.Changes.Missing.Count > 3 ? ", …" : "") + "); an update would replace them.";
            else if (!_entry.Changes.Known) line += " Local changes cannot be told for a theme downloaded before this list existed.";
            return line;
        }

        private static string Short(string? commit) => commit is { Length: >= 7 } ? commit[..7] : commit ?? "unknown";

        // Download or update; an update over local changes asks first, as ES-DE's downloader does.
        private async Task DownloadAsync()
        {
            if (_download is not null || _remote is null || _browser.Closed) return;
            string? directory = _entry.Installed?.Directory;
            bool first = directory is null, replace = false;
            if (directory is not null && ThemeDownloads.LocalChanges(directory) is { Any: true } changes)
            {
                string files = string.Join(", ", changes.Modified.Concat(changes.Missing).Take(5)) + (changes.Modified.Count + changes.Missing.Count > 5 ? ", …" : "");
                if (!await Dialogs.ConfirmAsync(this, "Local Changes", $"{_entry.Theme.Name} has local changes to {changes.Modified.Count + changes.Missing.Count} files ({files}). The update replaces them with the theme's own. Files you added, and theme-customizations, are kept.", "Update Anyway", "Cancel")) return;
                replace = true;
            }
            var cancel = _download = new CancellationTokenSource();
            _bar.Value = 0;
            _status.Text = $"Downloading {_entry.Theme.Name}…";
            Refill();
            var progress = new Progress<(long Read, long? Total)>(p =>
            {
                if (_download != cancel) return;
                _bar.IsIndeterminate = p.Total is null;
                if (p.Total is { } total and > 0) _bar.Value = (double)p.Read / total;
                _status.Text = $"Downloading {_entry.Theme.Name}: {p.Read / 1e6:F1}" + (p.Total is { } t ? $" of {t / 1e6:F1} MB" : " MB");
            });
            var unpacking = new Progress<(int Done, int Total)>(p =>
            {
                if (_download != cancel) return;
                _bar.IsIndeterminate = false;
                _bar.Value = p.Total == 0 ? 1 : (double)p.Done / p.Total;
                _status.Text = $"Unpacking {_entry.Theme.Name}: {p.Done} of {p.Total} files";
            });
            try
            {
                Downloading = _browser.DownloadAsync(_entry.Theme, _remote, progress, unpacking, replace, cancel.Token);
                ThemeStamp stamp = await Downloading;
                string installed = ThemeDownloads.DirectoryFor(stamp.Source);
                _status.Text = $"Installed {ThemeDownloads.NameOf(installed)}" + (stamp.Commit is null ? "." : $" at commit {Short(stamp.Commit)}.");
                _download = null;
                Reread();
                _browserWindow.Reshow();
                _owner.Installed(installed, first, this);
            }
            catch (Exception ex) when (ex is OperationCanceledException or ObjectDisposedException)
            {
                _status.Text = "The download was stopped; the theme there before is unchanged.";
            }
            catch (Exception ex) when (ex is HttpRequestException or IOException or InvalidDataException or UnauthorizedAccessException or InvalidOperationException)
            {
                ErrorLog.Error("themes", "A theme download failed", ex, $"{_entry.Theme.Name} ({_entry.Theme.Url})");
                _status.Text = $"The download failed, and the theme there before is unchanged: {ex.Message}";
            }
            finally
            {
                if (_download == cancel) _download = null;
                cancel.Dispose();
                Refill();
            }
        }

        private void Use()
        {
            if (_entry.Installed is { } i) _owner.UseDirectory(i.Directory);
            Refill();
        }

        private async Task RemoveAsync()
        {
            if (_entry.Installed is not { } i) return;
            if (await _owner.RemoveAsync(i.Directory, this))
            {
                Reread();
                _browserWindow.Reshow();
            }
        }
    }
}
