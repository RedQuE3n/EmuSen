using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia.Controls;
using Avalonia.VisualTree;
using EmuSen.Crystal.Systems.N64;
using EmuSen.Crystal.Systems.Ps2;
using EmuSen.Crystal.Systems.Psx;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Crystal;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The Memory Cards window on the PlayStation's tab, a shared card with saves and a game's own beside it, the same card held by a running game, and the other consoles' tabs - see EmuSen_SaveMedia.md §6.5.
    public partial class WindowFitAuditTests
    {
        internal static void ShowMemoryCards(MainWindow window, bool held)
        {
            var graphics = ConsolesConfig.Load();
            graphics.SetValue("PlayStation", EmuSen.Cores.MemoryCards.ModeKey, EmuSen.Cores.MemoryCards.Shared);
            graphics.Save();
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

        // Another console's tab, each of its media with a formatted card in the drawer under the name its first slot holds.
        internal static void ShowMemoryCardsTab(MainWindow window, string console)
        {
            Assert.NotEmpty(EmuSen.Cores.MemoryCards.MediaFor(console));
            foreach (EmuSen.Cores.SaveMedium medium in EmuSen.Cores.MemoryCards.MediaFor(console))
                if (medium.Entry?.Format?.Format() is { } blank)
                {
                    string card = medium.PathOf(medium.DefaultName(0));
                    Directory.CreateDirectory(Path.GetDirectoryName(card)!);
                    File.WriteAllBytes(card, blank);
                }
            Call(window, "ShowMemoryCards");
            SelectTab(window, console);
            var shown = Sheets(window).SheetOf(Sheets(window).Current!)!.GetVisualDescendants().OfType<TextBlock>().ToList();
            Assert.Contains(shown, t => t.Name?.StartsWith(console + ".", StringComparison.Ordinal) == true && t.Name.EndsWith(".About", StringComparison.Ordinal) && t.IsEffectivelyVisible);
        }

        // The PlayStation's Insert Disc chooser over a running console: its library games, one with a long title, or none - see Diamond_Shell.md §1.5.
        internal static void ShowInsertDisc(MainWindow window, bool empty)
        {
            IReadOnlyList<(string Title, string Path)> discs = empty ? [] :
            [
                ("Amber Saga", "/games/Amber Saga.cue"),
                ("Basalt Run - The Long Way Round Through the Mountains of the North (Disc 2 of 3)", "/games/Basalt Run (Disc 2).cue"),
                ("Cobalt Harbor", "/games/Cobalt Harbor.chd"),
            ];
            Call(window, "PresentInsertDisc", "PlayStation", discs, false);
            Settle(window);
            var shown = Sheets(window).SheetOf(Sheets(window).Current!)!.GetVisualDescendants().OfType<Control>().ToList();
            Assert.Contains(shown, c => c.Name == (empty ? "InsertDiscAbout" : "InsertDisc1") && c.IsEffectivelyVisible);
            Assert.DoesNotContain(shown, c => c.Name == "InsertDiscBrowse");
        }

        // The PlayStation 2's tab, behind the development mark: a shared card with two saves, one with a long title, a game's own beside it, and the card held by a running game - see Sapphire_Plan.md §8 (G11), Sapphire_Native.md §30.16.
        internal static void ShowMemoryCardsPs2(MainWindow window, bool held)
        {
            var graphics = ConsolesConfig.Load();
            graphics.SetValue("PlayStation 2", EmuSen.Cores.MemoryCards.ModeKey, EmuSen.Cores.MemoryCards.Shared);
            graphics.Save();
            string card = SaveLibrary.CardPathFor("PlayStation 2", "Card 1", ".ps2");
            Directory.CreateDirectory(Path.GetDirectoryName(card)!);
            File.WriteAllBytes(card, Ps2CardFormatTests.WithSave(Ps2CardFormatTests.WithSave(Ps2CardFormat.Instance.Format()!, "BASLUS-20001AMBER", "Amber Saga", 40, 1),
                "BESLES-50002BASALT", "Basalt Run Chapter Two, the Long Road North Through the Mountains", 300, 2));
            File.WriteAllBytes(Path.Combine(DataStore.Saves, "PlayStation 2", "Amber Saga.ps2"), Ps2CardFormat.Instance.Format()!);
            if (held && MediumLock.TryTake(card, "Amber Saga", shared: true) is { } taken) window.Closed += (_, _) => taken.Dispose();
            EmuSen.Mistress.Library.RuntimeCatalog.Forget();
            Call(window, "ShowMemoryCards");
            SelectTab(window, "PlayStation 2");
            var shown = Sheets(window).SheetOf(Sheets(window).Current!)!.GetVisualDescendants().OfType<Control>().ToList();
            Assert.Contains(shown, c => c.Name == "PlayStation 2.ps2.card.Save.1" && c.IsEffectivelyVisible);
            var about = shown.OfType<TextBlock>().First(t => t.Name == "PlayStation 2.ps2.card.About");
            Assert.Equal(held, about.Text!.Contains("In use by Amber Saga"));
            Assert.StartsWith("Shared: Card 1 in Slot 1, Card 2 in Slot 2.", about.Text);
            Assert.Contains(shown.OfType<TextBlock>(), t => t.Text?.Contains("7651 of 7998 kilobytes free") == true);
        }

        // The N64's tab: four Controller Paks, a shared pak with two notes, one with a long name, and a game's own beside it - see EmuSen_SaveMedia.md §7.2.
        internal static void ShowMemoryCardsN64(MainWindow window)
        {
            string pak = SaveLibrary.CardPathFor("N64", "Pak 1", ".mpk");
            Directory.CreateDirectory(Path.GetDirectoryName(pak)!);
            byte[] notes = N64PakFormatTests.WithNote(N64PakFormatTests.WithNote(N64PakFormat.Instance.Format(), 0, "NKTE", "01", new byte[] { 0x26, 0x1A, 0x2B, 0x22, 0x28, 0x01, 0x24, 0x1A, 0x2B, 0x2D }, new[] { 5, 6 }, 1),
                1, "NZLE", "01", Enumerable.Repeat((byte)0x2C, 16).ToArray(), new[] { 7, 8, 9 }, 2);
            File.WriteAllBytes(pak, notes);
            File.WriteAllBytes(Path.Combine(DataStore.Saves, "N64", "Mario Kart 64.mpk"), N64PakFormat.Instance.Format());
            Call(window, "ShowMemoryCards");
            SelectTab(window, "N64");
            var shown = Sheets(window).SheetOf(Sheets(window).Current!)!.GetVisualDescendants().OfType<Control>().ToList();
            Assert.Contains(shown, c => c.Name == "N64.n64.pak.Save.1" && c.IsEffectivelyVisible);
            var about = shown.OfType<TextBlock>().First(t => t.Name == "N64.n64.pak.About");
            Assert.StartsWith("Shared: Pak 1 in Controller 1, Pak 2 in Controller 2, Pak 3 in Controller 3, Pak 4 in Controller 4.", about.Text);
        }
    }
}
