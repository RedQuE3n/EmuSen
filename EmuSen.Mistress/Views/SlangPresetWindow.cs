using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Layout;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Views
{
    // Picks one of RetroArch's presets for a console, and downloads libretro's pack when the player asks - see EmuSen_Settings_Reference.md §4.41.
    public class SlangPresetWindow : ToolWindow
    {
        public const string AllFolders = "All";

        private readonly Func<HttpClient> _http;
        private readonly string _pack;
        private readonly Action<string> _chose;
        private readonly FilterBar _filter = new() { Name = "PresetFilter", Placeholder = "Search presets", FacetLabel = "Folder", ShowFacet = true, SearchDelay = TimeSpan.Zero };
        private readonly LunaList<string> _list = new() { Name = "PresetList" };
        private readonly TextBlock _status = new() { Name = "PackStatus", TextWrapping = Avalonia.Media.TextWrapping.Wrap };
        private readonly Button _download;
        private readonly Button _use;
        private string[] _presets = Array.Empty<string>();

        // Tells a test when the download it started has finished.
        public Task? Downloading { get; private set; }

        public SlangPresetWindow() : this(() => new HttpClient(), SlangPackDownload.DefaultDirectory, null, _ => { }) { }

        public SlangPresetWindow(Func<HttpClient> http, string pack, string? current, Action<string> chose, string? console = null)
        {
            _http = http;
            _pack = pack;
            _chose = chose;

            Title = console is null ? "RetroArch Shaders" : $"RetroArch Shaders for {console}";
            Width = 640;
            Height = 560;
            MinWidth = 480;
            MinHeight = 360;
            CanResize = true;
            ClosesOnEscape = true;

            _download = Ui.Button("Download Pack", () => Downloading = DownloadAsync());
            _download.Name = "DownloadPack";
            _use = Ui.Button("Use This Preset", Use);
            _use.Name = "UsePreset";

            _filter.Changed += ShowPresets;
            _list.Chose += _ => _use.IsEnabled = _list.Selected is not null;
            _list.DoubleTapped += (_, _) => Use();
            // Handled events too: the list claims Enter itself, so a plain KeyDown never heard it - see EmuSen_Settings_Reference.md §4.45.3.
            _list.AddHandler(KeyDownEvent, (_, e) => { if (e.Key == Key.Enter) Use(); }, handledEventsToo: true);

            Control hint = Ui.Hint("libretro's slang shader pack, the one RetroArch's online updater fetches, from buildbot.libretro.com. EmuSen ships none of it; it is downloaded to this machine only when you ask, and each shader keeps the licence its authors gave it.");
            Control top = Ui.Stack(8, hint, Ui.Row(12, _download, _status), _filter);
            Control buttons = Ui.Buttons(_use, Ui.Button("Cancel", Close));
            top.Margin = new Avalonia.Thickness(0, 0, 0, 10);
            buttons.Margin = new Avalonia.Thickness(0, 10, 0, 0);
            DockPanel.SetDock(top, Dock.Top);
            DockPanel.SetDock(buttons, Dock.Bottom);
            var dock = new DockPanel { Margin = new Avalonia.Thickness(16), LastChildFill = true };
            dock.Children.Add(top);
            dock.Children.Add(buttons);
            dock.Children.Add(_list);
            Content = dock;

            Load(current);
        }

        private static string Folder(string preset) => preset.Contains('/') ? preset[..preset.IndexOf('/')] : AllFolders;

        private void Load(string? select)
        {
            _presets = SlangPackDownload.Presets(_pack);
            string? built = SlangPackDownload.Installed(_pack);
            _status.Text = built is null ? "No pack downloaded yet." : $"{_presets.Length:N0} presets, built {built}.";
            _download.Content = built is null ? "Download Pack" : "Update Pack";

            var folders = new List<string> { AllFolders };
            folders.AddRange(_presets.Select(Folder).Where(f => f != AllFolders).Distinct(StringComparer.OrdinalIgnoreCase));
            string? facet = select is null ? AllFolders : Folder(select);
            _filter.SetFacets(folders, folders.Contains(facet) ? facet : AllFolders);
            ShowPresets();
            if (select is not null && _presets.Contains(select)) _list.Select(select);
            _use.IsEnabled = _list.Selected is not null;
        }

        private void ShowPresets()
        {
            string folder = _filter.Facet as string ?? AllFolders;
            _list.Refresh(_presets.Where(p => (folder == AllFolders || Folder(p) == folder) && FilterBar.Matches(_filter.SearchText, p)));
            _use.IsEnabled = _list.Selected is not null;
        }

        private void Use()
        {
            if (_list.Selected is not string preset) return;
            _chose(preset);
            Close();
        }

        private async Task DownloadAsync()
        {
            _download.IsEnabled = false;
            _status.Text = "Downloading libretro's slang pack...";
            var progress = new Progress<(long Read, long? Total)>(p => _status.Text = p.Total is long total
                ? $"Downloading... {p.Read / 1048576.0:F1} of {total / 1048576.0:F1} MB"
                : $"Downloading... {p.Read / 1048576.0:F1} MB");
            try
            {
                // Its own client with a long timeout: 54 MB outlasts the thirty seconds a cover lookup is given.
                using HttpClient http = _http();
                http.Timeout = TimeSpan.FromMinutes(30);
                await SlangPackDownload.FetchAsync(http, _pack, progress);
                Load(_list.Selected);
            }
            catch (Exception ex) when (ex is HttpRequestException or IOException or InvalidDataException or TaskCanceledException or UnauthorizedAccessException)
            {
                _status.Text = $"Could not download the pack: {ex.Message}";
            }
            finally
            {
                _download.IsEnabled = true;
            }
        }
    }
}
