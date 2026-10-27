using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.LunaP.Commands;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // Hotaru, the shell's window in Mistress: opened from the menu, its toolbar, and the pad - see EmuSen_HotaruWindow.md §6.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class HotaruWindowTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(HotaruWindowTests).GetTypeInfo().Assembly);

        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenHotaruWindow", Guid.NewGuid().ToString("N"));

        public HotaruWindowTests()
        {
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static HotaruWindow Opened(MainWindow main)
        {
            MainWindowMenu.Click(main, "_Settings", "S_hell...");
            Dispatcher.UIThread.RunJobs();
            return main.OwnedWindows.OfType<HotaruWindow>().Single();
        }

        private static void Submit(HotaruWindow w, string line) =>
            typeof(HotaruWindow).GetMethod("Submit", Hidden)!.Invoke(w, new object[] { line });

        private static ConsolePane Pane(HotaruWindow w) => (ConsolePane)typeof(HotaruWindow).GetField("_console", Hidden)!.GetValue(w)!;

        private static void Toolbar(HotaruWindow w, string text) =>
            w.GetVisualDescendants().OfType<ToolBar>().Single(t => t.Name == HotaruWindow.ToolbarName).Actions.Single(a => a.Text == text).Invoke();

        // A window with the pane's template applied, so its input box exists.
        private static HotaruWindow Ready()
        {
            var main = new MainWindow();
            main.Show();
            HotaruWindow w = Opened(main);
            w.UpdateLayout();
            Dispatcher.UIThread.RunJobs();
            return w;
        }

        [Fact]
        public Task The_menu_opens_the_window_and_a_second_choice_finds_the_open_one() => Session.Dispatch(() =>
        {
            var main = new MainWindow();
            main.Show();
            HotaruWindow first = Opened(main);
            MainWindowMenu.Click(main, "_Settings", "S_hell...");
            Dispatcher.UIThread.RunJobs();
            Assert.Same(first, main.OwnedWindows.OfType<HotaruWindow>().Single());
            Assert.Equal("Shell", first.Title);
            main.Close();
        }, default);

        [Fact]
        public Task History_back_and_forward_step_through_the_lines_run() => Session.Dispatch(() =>
        {
            HotaruWindow w = Ready();
            Submit(w, "echo one");
            Submit(w, "echo two");
            Toolbar(w, "History Back");
            Assert.Equal("echo two", Pane(w).InputText);
            Toolbar(w, "History Back");
            Assert.Equal("echo one", Pane(w).InputText);
            Toolbar(w, "History Forward");
            Assert.Equal("echo two", Pane(w).InputText);
            w.Close();
        }, default);

        [Fact]
        public Task Interrupt_stops_a_running_loop_and_the_next_line_runs() => Session.Dispatch(() =>
        {
            HotaruWindow w = Ready();
            Submit(w, "while true; do echo x | grep x; done");
            // The line is queued for the emulation thread; this task is that thread, and the toolbar stops it.
            var run = Task.Run(w.DrainPendingFromEmulationThread);
            System.Threading.Thread.Sleep(100);
            Toolbar(w, "Interrupt");
            Assert.True(run.Wait(TimeSpan.FromSeconds(5)));
            Dispatcher.UIThread.RunJobs();
            Assert.EndsWith("^C", (string)typeof(HotaruWindow).GetField("_lastOutput", Hidden)!.GetValue(w)!);
            Submit(w, "echo after");
            Task.Run(w.DrainPendingFromEmulationThread).Wait();
            Dispatcher.UIThread.RunJobs();
            Assert.Equal("after", (string)typeof(HotaruWindow).GetField("_lastOutput", Hidden)!.GetValue(w)!);
            w.Close();
        }, default);

        [Fact]
        public Task Complete_finishes_the_first_word_to_the_one_command_it_starts() => Session.Dispatch(() =>
        {
            HotaruWindow w = Ready();
            var names = DianaShInterpreter.CreateWithoutSession().CommandNames;
            string name = names.Last(n => n.Length >= 4 && names.Count(m => m.StartsWith(n[..3], StringComparison.Ordinal)) == 1);
            Pane(w).SetInput(name[..3]);
            Toolbar(w, "Complete");
            Assert.Equal(name + " ", Pane(w).InputText);

            Pane(w).SetInput("");
            Toolbar(w, "Complete");
            Assert.Equal("", Pane(w).InputText);
            w.Close();
        }, default);

        [Fact]
        public Task Clear_empties_the_output() => Session.Dispatch(() =>
        {
            HotaruWindow w = Ready();
            Submit(w, "echo hello");
            Assert.Contains("hello", Pane(w).OutputText);
            Toolbar(w, "Clear");
            Assert.Equal("", Pane(w).OutputText);
            w.Close();
        }, default);

        [Fact]
        public Task Copy_output_puts_the_last_output_on_the_clipboard() => Session.Dispatch(async () =>
        {
            HotaruWindow w = Ready();
            Submit(w, "echo first");
            Submit(w, "echo second");
            Toolbar(w, "Copy Output");
            Assert.Equal("second", await Avalonia.Input.Platform.ClipboardExtensions.TryGetTextAsync(w.Clipboard!));
            w.Close();
        }, default);

        [Fact]
        public Task The_shoulders_recall_history_a_trigger_completes_and_B_clears_then_closes() => Session.Dispatch(() =>
        {
            HotaruWindow w = Ready();
            Submit(w, "echo one");
            PadWindowRouter.Send(w, UiButton.PageUp);
            Assert.Equal("echo one", Pane(w).InputText);
            PadWindowRouter.Send(w, UiButton.PageDown);
            Assert.Equal("", Pane(w).InputText);

            var names = DianaShInterpreter.CreateWithoutSession().CommandNames;
            string name = names.Last(n => n.Length >= 4 && names.Count(m => m.StartsWith(n[..3], StringComparison.Ordinal)) == 1);
            Pane(w).SetInput(name[..3]);
            PadWindowRouter.Send(w, UiButton.Last);
            Assert.Equal(name + " ", Pane(w).InputText);

            PadWindowRouter.Send(w, UiButton.Back);
            Assert.Equal("", Pane(w).InputText);
            Assert.True(w.IsVisible);
            PadWindowRouter.Send(w, UiButton.Back);
            Assert.False(w.IsVisible);
        }, default);
    }
}
