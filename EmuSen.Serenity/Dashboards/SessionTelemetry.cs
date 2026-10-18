using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Session;
using EmuSen.Crystal.Telemetry;

namespace EmuSen.Serenity.Dashboards
{
    // What the coretop window reads from a session, as the frontend API answers it; null where the call failed, a closed session among them.
    public interface ISessionReadout
    {
        string? Telemetry(uint kind);
        string? Info();
        string? Summary();
        byte[]? TileSheet();
        byte[]? PaletteSwatch();
        byte[]? AudioPeek();
    }

    // The coretop window's data over a session's readout, parsed as the session service's JSON and pictures give it - see EmuSen_Backend.md §14.11.
    public sealed class SessionTelemetry : ICoreTelemetry
    {
        private readonly ISessionReadout _session;

        public SessionTelemetry(ISessionReadout session)
        {
            _session = session;
            CpuRegisters = Registers(SessionAbi.TelemetryCpu);
            VideoRegisters = Registers(SessionAbi.TelemetryVideo);
            ApuRegisters = Registers(SessionAbi.TelemetryApu);
            CoprocessorRegisters = Registers(SessionAbi.TelemetryCoprocessor);
            Sprites = Read(SessionAbi.TelemetrySprites, e => new DebugSpriteInfo(e.GetProperty("index").GetInt32(), e.GetProperty("x").GetInt32(), e.GetProperty("y").GetInt32(),
                e.GetProperty("width").GetInt32(), e.GetProperty("height").GetInt32(), e.GetProperty("tile").GetInt32(), e.GetProperty("palette").GetInt32(),
                e.GetProperty("priority").GetInt32(), e.GetProperty("flip_x").GetBoolean(), e.GetProperty("flip_y").GetBoolean()));
            Palettes = Read(SessionAbi.TelemetryPalettes, e => new DebugPaletteInfo(e.GetProperty("index").GetInt32(),
                e.GetProperty("colors").EnumerateArray().Select(c => (c[0].GetByte(), c[1].GetByte(), c[2].GetByte())).ToArray()));
            AudioChannels = Read(SessionAbi.TelemetryAudioChannels, e => new DebugAudioChannelInfo(e.GetProperty("index").GetInt32(), e.GetProperty("name").GetString() ?? "",
                e.GetProperty("active").GetBoolean(), e.GetProperty("level").GetInt32(), e.GetProperty("muted").GetBoolean(), e.GetProperty("info").GetString() ?? ""));
            HardwareLoad = Read(SessionAbi.TelemetryHardwareLoad, e => new DebugLoadInfo(e.GetProperty("name").GetString() ?? "", e.GetProperty("percent").GetDouble(),
                e.GetProperty("kind").GetString() == "guest_utilization" ? DebugLoadKind.GuestUtilization : DebugLoadKind.EmulatorCost));
        }

        public string CoreName => Info("engine")?.GetString() ?? "";
        public long FrameCount => Info("frame")?.GetInt64() ?? 0;
        public int MaxSprites => Info("max_sprites")?.GetInt32() ?? 0;

        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CpuRegisters { get; }
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> VideoRegisters { get; }
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> ApuRegisters { get; }
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CoprocessorRegisters { get; }
        public IRealtimeProvider<IReadOnlyList<DebugSpriteInfo>> Sprites { get; }
        public IRealtimeProvider<IReadOnlyList<DebugPaletteInfo>> Palettes { get; }
        public IRealtimeProvider<IReadOnlyList<DebugAudioChannelInfo>> AudioChannels { get; }
        public IRealtimeProvider<IReadOnlyList<DebugLoadInfo>> HardwareLoad { get; }

        // The session publishes each frame's snapshot itself.
        public void RefreshProviders() { }

        public (byte[] Rgba, int Width, int Height) RenderTileSheet() => Picture(_session.TileSheet());

        public (byte[] Rgba, int Width, int Height) RenderPaletteSwatch() => Picture(_session.PaletteSwatch());

        public (short[] Samples, int SampleRate) GetAudioSamples()
        {
            if (_session.AudioPeek() is not { Length: >= 4 } bytes) return (Array.Empty<short>(), 0);
            var samples = new short[(bytes.Length - 4) / 2];
            for (int i = 0; i < samples.Length; i++) samples[i] = BinaryPrimitives.ReadInt16LittleEndian(bytes.AsSpan(4 + i * 2));
            return (samples, (int)BinaryPrimitives.ReadUInt32LittleEndian(bytes));
        }

        public string GetSummaryText() => _session.Summary() ?? "";

        // Width and height as two little-endian uint32, then the rows.
        private static (byte[] Rgba, int Width, int Height) Picture(byte[]? bytes)
        {
            if (bytes is not { Length: >= 8 }) return (Array.Empty<byte>(), 0, 0);
            return (bytes[8..], (int)BinaryPrimitives.ReadUInt32LittleEndian(bytes), (int)BinaryPrimitives.ReadUInt32LittleEndian(bytes.AsSpan(4)));
        }

        private JsonElement? Info(string key)
        {
            if (_session.Info() is not { } text) return null;
            using JsonDocument info = JsonDocument.Parse(text);
            return info.RootElement.TryGetProperty(key, out JsonElement value) ? value.Clone() : null;
        }

        private Provider<DebugRegisterValue> Registers(uint kind) =>
            Read(kind, e => new DebugRegisterValue(e.GetProperty("name").GetString() ?? "", e.GetProperty("value").GetUInt64(), e.GetProperty("bits").GetInt32()));

        private Provider<T> Read<T>(uint kind, Func<JsonElement, T> item) => new(() =>
        {
            if (_session.Telemetry(kind) is not { } text) return Array.Empty<T>();
            using JsonDocument json = JsonDocument.Parse(text);
            return json.RootElement.EnumerateArray().Select(item).ToArray();
        });

        // Read when the window asks; Refresh has nothing to do.
        private sealed class Provider<T>(Func<IReadOnlyList<T>> read) : IRealtimeProvider<IReadOnlyList<T>>
        {
            public IReadOnlyList<T> Current => read();

            public void Refresh() { }
        }
    }
}
