using System.Collections.Generic;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;
using EmuSen.TestingStudio.Settings;

namespace EmuSen.TestingStudio.Views
{
    // Log directory / ROM directory / core selection - see AppSettings for
    // the persisted shape. Saves immediately on every change (same pattern
    // InputSettingsWindow uses for rebinds) rather than needing a separate
    // Save button - there's nothing here a user would want to stage and
    // discard.
    public partial class PreferencesWindow : Window
    {
        // Only entry today - see AppSettings.SelectedCore's own comment on
        // why this exists at all despite doing nothing yet.
        private static readonly string[] AvailableCores = { "SNES (Venus)" };

        private readonly AppSettings _settings;
        private bool _initializing;

        // Parameterless constructor exists only so Avalonia's XAML tooling
        // (previewer, generated InitializeComponent) is happy - always use
        // the full constructor in real code (see MainWindow's menu handler).
        public PreferencesWindow() : this(new AppSettings()) { }

        public PreferencesWindow(AppSettings settings)
        {
            InitializeComponent();
            _settings = settings;

            _initializing = true;
            LogDirectoryBox.Text = _settings.LogDirectory;
            RomDirectoryBox.Text = _settings.RomDirectory;
            CoreComboBox.ItemsSource = AvailableCores;
            CoreComboBox.SelectedIndex = System.Math.Max(0, System.Array.IndexOf(AvailableCores, _settings.SelectedCore));
            _initializing = false;
        }

        private async void OnBrowseLogDirectoryClick(object? sender, RoutedEventArgs e)
        {
            string? picked = await PickFolder("Choose Log Directory");
            if (picked is null) return;

            _settings.LogDirectory = picked;
            _settings.Save();
            LogDirectoryBox.Text = picked;
        }

        private async void OnBrowseRomDirectoryClick(object? sender, RoutedEventArgs e)
        {
            string? picked = await PickFolder("Choose ROM Directory");
            if (picked is null) return;

            _settings.RomDirectory = picked;
            _settings.Save();
            RomDirectoryBox.Text = picked;
        }

        private async System.Threading.Tasks.Task<string?> PickFolder(string title)
        {
            IReadOnlyList<IStorageFolder> folders = await StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions
            {
                Title = title,
                AllowMultiple = false,
            });

            return folders.FirstOrDefault()?.Path.LocalPath;
        }

        private void OnCoreSelectionChanged(object? sender, SelectionChangedEventArgs e)
        {
            // Fires once as a side effect of setting ItemsSource/SelectedIndex
            // in the constructor above, purely reflecting the value already
            // just loaded from _settings - skip that spurious first event so
            // this doesn't immediately re-save the exact value it just read.
            if (_initializing) return;
            if (CoreComboBox.SelectedItem is not string selected) return;

            _settings.SelectedCore = selected;
            _settings.Save();
        }

        private void OnCloseClick(object? sender, RoutedEventArgs e)
        {
            Close();
        }
    }
}
