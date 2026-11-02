using System;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Serenity;

namespace EmuSen.WiseMan.Common
{
    // No test builds Avalonia objects off the headless session's dispatcher - see EmuSen_Settings_Reference.md §4.78.5.
    public class OffDispatcherTests
    {
        // The race tool does it on purpose; the shader bench needs a Vulkan device and was not run; C7 reaches the coretop window only through a command it never gives - §4.78.5.
        private static readonly string[] NotRunHere =
        [
            Of<HeadlessRaceTool>(nameof(HeadlessRaceTool.A_picture_off_the_dispatcher_borrows_whichever_application_is_running)),
            Of<HeadlessRaceTool>(nameof(HeadlessRaceTool.Start_up_fails_only_beside_controls_built_off_the_dispatcher)),
            Of<ShaderBenchTests>(nameof(ShaderBenchTests.A_short_bench_case_reports_the_chain_s_stages_and_leaves_no_probe)),
            Of<ShaderBenchTests>(nameof(ShaderBenchTests.A_short_device_path_case_reports_the_present_s_stages_and_draws_the_readback_s_pictures)),
            Of<HotaruOnTheApiTests>(nameof(HotaruOnTheApiTests.C7_hotaru_s_console_state_load_runs_the_api_s_sequence_and_resets_the_rate_control)),
        ];

        // By nameof, so renaming an exempt case fails the build rather than the scan.
        private static string Of<T>(string method) => $"{typeof(T).FullName}.{method}";

        private static bool Ours(Assembly assembly) => assembly.GetName().Name?.StartsWith("EmuSen", StringComparison.Ordinal) == true;

        private static string? Explain(string method) =>
            OffDispatcherScan.Explain(typeof(OffDispatcherTests).GetMethod(method, BindingFlags.NonPublic | BindingFlags.Static)!, typeof(UiTest).Assembly, typeof(AvaloniaObject), Ours);

        [Fact]
        public void No_test_builds_Avalonia_objects_off_the_session_s_dispatcher()
        {
            var found = OffDispatcherScan.Scan(typeof(UiTest).Assembly, typeof(AvaloniaObject), Ours).Where(f => !NotRunHere.Contains(f.Test)).ToList();

            Assert.True(found.Count == 0, "Built off the dispatcher; wrap the case in UiTest.Run:\n" + string.Join("\n", found.Select(f => $"{f.Test}: {f.Site}, via {f.Chain}")));
        }

        // The scan's controls: a scene built in a plain method is seen through the product's constructor, and the same scene dispatched is not.
        [Fact]
        public void The_scan_sees_a_scene_built_off_the_dispatcher_and_passes_one_built_on_it()
        {
            Assert.Contains("SceneView..ctor", Explain(nameof(SceneOffTheDispatcher)));
            Assert.Contains("RenderTargetBitmap", Explain(nameof(PictureOffTheDispatcher)));
            Assert.Null(Explain(nameof(SceneOnTheDispatcher)));
        }

        private static object SceneOffTheDispatcher(SceneData data) => new SceneView(data, "gamelist", TimeSpan.Zero);

        private static string PictureOffTheDispatcher() => SceneAssets.Halves("scan-control", 2, 2, Avalonia.Media.Colors.Red, Avalonia.Media.Colors.Blue);

        private static Task SceneOnTheDispatcher(SceneData data) => UiTest.Run(() => _ = new SceneView(data, "gamelist", TimeSpan.Zero));
    }
}
