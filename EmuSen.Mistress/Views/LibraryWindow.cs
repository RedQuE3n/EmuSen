using EmuSen.Tools.Library;

namespace EmuSen.Mistress.Views
{
    // Mistress's thin host for Library, the library manager - see EmuSen_LibraryTools.md §1b and §10.
    public sealed class LibraryWindow : ToolPaneWindow
    {
        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public LibraryWindow() : this(null, null) { }

        public LibraryWindow(string? library, ILibraryRecords? records) : this(new LibraryPane { Library = library, Records = records }) { }

        private LibraryWindow(LibraryPane pane) : base("Library Manager", pane)
        {
            Pane = pane;
            pane.MarksChanged += ShowHints;
        }

        public LibraryPane Pane { get; }

        protected override void FirstShown() => Pane.Refresh();
    }
}
