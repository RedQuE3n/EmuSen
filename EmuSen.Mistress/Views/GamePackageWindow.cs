using System.Collections.Generic;
using Avalonia;
using Avalonia.Controls;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Tools.Packages;

namespace EmuSen.Mistress.Views
{
    // One game's package from its menu, the game options sheet and the pad: a thin host of Packages' GamePackageDetail, the same control Packages' detail shows - see EmuSen_Packages.md §8 and EmuSen_GamePackages.md §15, §19.
    public sealed class GamePackageWindow : ToolWindow
    {
        public static readonly string[] PatchExtensions = GamePackageDetail.PatchExtensions;

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public GamePackageWindow() : this("", "", null, null, browse: true) { }

        // refused: the package claims the game but will not open with the file as it is now, most often another dump, so the game plays as a plain one - see EmuSen_GamePackages.md §14.8 Q34.
        public GamePackageWindow(string title, string game, string? library, string? package, bool browse, bool refused = false)
        {
            Title = "Game Package";
            Width = 640;
            Height = 560;
            MinWidth = 480;
            MinHeight = 380;
            CanResize = true;
            Detail = new GamePackageDetail(title, game, library, package, browse, refused) { Margin = new Thickness(0, 0, 4, 0) };

            Button close = Ui.Button("Close", Close);
            close.Name = "GamePackageClose";
            Control foot = Ui.Buttons(close);
            foot.Margin = new Thickness(0, 10, 0, 0);
            DockPanel.SetDock(foot, Dock.Bottom);
            var dock = new DockPanel { Margin = new Thickness(16), LastChildFill = true };
            dock.Children.Add(foot);
            dock.Children.Add(new ScrollViewer { Content = Detail, HorizontalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Disabled });
            Content = dock;
            Closed += (_, _) => Detail.Stop();
        }

        public GamePackageDetail Detail { get; }

        public string? Package => Detail.Package;

        public bool Refused => Detail.Refused;

        public bool Changed => Detail.Changed;

        public bool Busy => Detail.Busy;

        public string? Status => Detail.Status;

        public bool SelfContained => Detail.SelfContained;

        public string Identity => Detail.Identity;

        public IReadOnlyList<string> Beside() => Detail.Beside();

        public void Add(string file) => Detail.Add(file);

        public void Remove(int n) => Detail.Remove(n);

        public void SetIdentity(string identity) => Detail.SetIdentity(identity);

        public void Poll() => Detail.Poll();

        public void Apply(string line) => Detail.Apply(line);

        public IReadOnlyList<(string File, string Format)> Patches() => Detail.Patches();
    }
}
