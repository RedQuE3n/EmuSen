using System;
using System.Collections.Generic;
using System.Linq;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Layout;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture;
using EmuSen.Galaxia.Library;

namespace EmuSen.Mistress.Views
{
    // ES-DE's official theme list as ES-DE's theme downloader shows it: a list, the selected theme's screenshot and what it supports - see EmuSen_Settings_Reference.md §4.62.
    public class ThemeBrowserWindow : ToolWindow
    {
        private readonly ThemeSettingsWindow _owner;
        private readonly LunaList<ThemeBrowserEntry> _list = new() { Name = "ThemeBrowserList", Height = 470, Width = 400 };
        private readonly TextBlock _fetched = new() { Name = "ThemeBrowserFetched", TextWrapping = TextWrapping.Wrap };
        private readonly TextBlock _status = new() { Name = "ThemeBrowserStatus", TextWrapping = TextWrapping.Wrap };
        private readonly FittedImage _shot = new() { Name = "ThemeBrowserScreenshot", Width = 560, Height = 315, Fit = ImageFit.Contain };
        private readonly TextBlock _caption = new() { Name = "ThemeBrowserCaption", TextWrapping = TextWrapping.Wrap, Opacity = 0.8 };
        private readonly TextBlock _name = new() { Name = "ThemeBrowserName", FontSize = 20, FontWeight = FontWeight.SemiBold, TextWrapping = TextWrapping.Wrap };
        private readonly TextBlock _author = new() { Name = "ThemeBrowserAuthor", TextWrapping = TextWrapping.Wrap };
        private readonly TextBlock _supports = new() { Name = "ThemeBrowserSupports", TextWrapping = TextWrapping.Wrap };
        private readonly TextBlock _state = new() { Name = "ThemeBrowserState", TextWrapping = TextWrapping.Wrap };
        private readonly Button _refresh, _details;
        private CancellationTokenSource? _preview;
        private ThemeDetailWindow? _detail;

        public ThemeBrowserWindow(ThemeSettingsWindow owner, Func<HttpClient> http)
        {
            _owner = owner;
            Browser = new ThemeBrowser(http);
            Title = "ES-DE Themes";
            Width = 1040;
            CanResize = false;
            SizeToContent = SizeToContent.Height;

            _list.Label = Label;
            _list.Key = e => e.Theme.Url;
            _list.Chose += e => Preview(e);
            _list.DoubleTapped += (_, _) => OpenDetail();
            _list.AddHandler(KeyDownEvent, (_, e) => { if (e.Key == Key.Enter) { e.Handled = true; OpenDetail(); } }, handledEventsToo: true);
            _refresh = Ui.Button("Refresh", () => _ = LoadAsync(refresh: true));
            _refresh.Name = "ThemeBrowserRefresh";
            _details = Ui.Button("Details…", OpenDetail);
            _details.Name = "ThemeBrowserDetails";
            _details.IsEnabled = false;

            Control preview = Ui.Stack(8, _shot, _caption, _name, _author, _supports, _state, Ui.Row(8, _details));
            var body = new Grid { ColumnDefinitions = new ColumnDefinitions("Auto,16,*") };
            body.Children.Add(_list);
            Grid.SetColumn(preview, 2);
            body.Children.Add(preview);
            Control top = Ui.Stack(6, new DockPanel { LastChildFill = true, Children = { _refresh.Dock(Dock.Right), _fetched } }, _status);
            Control buttons = Ui.Buttons(Ui.Button("Close", Close)).Margin(0, 12, 0, 0);
            DockPanel.SetDock(top, Dock.Top);
            DockPanel.SetDock(buttons, Dock.Bottom);
            var dock = new DockPanel { LastChildFill = true, Children = { top, buttons, body.Margin(0, 12, 0, 0) } }.Margin(16);
            Content = dock;
            // On a big-screen sheet: the list and the preview as two columns, the screenshot fitted in its column with its words under it, Details and Close in a row of their own (§4.83).
            MenuLook.SetWidthFraction(this, 0.85);
            MenuLook.WhenApplied(dock, () =>
            {
                dock.Margin = new Avalonia.Thickness(0, 0, 0, 12);
                body.ColumnDefinitions = new ColumnDefinitions("2*,24,3*");
                _list.Width = double.NaN;
                _list.Height = double.NaN;
                _list.VerticalAlignment = VerticalAlignment.Stretch;
                _shot.Width = double.NaN;
                _shot.Height = 190;
                _shot.HorizontalAlignment = HorizontalAlignment.Stretch;
                _name.FontSize = 28;
                body.Children.Remove(preview);
                var words = new ScrollViewer { Name = "ThemeBrowserPreview", Content = preview, HorizontalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Disabled };
                Grid.SetColumn(words, 2);
                body.Children.Add(words);
                if (preview is StackPanel stack && stack.Children[^1] is StackPanel row)
                {
                    stack.Children.Remove(row);
                    row.Children.Add(Ui.Button("Close", Close));
                    row.HorizontalAlignment = HorizontalAlignment.Center;
                    row.Margin = new Avalonia.Thickness(0, 12, 0, 0);
                    DockPanel.SetDock(row, Dock.Bottom);
                    dock.Children.Insert(dock.Children.IndexOf(buttons), row);
                }
                buttons.IsVisible = false;
            });
            Closed += (_, _) => Stop();
            ShowCached();
        }

        // Called once the sheet is shown: opening the browser is the player's action that may fetch the list.
        public void Start() => Loading ??= LoadAsync(refresh: false);

        public ThemeBrowser Browser { get; }

        // The list's load in flight or done, for a test to await.
        public Task? Loading { get; private set; }

        // The screenshot fetch of the last preview, for a test to await.
        public Task? PreviewLoading { get; private set; }

        public ThemeDetailWindow? Detail => _detail;

        public IReadOnlyList<ThemeBrowserEntry> Entries => _list.Models;

        // Closing, or the window under it closing, stops every request and download the browser started.
        public void Stop()
        {
            _preview?.Cancel();
            _detail?.StopDownload();
            Browser.Dispose();
        }

        public static string StateText(ThemeBrowserEntry e)
        {
            ThemeListState s = e.State;
            if (s == ThemeListState.NotInstalled) return "Not installed";
            var words = new List<string> { "Installed" };
            if (s.HasFlag(ThemeListState.UpdateAvailable)) words.Add("update available");
            if (s.HasFlag(ThemeListState.LocalChanges)) words.Add("local changes");
            return string.Join(", ", words);
        }

        private static string Label(ThemeBrowserEntry e) =>
            e.Theme.Name + (e.State == ThemeListState.NotInstalled ? "" : "   ·   " + StateText(e)) + (e.Theme.NewEntry ? "   ·   New" : "");

        public static string Counts(ThemeListEntry t) =>
            $"{Count(t.Variants.Count, "variant")} · {Count(t.ColorSchemes.Count, "colour scheme")} · {Count(t.AspectRatios.Count, "aspect ratio")}";

        private static string Count(int n, string what) => n == 0 ? $"no {what}s stated" : n == 1 ? $"1 {what}" : $"{n} {what}s";

        // What themes.db already holds is shown at once, with no request; opening then asks only when it is older than a day.
        private void ShowCached()
        {
            if (ThemeBrowser.Cached() is { } list) Show(list, ThemeBrowser.Entries(list));
            else _fetched.Text = "ES-DE's theme list has not been fetched yet.";
        }

        private void Show(ThemeList list, IReadOnlyList<ThemeBrowserEntry> entries)
        {
            ThemeBrowserEntry? was = _list.Selected;
            _list.Refresh(entries);
            _list.Select(entries.FirstOrDefault(e => e.Theme.Url == was?.Theme.Url) ?? entries.FirstOrDefault());
            _fetched.Text = $"ES-DE's theme list: {list.Themes.Count} themes, fetched {list.FetchedAt.ToLocalTime():yyyy-MM-dd HH:mm}" +
                            (list.Skipped.Count > 0 ? $"; {list.Skipped.Count} left out (not on GitHub or GitLab)" : "") + ".";
            if (_list.Selected is { } s) Preview(s);
        }

        // Called on opening and by Refresh: the list, then the newest commit of each installed theme for its Update mark.
        public async Task LoadAsync(bool refresh)
        {
            if (Browser.Closed) return;
            _refresh.IsEnabled = false;
            _status.Text = refresh ? "Fetching ES-DE's theme list…" : "Opening ES-DE's theme list…";
            try
            {
                ThemeList list = await Browser.ListAsync(refresh);
                Show(list, ThemeBrowser.Entries(list));
                var installed = ThemeBrowser.Entries(list).Where(e => e.Installed is not null).ToList();
                if (installed.Count > 0) _status.Text = $"Asking for updates to {installed.Count} installed theme{(installed.Count == 1 ? "" : "s")}…";
                foreach (ThemeBrowserEntry e in installed) await Browser.HeadAsync(e.Theme, e.Installed!, refresh);
                Show(list, ThemeBrowser.Entries(list));
                _status.Text = "";
            }
            catch (OperationCanceledException) when (Browser.Closed) { }
            catch (ObjectDisposedException) when (Browser.Closed) { }
            catch (Exception ex) when (ex is HttpRequestException or TaskCanceledException or System.Text.Json.JsonException or System.IO.IOException)
            {
                ErrorLog.Error("themes", "ES-DE's theme list could not be fetched", ex);
                _status.Text = $"ES-DE's theme list could not be fetched: {ex.Message}" + (ThemeBrowser.Cached() is not null ? " The copy shown is the last one fetched." : "");
            }
            finally
            {
                if (!Browser.Closed) _refresh.IsEnabled = true;
            }
        }

        // The theme under the selection; its first screenshot is fetched only once it has stayed selected a moment, so a held list does not fetch every row.
        private void Preview(ThemeBrowserEntry? e)
        {
            _preview?.Cancel();
            _details.IsEnabled = e is not null;
            if (e is null) return;
            ThemeListEntry t = e.Theme;
            _name.Text = t.Name;
            _author.Text = $"by {t.Author}   ·   {t.Source?.Url.Replace("https://", "")}";
            _supports.Text = Counts(t);
            _state.Text = StateText(e) + (e.Remote?.HeadDate is { } d ? $"   ·   last updated {d.ToLocalTime():yyyy-MM-dd}" : "");
            _shot.Source = null;
            _caption.Text = t.Screenshots.Count == 0 ? "The list gives no screenshot." : "";
            if (t.Screenshots.Count == 0 || Browser.Closed) return;
            var cancel = _preview = new CancellationTokenSource();
            PreviewLoading = ShowShotAsync(t.Screenshots[0], cancel.Token);
        }

        public static TimeSpan PreviewDelay { get; set; } = TimeSpan.FromMilliseconds(250);

        private async Task ShowShotAsync(ThemeScreenshot shot, CancellationToken cancel)
        {
            try
            {
                await Task.Delay(PreviewDelay, cancel);
                string? file = await Browser.ScreenshotAsync(shot, cancel);
                if (cancel.IsCancellationRequested) return;
                _shot.Source = file;
                _caption.Text = file is null ? "The screenshot could not be fetched." : shot.Caption ?? "";
            }
            catch (Exception ex) when (ex is OperationCanceledException or ObjectDisposedException) { }
            catch (Exception ex) when (ex is HttpRequestException or System.IO.IOException or System.IO.InvalidDataException)
            {
                ErrorLog.Error("themes", "A theme screenshot could not be fetched", ex, shot.Image);
                if (!cancel.IsCancellationRequested) _caption.Text = $"The screenshot could not be fetched: {ex.Message}";
            }
        }

        private void OpenDetail()
        {
            if (_list.Selected is not { } e || Browser.Closed) return;
            var detail = _detail = new ThemeDetailWindow(this, _owner, e);
            detail.Closed += (_, _) =>
            {
                if (_detail == detail) _detail = null;
                Reshow();
            };
            _ = SheetLayer.Show(detail, this);
        }

        // After a download, update or removal the rows are read again from themes.db and the folders, with no request.
        public void Reshow()
        {
            if (!Browser.Closed && ThemeBrowser.Cached() is { } list) Show(list, ThemeBrowser.Entries(list));
        }
    }
}
