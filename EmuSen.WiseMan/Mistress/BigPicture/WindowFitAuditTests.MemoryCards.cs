using System;
using System.IO;
using System.Linq;
using Avalonia.Controls;
using Avalonia.VisualTree;
using EmuSen.Crystal.Systems.Psx;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Crystal;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The Memory Cards window on the PlayStation's tab, a shared card with saves and a game's own beside it, and the same card held by a running game - see EmuSen_SaveMedia.md §6.5.
    public partial class WindowFitAuditTests
    {
        internal static void ShowMemoryCards(MainWindow window, bool held)
        {
            var graphics = (GraphicsConfig)typeof(MainWindow).GetField("_graphics", Hidden)!.GetValue(window)!;
            graphics.SetValue("PlayStation", EmuSen.Cores.MemoryCards.ModeKey, EmuSen.Cores.MemoryCards.Shared);
            string card = SaveLibrary.CardPathFor("PlayStation", "Card 1", ".mcr");
            Directory.CreateDirectory(Path.GetDirectoryName(card)!);
            byte[] saves = SaveMediaManagerTests.WithSave(SaveMediaManagerTests.WithSave(PsxCardFormat.Instance.Format(), "BASLUS-00067DRACULA", "ＤＲＡＣＵＬＡ　Ｘ　ＣＨＲＯＮＩＣＬＥ　ＳＡＶＥ", new[] { 0 }, 1), "BESCES-00344TOMB", "Tomb Raider II, Venice, a long title", new[] { 1, 2, 3 }, 2);
            File.WriteAllBytes(card, saves);
            File.WriteAllBytes(Path.Combine(DataStore.Saves, "PlayStation", "Amber Saga.mcr"), PsxCardFormat.Instance.Format());
            if (held && MediumLock.TryTake(card, "Amber Saga", shared: true) is { } taken) window.Closed += (_, _) => taken.Dispose();
            Call(window, "ShowMemoryCards");
            SelectTab(window, "PlayStation");
            var shown = Sheets(window).SheetOf(Sheets(window).Current!)!.GetVisualDescendants().OfType<Control>().ToList();
            Assert.Contains(shown, c => c.Name == "PlayStation.psx.card.Save.1" && c.IsEffectivelyVisible);
            var about = shown.OfType<TextBlock>().First(t => t.Name == "PlayStation.psx.card.About");
            Assert.Equal(held, about.Text!.Contains("In use by Amber Saga"));
            Assert.StartsWith("Shared: Card 1 in Slot 1, Card 2 in Slot 2.", about.Text);
        }
    }
}
