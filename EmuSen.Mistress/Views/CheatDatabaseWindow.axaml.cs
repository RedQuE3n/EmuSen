using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Models;

namespace EmuSen.Mistress.Views
{
    // The GUI for `cheat db` - see `man cheat`. EmuSen redistributes no
    // cheat data: this either indexes a folder the user already has, or
    // downloads one to their machine on their explicit request.
    public partial class CheatDatabaseWindow : Window
    {
        private readonly AppSettings _settings;

        public CheatDatabaseWindow() : this(AppSettings.Load()) { }

        public CheatDatabaseWindow(AppSettings settings)
        {
            InitializeComponent();
            _settings = settings;
            AttributionText.Text = CheatDatabaseInstaller.Attribution;
            DirectoryBox.Text = _settings.CheatDatabaseDirectory;
            Refresh();
        }

        // AppSettings when set, the sandbox's own Cheats folder otherwise -
        // the same resolution `cheat db` uses.
        private string Directory =>
            string.IsNullOrWhiteSpace(_settings.CheatDatabaseDirectory)
                ? DianaOSSandbox.CheatDatabaseDirectory
                : _settings.CheatDatabaseDirectory;

        private void Refresh()
        {
            var db = new CheatDatabase(Directory);
            IReadOnlyList<(string System, int Count)> systems = db.Systems();
            int total = systems.Sum(s => s.Count);

            SystemsList.ItemsSource = systems.Select(s => $"{s.System}  ({s.Count})").ToList();
            StatusText.Text = total > 0
                ? $"{total:N0} cheat file(s) in {Directory}"
                : $"No cheat files in {Directory}";
        }

        private async void OnBrowseClick(object? sender, RoutedEventArgs e)
        {
            IReadOnlyList<IStorageFolder> folders = await StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions
            {
                Title = "Choose a cheat folder (a RetroArch cheats folder works as-is)",
                AllowMultiple = false,
            });

            if (folders.FirstOrDefault()?.Path.LocalPath is not string picked) return;

            _settings.CheatDatabaseDirectory = picked;
            _settings.Save();
            DirectoryBox.Text = picked;
            Refresh();
        }

        private async void OnDownloadClick(object? sender, RoutedEventArgs e)
        {
            DownloadButton.IsEnabled = false;
            BrowseButton.IsEnabled = false;
            StatusText.Text = "Downloading from libretro...";

            string target = Directory;

            try
            {
                // Off the UI thread - this is a real network fetch of a
                // multi-megabyte archive.
                CheatDatabaseInstallResult result = await Task.Run(() =>
                {
                    using Stream zip = CheatDatabaseInstaller.Fetch(CheatDatabaseInstaller.LibretroCheatsUrl);
                    return CheatDatabaseInstaller.Install(zip, target);
                });

                Refresh();
                StatusText.Text = $"Installed {result.Installed:N0} cheat file(s) into {target}";
            }
            catch (Exception ex)
            {
                StatusText.Text = $"Download failed: {ex.Message}";
            }
            finally
            {
                DownloadButton.IsEnabled = true;
                BrowseButton.IsEnabled = true;
            }
        }

        private void OnCloseClick(object? sender, RoutedEventArgs e) => Close();
    }
}
