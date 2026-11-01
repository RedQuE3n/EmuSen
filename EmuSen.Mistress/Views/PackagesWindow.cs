using EmuSen.Tools.Packages;

namespace EmuSen.Mistress.Views
{
    // Mistress's thin host for the Packages tool - see EmuSen_LibraryTools.md §1b and EmuSen_Packages.md §7.
    public sealed class PackagesWindow : ToolPaneWindow
    {
        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public PackagesWindow() : this((string?)null) { }

        public PackagesWindow(string? library) : this(new PackagesPane { Library = library }) { }

        private PackagesWindow(PackagesPane pane) : base("Packages", pane)
        {
            Pane = pane;
            pane.MarksChanged += ShowHints;
        }

        public PackagesPane Pane { get; }

        protected override void FirstShown() => Pane.Refresh();
    }
}
