using EmuSen.Galaxia.Native;
using System;
using System.IO;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Media;
using EmuSen.Galaxia.Library;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;

namespace EmuSen.Tools.Packages
{
    // The chosen game's detail: its package and game, state, sizes and what the last run did, with Open Folder, then the game's package controls shared with the Game Package window - see EmuSen_Packages.md §7 and §8.
    public sealed class PackageDetail : StackPanel
    {
        public PackageDetail(PackageRow row, string? outcome, string? library = null, bool browse = true, Action? changed = null)
        {
            Name = "PackagesDetail";
            Spacing = 10;
            Row = row;
            Add("Package", row.Package is null ? "Not a package yet. Making one keeps everything EmuSen holds for the game in one folder, and the library is only read."
                : row.Mode == "self-contained" ? $"Packages/{row.Package}, self-contained: the game is kept in the package."
                : $"Packages/{row.Package}, linked to the game where it is.");
            Add("Game", row.Source is { } source ? $"{source} in the library ({row.Console})" : row.Where == "package" ? $"In its package ({row.Console})" : row.Console);
            Add("State", row.Words is { Length: > 0 } words ? $"{PackagesWords.State(row)}: {ToolWords.Reason(words)}" : PackagesWords.State(row));
            if (outcome is not null) Add("Last run", outcome);
            string sizes = row.Size > 0 ? $"the game {ToolWords.Size(row.Size)}" : "";
            if (row.Packaged) sizes += (sizes.Length > 0 ? "; " : "") + $"the package's folder {ToolWords.Size(row.PackageSize)}";
            if (sizes.Length > 0) Add("Size", char.ToUpperInvariant(sizes[0]) + sizes[1..]);
            if (row.Format.Length > 0) Add("Format", row.Convertible ? $"{row.Format}, which can be converted to CHD" : row.Format);
            if (row.Package is not null)
            {
                Button open = Ui.Button("Open Folder", OpenFolder);
                open.Name = "PackagesOpenFolder";
                Children.Add(Ui.Buttons(open));
            }
            if (GameOf(row, library) is not { } game) return;
            string? package = row.Package is null ? null : Path.Combine(DataStore.Packages, row.Package.Replace('/', Path.DirectorySeparatorChar));
            Game = new GamePackageDetail(row.Title, game, library, package, browse, refused: row.State == "changed");
            if (changed is not null) Game.PackageChanged += changed;
            Children.Add(Game);
        }

        public PackageRow Row { get; }

        // The game's package controls, null where the game's file cannot be named: a CHD in Compressed or a file outside the library.
        public GamePackageDetail? Game { get; }

        // The game's file: in the library, or kept in its package.
        private static string? GameOf(PackageRow row, string? library)
        {
            if (row.Source is { } source && !string.IsNullOrWhiteSpace(library)) return Path.Combine(library, source.Replace('/', Path.DirectorySeparatorChar));
            if (row.Where == "package" && row.Package is { } package && row.File.Length > 0) return Path.Combine(DataStore.Packages, package.Replace('/', Path.DirectorySeparatorChar), row.File.Replace('/', Path.DirectorySeparatorChar));
            return null;
        }

        private void Add(string label, string text) =>
            Children.Add(new FieldRow { Label = label, Content = new TextBlock { Text = text, TextWrapping = TextWrapping.Wrap } });

        // The package's folder shown by the platform's own file manager, through Avalonia's launcher.
        private void OpenFolder()
        {
            string folder = Path.Combine(DataStore.Packages, Row.Package!.Replace('/', Path.DirectorySeparatorChar));
            if (Directory.Exists(folder)) _ = TopLevel.GetTopLevel(this)?.Launcher.LaunchUriAsync(new Uri(folder));
        }
    }
}
