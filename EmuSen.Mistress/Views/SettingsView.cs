using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Pages;

namespace EmuSen.Mistress.Views
{
    // Settings' plain description drawn on LunaP's SettingsLayout, and the layout's presses handed back to it - see EmuSen_BigPicture.md §52.
    public sealed class SettingsView
    {
        private readonly Func<IImage?> _frame;
        private int _depth;

        public SettingsView(SettingsScreen model, SettingsLayout layout, Func<IImage?> frame)
        {
            Model = model;
            Layout = layout;
            _frame = frame;
            layout.SectionChosen += key => { Model.ChooseSection(key); Fill(); };
            layout.RowStepped += (key, by) => { Model.Step(key, by); Fill(); };
            layout.RowChosen += key => { Model.Choose(key); Fill(); };
            Fill();
        }

        public SettingsScreen Model { get; }

        public SettingsLayout Layout { get; }

        // B: a page closes back to its row; false when nothing is left to close.
        public bool Back()
        {
            if (!Model.Back()) return false;
            Fill();
            return true;
        }

        // Draws the screen from the description: a page just opened starts on its first row, a page closed returns to the row it came from.
        public void Fill()
        {
            int depth = Model.Trail.Count;
            string? focus = depth > _depth ? Model.Rows.FirstOrDefault()?.Key : depth < _depth ? Model.Returned : null;
            if (depth == 0 && Layout.SelectedSection != Model.Section) focus = Model.Rows.FirstOrDefault()?.Key;
            _depth = depth;
            Layout.SetSections(SettingsScreen.SectionList.Select(s => new AdaptiveSection(s.Key, s.Label)), Model.Section);
            Layout.SetTrail(Model.Trail);
            Layout.PaneFor = key => Pane(Model.PaneOf(key));
            Layout.HintsFor = key => Model.HintsFor(key).Select(h => new PageHint(Enum.Parse<PadGlyphButton>(h.Button), h.Text, h.Key)).ToList();
            Layout.SetRows(Model.Rows, focus);
        }

        private SettingsPane? Pane(ScreenPane? pane) => pane is null ? null : new SettingsPane(pane.Lines)
        {
            Preview = pane.Preview switch
            {
                PanePreview.Frame f => new SettingsPicture(_frame(), f.Caption),
                PanePreview.Meter m => new SettingsMeter(m.Fraction),
                PanePreview.Variants v => new SettingsVariants(v.Dark, v.Light),
                _ => null,
            },
        };
    }
}
