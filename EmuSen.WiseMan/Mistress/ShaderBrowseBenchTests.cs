using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Threading;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress
{
    // Moving onto a pack preset and walking the list with a held pad, timed to a drawn, usable row; gated on EMUSEN_SLANG_PACK - see EmuSen_Settings_Reference.md §4.48.9.
    [Collection(TestCollections.ProcessGlobals)]
    public class ShaderBrowseBenchTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ShaderBrowseBenchTests).GetTypeInfo().Assembly);

        // Where each measurement is appended as one line, for runs of two builds interleaved.
        private const string OutVariable = "EMUSEN_SHADER_BENCH_OUT";

        private static readonly string[] Targets =
        {
            "crt/crt-lottes.slangp", "crt/crt-royale.slangp", "crt/crt-guest-advanced.slangp", "bezel/Mega_Bezel/Presets/MBZ__0__SMOOTH-ADV.slangp",
        };

        private readonly ITestOutputHelper _out;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenShaderBrowseBench", Guid.NewGuid().ToString("N"));

        public ShaderBrowseBenchTests(ITestOutputHelper output)
        {
            _out = output;
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private void Record(string line)
        {
            _out.WriteLine(line);
            if (Environment.GetEnvironmentVariable(OutVariable) is { Length: > 0 } file) File.AppendAllText(file, line + Environment.NewLine);
        }

        private static (ShaderSettingsWindow Window, ShaderPanel Panel) Open(string pack)
        {
            var window = new ShaderSettingsWindow(new GraphicsConfig(), null, "SNES", pack: pack);
            window.Show();
            ShaderPanel panel = window.PanelFor("SNES");
            Wait(panel);
            UiTest.Capture(window);
            return (window, panel);
        }

        private static void Wait(ShaderPanel panel)
        {
            var limit = Stopwatch.StartNew();
            while (!panel.Reading.IsCompleted && limit.ElapsedMilliseconds < 20000) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(1); }
            Dispatcher.UIThread.RunJobs();
            Assert.True(panel.Reading.IsCompleted);
        }

        // From the act to the shown preset's sliders built, laid out and drawn.
        private static long Usable(ShaderSettingsWindow window, ShaderPanel panel, Stopwatch clock, string relative)
        {
            var limit = Stopwatch.StartNew();
            while ((panel.Shown?.Relative != relative || !panel.Reading.IsCompleted) && limit.ElapsedMilliseconds < 20000) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(1); }
            Dispatcher.UIThread.RunJobs();
            UiTest.Capture(window);
            long elapsed = clock.ElapsedMilliseconds;
            Assert.Equal(relative, panel.Shown?.Relative);
            Assert.NotEmpty(panel.Sliders);
            return elapsed;
        }

        private static void FocusRow(ShaderSettingsWindow window, ShaderPanel panel, int index)
        {
            panel.List.SelectedIndex = index;
            Wait(panel);
            panel.List.ScrollIntoView(index);
            window.UpdateLayout();
            panel.List.ContainerFromIndex(index)!.Focus(NavigationMethod.Directional);
        }

        // Long enough for any settle and background read a stop starts, measured or not, to have finished.
        private static void Settled(ShaderPanel panel)
        {
            Wait(panel);
            var clock = Stopwatch.StartNew();
            while (clock.ElapsedMilliseconds < 1500) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(1); }
            Wait(panel);
        }

        private static int IndexOf(ShaderPanel panel, string relative) => panel.List.Models.ToList().FindIndex(e => e.Relative == relative && !e.Recent);

        [Fact]
        public Task Moving_onto_a_preset_and_a_held_walk_are_timed_to_a_usable_row() => Session.Dispatch(() =>
        {
            string? pack = Environment.GetEnvironmentVariable(EmuSen.WiseMan.Serenity.SlangPresetTests.PackVariable);
            if (string.IsNullOrEmpty(pack)) return;

            // Unmeasured: the parser, the list and the pad path compiled once, on presets that are not measured.
            {
                (ShaderSettingsWindow window, ShaderPanel panel) = Open(pack);
                int other = panel.List.Models.ToList().FindIndex(e => e.IsPreset && !e.Recent && !Targets.Contains(e.Relative) && e.Relative!.StartsWith("crt/", StringComparison.Ordinal));
                FocusRow(window, panel, other);
                Settled(panel);
                PadWindowRouter.Send(window, UiButton.Down);
                Settled(panel);
                UiTest.Capture(window);
                window.Close();
            }

            foreach (string relative in Targets)
            {
                (ShaderSettingsWindow window, ShaderPanel panel) = Open(pack);
                int index = IndexOf(panel, relative);
                var clock = Stopwatch.StartNew();
                panel.List.SelectedIndex = index;
                long ms = Usable(window, panel, clock, relative);
                Record($"select\t{relative}\t{ms}");
                window.Close();

                (window, panel) = Open(pack);
                panel.List.ScrollIntoView(index);
                window.UpdateLayout();
                Control row = panel.List.ContainerFromIndex(index)!;
                Point at = row.TranslatePoint(new Point(row.Bounds.Width / 2, row.Bounds.Height / 2), window)!.Value;
                clock = Stopwatch.StartNew();
                window.MouseDown(at, MouseButton.Left);
                window.MouseUp(at, MouseButton.Left);
                ms = Usable(window, panel, clock, relative);
                Record($"click\t{relative}\t{ms}");
                window.Close();

                // Two rows by the d-pad without stopping, so the row arrived at was never beside a settled one.
                (window, panel) = Open(pack);
                FocusRow(window, panel, index - 2);
                Settled(panel);
                PadWindowRouter.Send(window, UiButton.Down);
                Dispatcher.UIThread.RunJobs();
                clock = Stopwatch.StartNew();
                PadWindowRouter.Send(window, UiButton.Down);
                ms = Usable(window, panel, clock, relative);
                Record($"pad-cold\t{relative}\t{ms}");
                window.Close();

                // One row by the d-pad from a row the player stopped on.
                (window, panel) = Open(pack);
                FocusRow(window, panel, index - 1);
                Settled(panel);
                clock = Stopwatch.StartNew();
                PadWindowRouter.Send(window, UiButton.Down);
                ms = Usable(window, panel, clock, relative);
                Record($"pad-stopped\t{relative}\t{ms}");
                window.Close();
            }

            // A held d-pad repeats every 80 ms (PadNavigator.RepeatEvery), on the UI thread, so a busy thread delays the next step.
            {
                (ShaderSettingsWindow window, ShaderPanel panel) = Open(pack);
                int start = panel.List.Models.ToList().FindIndex(e => e.Relative?.StartsWith("bezel/Mega_Bezel/Presets/", StringComparison.Ordinal) == true && !e.Recent);
                panel.List.SelectedIndex = start;
                Wait(panel);
                panel.List.ScrollIntoView(start);
                window.UpdateLayout();
                panel.List.ContainerFromIndex(start)!.Focus(NavigationMethod.Directional);
                const int rows = 40;
                var walk = Stopwatch.StartNew();
                var press = Stopwatch.StartNew();
                for (int step = 0; step < rows; step++)
                {
                    press.Restart();
                    PadWindowRouter.Send(window, UiButton.Down);
                    if (step == rows - 1) break;
                    while (press.ElapsedMilliseconds < 80) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(1); }
                }
                string last = panel.List.Selected!.Relative!;
                long stop = Usable(window, panel, press, last);
                long total = walk.ElapsedMilliseconds;
                Assert.Equal(start + rows, panel.List.SelectedIndex);
                Record($"walk\t{rows} rows from {panel.List.Models[start].Relative}\t{(total - stop) / (double)(rows - 1):F1} ms a row\tstop {stop}");
                window.Close();
            }
        }, default);
    }
}
