using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Security.Cryptography;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The themed gamelist's game options and metadata editor, driven by the pad alone - see EmuSen_Settings_Reference.md §4.59 and EmuSen_BigPicture.md §23.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedGameOptionsTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedGameOptionsTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;

        public ThemedGameOptionsTests(ITestOutputHelper output) => _out = output;

        internal static SheetLayer Sheets(ThemedSession s) => s.Window.GetControl<SheetLayer>("Sheets");

        internal static Control Sheet(ThemedSession s) => Sheets(s).SheetOf(Sheets(s).Current!)!;

        internal static GamelistOptionsWindow OpenOptions(ThemedSession s)
        {
            s.Pad.Select();
            return Assert.IsType<GamelistOptionsWindow>(Sheets(s).Current);
        }

        // Select, then the pad walked to the entry and A pressed on it.
        internal static void Choose(ThemedSession s, string label)
        {
            OpenOptions(s);
            PadAudit.Reach(Sheet(s), s.Pad, e => e is Button { Content: string text } && text == label);
            s.Pad.A();
            s.Settle();
        }

        internal static void Reach(ThemedSession s, string name) => PadAudit.Reach(Sheet(s), s.Pad, e => e is Control { Name: { } n } && n == name);

        // The box reached and opened on the keyboard, what it held erased with B, the text typed and the keyboard put away with Start.
        internal static void Type(ThemedSession s, string box, string text)
        {
            Reach(s, box);
            s.Pad.A();
            OnScreenKeyboard keyboard = OnScreenKeyboard.OpenOver(s.Window)!;
            Assert.NotNull(keyboard);
            var target = (TextBox)Sheet(s).GetVisualDescendants().OfType<Control>().Single(c => c.Name == box);
            for (int n = target.Text?.Length ?? 0; n > 0; n--) s.Pad.B();
            PadCheatsTests.TypeByPad(s.Pad, keyboard, text);
            s.Pad.Start();
            Assert.Null(OnScreenKeyboard.OpenOver(s.Window));
        }

        // A dialog's button, reached and pressed by the pad.
        internal static void Answer(ThemedSession s, string button)
        {
            Assert.Equal("DialogWindow", Sheets(s).Current!.GetType().Name);
            PadAudit.Reach(Sheet(s), s.Pad, e => e is Button { Content: string text } && text == button);
            s.Pad.A();
            s.Settle();
        }

        internal static IReadOnlyDictionary<string, string> StoredEdits(ThemedSession s, string game)
        {
            using GameRecords records = GameRecords.Open(GameRecords.DefaultPath);
            return records.Edits(Path.Combine(s.RomDirectory, game));
        }

        internal static IReadOnlyList<string> ListedTitles(ThemedSession s) =>
            s.Themed.Stage!.Current.Data.System.Games.Select(g => g.Name).ToList();

        // ES-DE's "Exclude from game counter": left out of the system view's counts, favourites included.
        [Fact]
        public Task A_game_left_out_of_the_counter_is_not_counted_in_the_system_view() => UiTest.Run(() =>
        {
            using var theme = new SyntheticTheme();
            theme.Capabilities("").Theme("<view name=\"system\"><text name=\"x\"><systemdata>gamecount</systemdata></text></view>");
            (EmuSen.Mistress.BigPicture.Theme.ThemeSystem system, string extension) = SyntheticLibrary.Systems[1];
            List<SceneGame> games = SyntheticLibrary.Games(system, extension).Select((g, i) => i < 3 ? g with { NotCounted = true } : g).ToList();
            var choices = new EmuSen.Mistress.BigPicture.Theme.ThemeChoices { ScreenWidth = 320, ScreenHeight = 200 };
            var data = new SceneData([new SceneSystem(system, theme.Load(choices, system), games)], new Avalonia.Size(320, 200));
            SceneBuilder scene = SceneBuilder.Build(data.System.Theme.View("system"), data);
            Assert.Equal("9 games available, 4 favorites", ((FontText)scene.Entries.Single(e => e.Element.Name == "x").Control!).Text);
        });

        // Every file under a folder with its size, write time and SHA-256.
        internal static SortedDictionary<string, string> Fingerprint(string folder)
        {
            var all = new SortedDictionary<string, string>(StringComparer.Ordinal);
            if (!Directory.Exists(folder)) return all;
            foreach (string file in Directory.EnumerateFiles(folder, "*", SearchOption.AllDirectories))
            {
                var info = new FileInfo(file);
                all[Path.GetRelativePath(folder, file)] = $"{info.Length}|{info.LastWriteTimeUtc.Ticks}|{Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(file)))}";
            }
            return all;
        }

    }
}
