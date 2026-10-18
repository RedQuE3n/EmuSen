using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia.Input;

namespace EmuSen.Hotaru.Input
{
    // What a session's machine takes as input, read from the API's session_info: how many ports and which axes.
    public sealed record HotaruControls(int Ports, IReadOnlyList<PadAxis> Axes)
    {
        public static HotaruControls Of(CrystalSession session)
        {
            using JsonDocument info = JsonDocument.Parse(session.Info());
            JsonElement root = info.RootElement;
            return new HotaruControls(root.GetProperty("ports").GetArrayLength(), root.GetProperty("axes").EnumerateArray().Select(a => (PadAxis)a.GetInt32()).ToArray());
        }

        // The machine's frame rate, for the pacer's first frame before a frame call has said it.
        public static double FrameRateOf(CrystalSession session)
        {
            using JsonDocument info = JsonDocument.Parse(session.Info());
            double hz = info.RootElement.GetProperty("frame_rate_hz").GetDouble();
            return hz > 0 ? hz : 60.0;
        }
    }

    // The consoles of the systems table, oldest first, which the pad bindings are kept by.
    public static class HotaruSystems
    {
        public static IReadOnlyList<string> ConsolesInReleaseOrder(CrystalRuntime runtime)
        {
            using JsonDocument systems = JsonDocument.Parse(runtime.Systems());
            return systems.RootElement.EnumerateArray()
                .Where(s => s.TryGetProperty("console", out _))
                .Select(s => (Console: s.GetProperty("console").GetString()!, Year: s.TryGetProperty("release_year", out JsonElement y) ? y.GetInt32() : int.MaxValue))
                .OrderBy(s => s.Year)
                .Select(s => s.Console)
                .Distinct()
                .ToArray();
        }
    }
}
