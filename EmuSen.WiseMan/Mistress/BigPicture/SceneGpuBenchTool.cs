using System;
using System.Globalization;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // EMUSEN_BIGPICTURE_GPU=1 runs the GPU bench on the device EMUSEN_BIGPICTURE_GL_DEVICE names (the RX 6800 by default) - see EmuSen_BigPicture.md §14.
    public class SceneGpuBenchTool
    {
        private readonly ITestOutputHelper _output;

        public SceneGpuBenchTool(ITestOutputHelper output) => _output = output;

        [ArtBookNextFact]
        public System.Threading.Tasks.Task Gpu_frame_cost() => UiTest.Run(() =>
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_GPU") != "1") return;
            string device = Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_GL_DEVICE") is { Length: > 0 } d ? d : "RX 6800";
            int frames = int.TryParse(Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_FRAMES"), CultureInfo.InvariantCulture, out int f) ? f : 60;
            string png = System.IO.Path.Combine(SceneRenderTool.Cache, "gpu");
            foreach (string line in SceneGpuBench.Run(ArtBookNextFactAttribute.Folder, SceneRenderTool.MediaRoot, [(1280, 800), (1920, 1200)], ["system", "gamelist"], frames, device, _output.WriteLine, png,
                [.. SceneGpuBench.NoLevers, SceneGpuBench.ImmutableBitmaps]))
                _output.WriteLine(line);
        });
    }
}
