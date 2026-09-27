using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Media.Imaging;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Common
{
    public sealed class HeadlessRaceFactAttribute : FactAttribute
    {
        public HeadlessRaceFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_HEADLESS_RACE") != "1")
                Skip = "Provokes the headless session's start-up race on purpose, so it runs alone; set EMUSEN_HEADLESS_RACE=1 - see EmuSen_Settings_Reference.md §4.77.2";
        }
    }

    // What Avalonia work off the session's dispatcher does to itself and to the tests beside it - see EmuSen_Settings_Reference.md §4.77.2.
    [Collection(TestCollections.ProcessGlobals)]
    public class HeadlessRaceTool(ITestOutputHelper output)
    {
        private static int Rounds => int.TryParse(Environment.GetEnvironmentVariable("EMUSEN_HEADLESS_RACE_ROUNDS"), out int n) ? n : 400;

        // A picture made off the dispatcher finds a renderer only while some dispatch is inside its application.
        [HeadlessRaceFact]
        public async Task A_picture_off_the_dispatcher_borrows_whichever_application_is_running()
        {
            string alone = Attempt();

            using var inside = new ManualResetEventSlim();
            using var release = new ManualResetEventSlim();
            Task running = UiTest.Run(() =>
            {
                inside.Set();
                release.Wait(TimeSpan.FromSeconds(10));
            });
            inside.Wait(TimeSpan.FromSeconds(10));
            string beside = Attempt();
            release.Set();
            await running;

            output.WriteLine($"no dispatch running: {alone}");
            output.WriteLine($"a dispatch running:  {beside}");
            Assert.StartsWith("InvalidOperationException", alone);
            Assert.Equal("made", beside);
        }

        // Dispatches that each show a window: alone, beside a thread building a Border at a time, and beside one building the scene GridSceneTests' plain facts built.
        [HeadlessRaceFact]
        public async Task Start_up_fails_only_beside_controls_built_off_the_dispatcher()
        {
            SceneData data = null!;
            await UiTest.Run(() => data = GridData());
            var quiet = await Dispatch(null);
            var border = await Dispatch(() => _ = new Border());
            var scene = await Dispatch(() => _ = new SceneView(data, "gamelist", TimeSpan.Zero));

            output.WriteLine($"{Rounds} dispatches alone: {Describe(quiet)}");
            output.WriteLine($"{Rounds} dispatches beside a thread building Borders: {Describe(border)}");
            output.WriteLine($"{Rounds} dispatches beside a thread building GridSceneTests' scene: {Describe(scene)}");
            Assert.Empty(quiet.Failures);
        }

        private static string Attempt()
        {
            try
            {
                using var bitmap = new RenderTargetBitmap(new PixelSize(2, 2));
                return "made";
            }
            catch (Exception e)
            {
                return $"{e.GetType().Name}: {e.Message}";
            }
        }

        // GridSceneTests' Data(Grid()) as it stood before b272675c, built once on the dispatcher.
        private static SceneData GridData()
        {
            const string grid =
                "<grid name=\"g\"><pos>0 0.1</pos><size>1 0.8</size><itemSize>0.2 0.3</itemSize><itemSpacing>0.02 0.03</itemSpacing><itemScale>1.2</itemScale>" +
                "<imageType>cover</imageType><imageFit>fill</imageFit><unfocusedItemOpacity>0.4</unfocusedItemOpacity></grid>" +
                "<text name=\"n\"><pos>0 0</pos><size>1 0.08</size><metadata>name</metadata><fontSize>0.05</fontSize><color>FFFFFF</color></text>";
            using var theme = new SyntheticTheme();
            theme.Capabilities("").Theme($"<view name=\"gamelist\">{grid}</view>");
            var systems = SyntheticLibrary.Systems.Select(s => new SceneSystem(s.System, theme.Load(new ThemeChoices { ScreenWidth = 640, ScreenHeight = 400 }, s.System), SyntheticLibrary.Games(s.System, s.Extension))).ToList();
            return new SceneData(systems, new Size(640, 400)) { SystemIndex = 1, GameIndex = 0, Media = new SceneAssets.Media(), Motion = SceneMotion.Esde };
        }

        private sealed record Outcome(Dictionary<string, int> Failures, int Built, TimeSpan Took);

        private static async Task<Outcome> Dispatch(Action? offThread)
        {
            var failures = new Dictionary<string, int>();
            int built = 0;
            using var stop = new CancellationTokenSource();
            var builder = new Thread(() =>
            {
                while (!stop.IsCancellationRequested)
                {
                    try { offThread!(); built++; } catch (InvalidOperationException) { }
                }
            }) { IsBackground = true };
            if (offThread is not null) builder.Start();

            var clock = Stopwatch.StartNew();
            for (int i = 0; i < Rounds; i++)
            {
                try
                {
                    await UiTest.Run(() =>
                    {
                        var window = new Window { Width = 32, Height = 32 };
                        window.Show();
                        window.Close();
                    });
                }
                catch (Exception e)
                {
                    string where = e.StackTrace?.Contains("EnsureIsolatedApplication", StringComparison.Ordinal) == true ? "in the session's start-up" : "in the dispatched body";
                    string kind = $"{e.GetType().Name}: {e.Message.Split('\n')[0]} ({where}, from {e.StackTrace?.Split('\n').Skip(1).FirstOrDefault()?.Trim()})";
                    failures[kind] = failures.GetValueOrDefault(kind) + 1;
                }
            }

            clock.Stop();
            stop.Cancel();
            if (offThread is not null) builder.Join();
            return new Outcome(failures, built, clock.Elapsed);
        }

        private static string Describe(Outcome o) =>
            $"{o.Took.TotalSeconds:F1} s, {o.Built} built off the dispatcher; " +
            (o.Failures.Count == 0 ? "none failed" : string.Join("; ", o.Failures.Select(f => $"{f.Value} x {f.Key}")));
    }
}
