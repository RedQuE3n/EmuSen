using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia.Input;

namespace EmuSen.Mistress
{
    // A game, or a console started empty, open on the frontend API, with what Mistress shows of it read from the session's info - see EmuSen_Backend.md §14.13.
    public sealed class GameSession : IDisposable
    {
        private GameSession(CrystalSession crystal, bool lent)
        {
            Crystal = crystal;
            DeviceOffered = lent;
            using JsonDocument info = JsonDocument.Parse(crystal.Info());
            JsonElement i = info.RootElement;
            Console = i.GetProperty("console").GetString() ?? "";
            Engine = i.GetProperty("engine").GetString() ?? "";
            System = i.TryGetProperty("system", out JsonElement s) && s.ValueKind == JsonValueKind.String ? s.GetString() : null;
            GamePath = i.GetProperty("game") is { ValueKind: JsonValueKind.Object } game ? game.GetProperty("path").GetString() : null;
            Axes = i.GetProperty("axes").EnumerateArray().Select(a => (PadAxis)a.GetInt32()).ToArray();
            FrameRateHz = i.GetProperty("frame_rate_hz").GetDouble();
            RewindCapture = i.GetProperty("rewind_capture").GetBoolean();
            RewindWithheld = Text(i, "rewind_withheld");
            JsonElement notices = i.GetProperty("notices");
            (EngineNotice, FirmwareNotice, MediaNotice) = (Text(notices, "engine"), Text(notices, "firmware"), Text(notices, "media"));
            TelevisionStandard = Text(i, "television_standard");
            ReadPorts(i);
        }

        private static string? Text(JsonElement e, string name) =>
            e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.String ? v.GetString() : null;

        // Opened on any thread; the frame thread is whichever first calls frame.
        public static GameSession Open(CrystalRuntime runtime, CrystalOpen open) => new(runtime.Open(open), open.Device != 0);

        public CrystalSession Crystal { get; }
        public uint Id => Crystal.Id;
        public string Console { get; }
        public string Engine { get; }
        public string? System { get; }
        public string? GamePath { get; }
        public IReadOnlyList<PadAxis> Axes { get; }
        public double FrameRateHz { get; }
        public bool RewindCapture { get; }
        public string? RewindWithheld { get; }
        public string? EngineNotice { get; }
        public string? FirmwareNotice { get; }
        public string? MediaNotice { get; }
        public string? TelevisionStandard { get; }

        // Whether the session was opened with the presenter's device to lend each machine.
        public bool DeviceOffered { get; }

        // Each port's controller as the running game holds it; the count follows a setting that moves it, as an adapter's.
        public int ControllerPorts => Volatile.Read(ref _controllers).Count;
        public IReadOnlyList<string?> PortControllers => Volatile.Read(ref _controllers);
        private IReadOnlyList<string?> _controllers = Array.Empty<string?>();

        private void ReadPorts(JsonElement info)
        {
            string?[] ports = info.GetProperty("ports").EnumerateArray().Select(p => p.TryGetProperty("controller", out JsonElement c) && c.ValueKind == JsonValueKind.String ? c.GetString() : null).ToArray();
            Volatile.Write(ref _controllers, ports.Length == 0 ? new string?[1] : ports);
        }

        // Read again after a setting changed, which can move a console's ports.
        public void RefreshPorts()
        {
            using JsonDocument info = JsonDocument.Parse(Crystal.Info());
            ReadPorts(info.RootElement);
        }

        // The drive as it stands: the game's discs, the one in it, whether it changes discs and whether its lid is open.
        public (int Count, int Index, bool CanChange, bool LidOpen) Discs
        {
            get
            {
                try
                {
                    using JsonDocument info = JsonDocument.Parse(Crystal.Info());
                    JsonElement d = info.RootElement.GetProperty("discs");
                    return (d.GetProperty("count").GetInt32(), d.GetProperty("index").GetInt32(), d.GetProperty("can_change").GetBoolean(), d.GetProperty("lid_open").GetBoolean());
                }
                catch (CrystalException) { return (0, 0, false, false); }
            }
        }

        public void Dispose() => Crystal.Dispose();
    }
}
