using System;
using System.Linq;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Pages;

namespace EmuSen.Mistress.Views
{
    // The thin view of a game's page: the model's records drawn by LunaP's PageLayout, pictures loaded by the host's cache - see EmuSen_BigPicture.md §50.
    public sealed class GamePageView
    {
        private readonly Func<string, IImage?> _picture;
        private string? _section, _deeper;
        private bool _drawing;

        public GamePageView(GamePage model, PageLayout layout, Func<string, IImage?> picture)
        {
            Model = model;
            Layout = layout;
            _picture = picture;
            layout.HintsFor = model.Hints;
            layout.Back = model.Back;
            layout.ActionInvoked += model.Act;
            layout.ActionStepped += model.StepAction;
            layout.SectionChosen += model.ChooseSection;
            layout.RowChosen += model.ChooseRow;
            layout.RowStepped += model.StepRow;
            layout.RowEdited += model.EditRow;
            layout.TileChosen += model.ChooseTile;
            layout.TileButton += model.TileButton;
            layout.ContentButton += model.ContentButton;
            layout.HintInvoked += model.Hint;
            layout.ViewerStepped += model.StepViewer;
            model.Changed += Draw;
            Draw();
        }

        public GamePage Model { get; }

        public PageLayout Layout { get; }

        public void Detach()
        {
            Model.Changed -= Draw;
            Layout.HintsFor = null;
            Layout.Back = null;
        }

        private void Draw()
        {
            if (_drawing) return;
            _drawing = true;
            try
            {
                GamePage m = Model;
                Layout.Hero = new PageHero(m.Facts.Title, m.FactsLine, Load(m.Art.Hero), Load(m.Art.Cover), GameFacts.CardShape(m.Facts.CoverAspect), m.Facts.Mark);
                Layout.Poster.Video = m.Art.Clip;
                Layout.SetActions(m.Actions);
                PageContent content = Map(m.Content);
                if (m.Trail is { } trail)
                {
                    bool fresh = _deeper != trail.Title;
                    _deeper = trail.Title;
                    if (fresh) Layout.ShowDeeper(trail.Parent, trail.Title, content);
                    else Layout.Content = content;
                }
                else
                {
                    bool left = _deeper is not null, moved = _section != m.Section;
                    _deeper = null;
                    if (left || moved || Layout.Sections.Count == 0) Layout.SetSections(m.SectionList(), m.Section);
                    _section = m.Section;
                    Layout.Content = content;
                    if (left && m.ReturnedFrom is { } from) Layout.FocusContent(RowFor(from));
                    if (moved && !Layout.Focused.InActions && !Layout.FocusContent()) Layout.FocusActions();
                }
                if (m.Viewer is { } viewer) Layout.ShowViewer(Load(viewer.Picture), viewer.Caption);
                else Layout.CloseViewer();
            }
            finally { _drawing = false; }
        }

        // The row a deeper page was opened from, focused again when B leaves it.
        private static string? RowFor(string page) => page switch
        {
            GamePage.CheatsPage => "cheats",
            GamePage.PackagePage => "package",
            GamePage.EditPage => "edit",
            GamePage.CoverPage => "cover",
            GamePage.ScrapePage => "scrape",
            _ => null,
        };

        private IImage? Load(string? path) => path is null ? null : _picture(path);

        private PageContent Map(GameContent content) => content switch
        {
            GameContent.Rows r => r.Value,
            GameContent.Tiles t => new PageContent.Tiles(t.Items.Select(i => new PageTile(i.Key, i.Label)
            {
                Status = i.Status, Picture = Load(i.Picture), Caption = i.Caption, Mark = i.Mark, IsEmpty = i.IsEmpty, IsMarked = i.IsMarked, Buttons = i.Buttons,
            }).ToList(), t.Width, t.Height),
            GameContent.Text x => new PageContent.Text(x.Words, Load(x.Picture), x.Caption),
            _ => throw new ArgumentOutOfRangeException(nameof(content)),
        };
    }
}
