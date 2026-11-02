using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.Mistress.Views.Covers;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;

namespace EmuSen.WiseMan.Mistress
{
    // The desktop's sidebar library as tests drive it: a session, the list, a click and the context menu - see EmuSen_Settings_Reference.md §4.63.
    [Collection(TestCollections.ProcessGlobals)]
    public class DesktopLibrary
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(DesktopLibrary).GetTypeInfo().Assembly);

        private static readonly string[] GameEntries = ["Add to Favourites", "Edit This Game's Metadata", "Crop This Game...", "Use Another Game's Cover...", "Game Package...", "Scrape This Game...", "Find by Name..."];

        // The Home card's game page: the same entries but the favourite, which is Y's, the television standard as a row of its own where the options menu draws a choice (EmuSen_BigPicture.md §49.3).
        private static readonly string[] GamePage = ["Edit This Game's Metadata", "Crop This Game...", MainWindow.StandardRow, "Use Another Game's Cover...", "Game Package...", "Scrape This Game...", "Find by Name..."];

        internal static ThemedSession Desktop(string view = AppSettings.LibraryList) =>
            new(settings: a => { a.BigScreen = false; a.LibraryView = view; });

        private static ThemedSession BuiltInBigScreen() => new(settings: a => a.LibraryStyle = AppSettings.LibraryStyleMistress);

        internal static LunaList<RomEntry> List(MainWindow w) => (LunaList<RomEntry>)w.GetControl<ListBox>("LibraryList");

        private static TileGrid<RomEntry> Grid(MainWindow w) => w.GetControl<TileGrid<RomEntry>>("LibraryGrid");

        private static SheetLayer Sheets(MainWindow w) => w.GetControl<SheetLayer>("Sheets");

        internal static string SnesPath(ThemedSession s, int n) => Path.Combine(s.RomDirectory, ThemedSession.SnesGames[n] + ".sfc");

        private static RomEntry Entry(ThemedSession s, int n) => List(s.Window).Models.Single(e => e.FullPath == SnesPath(s, n));

        // The list's names without the console the mixed list adds after a dash.
        internal static IEnumerable<string> ListLabels(MainWindow w) => List(w).Models.Select(List(w).Label).Select(l => l.Split("   —   ")[0]);

        internal static T Named<T>(Visual root, string name) where T : Control => root.GetVisualDescendants().OfType<T>().Single(c => c.Name == name);

        internal static void Click(ThemedSession s, Button button)
        {
            Assert.True(button.IsEffectivelyEnabled && button.IsEffectivelyVisible, button.Name);
            button.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            s.Settle();
        }

        // The context menu's entry clicked, as a right-click then a left-click on it would; returns every label the menu showed.
        internal static IReadOnlyList<string> ChooseFromContextMenu(ThemedSession s, Control owner, string label)
        {
            ContextMenu menu = owner.ContextMenu!;
            Assert.True(menu.IsOpen);
            ActionMenuItem[] items = menu.ItemsSource!.OfType<ActionMenuItem>().ToArray();
            string[] labels = items.Select(i => i.Action.Text).ToArray();
            ActionMenuItem item = items.Single(i => i.Action.Text == label);
            Assert.True(item.IsEnabled, label);
            menu.Close();
            item.RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));
            s.Settle();
            return labels;
        }

        // A right-click on the game's tile, which selects it and opens the grid's context menu.
        private static void RightClickTile(ThemedSession s, int n)
        {
            string title = ArtworkIndex.Untagged(ThemedSession.SnesGames[n]);
            RightClick(s, s.Window.GetVisualDescendants().OfType<CoverTile>().Single(t => t.IsEffectivelyVisible && t.Title == title));
        }

        // A right-click on the game's row in the list.
        private static void RightClickRow(ThemedSession s, int n)
        {
            LunaList<RomEntry> list = List(s.Window);
            RightClick(s, list.ContainerFromIndex(list.Models.ToList().IndexOf(Entry(s, n)))!);
        }

        private static void RightClick(ThemedSession s, Control target)
        {
            Point centre = target.TranslatePoint(new Point(target.Bounds.Width / 2, target.Bounds.Height / 2), s.Window)!.Value;
            s.Window.MouseDown(centre, MouseButton.Right);
            s.Window.MouseUp(centre, MouseButton.Right);
            s.Settle();
        }


    }
}
