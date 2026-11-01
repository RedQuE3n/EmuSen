using EmuSen.Tools.Scraper;

namespace EmuSen.Mistress.Views
{
    // Mistress's thin host for the Scraper tool; while it is open a run's progress is shown in it, not in the status sheet - see EmuSen_LibraryTools.md §2 and §9.
    public sealed class ScraperWindow : ToolPaneWindow
    {
        public ScraperWindow() : this(new ScraperPane()) { }

        private ScraperWindow(ScraperPane pane) : base("Scraper", pane)
        {
            Pane = pane;
            pane.MarksChanged += ShowHints;
        }

        public ScraperPane Pane { get; }

        protected override void FirstShown() => _ = Pane.Refresh();
    }
}
