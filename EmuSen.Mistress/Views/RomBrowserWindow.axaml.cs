using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Views
{
    // Lists ROM files from a configured directory (AppSettings.RomDirectory,
    // see PreferencesWindow) as a quicker alternative to the OS file picker
    // for anyone repeatedly loading ROMs from the same test folder - the
    // OS picker (MainWindow's Open ROM...) still exists unchanged for
    // anyone who prefers it, or for a ROM outside that folder.
    //
    // Returned as this window's ShowDialog<string?> result (via
    // Close(path)) rather than an event/property MainWindow reads
    // afterward - the idiomatic Avalonia modal-dialog shape, and avoids
    // MainWindow needing to know anything about this window's internal
    // state beyond "what did the user pick, if anything."
    public partial class RomBrowserWindow : Window
    {
        private IReadOnlyList<RomEntry> _entries = Array.Empty<RomEntry>();

        // Parameterless constructor exists only so Avalonia's XAML tooling
        // (previewer, generated InitializeComponent) is happy - always use
        // the full constructor in real code (see MainWindow's menu handler).
        public RomBrowserWindow() : this(null) { }

        public RomBrowserWindow(string? romDirectory)
        {
            InitializeComponent();
            PopulateList(romDirectory);
        }

        private void PopulateList(string? romDirectory)
        {
            RomLibraryResult result = RomLibrary.Scan(romDirectory);
            _entries = result.Entries;

            DirectoryText.Text = _entries.Count > 0 ? result.Directory : RomLibrary.DescribeEmpty(result);
            RomList.ItemsSource = _entries.Select(e => e.FileName).ToList();
        }

        private void OnOpenClick(object? sender, RoutedEventArgs e) => TryReturnSelection();

        private void OnRomListDoubleTapped(object? sender, TappedEventArgs e) => TryReturnSelection();

        private void OnCancelClick(object? sender, RoutedEventArgs e) => Close(null);

        private void TryReturnSelection()
        {
            int index = RomList.SelectedIndex;
            if (index < 0 || index >= _entries.Count) return;
            Close(_entries[index].FullPath);
        }
    }
}
