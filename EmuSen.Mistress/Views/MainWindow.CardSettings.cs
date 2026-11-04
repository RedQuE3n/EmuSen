using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Pages;

namespace EmuSen.Mistress.Views
{
    // Settings' description drawn as pages of the Pause Card: each row of a page a row of the card, its pane the card's pane - see EmuSen_Interface.md §18.5.
    public partial class MainWindow
    {
        // A game page of Settings' description opened as the next page of the card.
        private void OpenSettingsOnCard(string page, string accept, string side)
        {
            var model = new SettingsScreen(this);
            model.OpenGame(page);
            if (model.Trail.Count == 0) return;
            EnterPadMenuPage(SettingsCardPage(model, accept, side), Math.Max(PadMenuList.SelectedIndex, 0), key: null);
        }

        // The model's page now shown, its title its own; B goes back a page in the model as on the card.
        private PadMenuPage SettingsCardPage(SettingsScreen model, string accept, string side) =>
            new(model.Trail[^1], () => SettingsEntries(model, accept, side))
            {
                AcceptWord = accept, SideWord = model.Rows.Any(r => r.Kind == PageRowKind.Steps) ? side : null, StartResumes = false,
                Back = () => { model.Back(); return false; },
            };

        private List<PadMenuEntry> SettingsEntries(SettingsScreen model, string accept, string side)
        {
            model.Read();
            return model.Rows.Select(row =>
            {
                string key = row.Key;
                PageRow Now() => model.Rows.FirstOrDefault(r => r.Key == key) ?? row;
                return new PadMenuEntry(() => Now().Label, () => ChooseOnCard(model, key, accept, side), row.Kind == PageRowKind.Steps ? by => model.Step(key, by) : null, closes: false)
                {
                    Label = () => Now().Label, Value = () => Now().Status, Opens = row.Kind == PageRowKind.Opens, Dimmed = !row.IsEnabled,
                    PaneScreen = () => model.PaneOf(key),
                };
            }).ToList();
        }

        // A on a row: a page it opens turns the card's page; anything else acts and the rows are read again where they stand.
        private void ChooseOnCard(SettingsScreen model, string key, string accept, string side)
        {
            int depth = model.Trail.Count, at = Math.Max(PadMenuList.SelectedIndex, 0);
            model.Choose(key);
            if (!_padMenuOpen) return;
            if (model.Trail.Count > depth) EnterPadMenuPage(SettingsCardPage(model, accept, side), at, key: null);
            else if (_padMenuPages.Count > 0)
            {
                _padMenuPages[^1].At = at;
                ShowPadMenuPage();
            }
        }
    }
}
