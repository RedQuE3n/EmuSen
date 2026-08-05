using System;
using System.Collections.Generic;
using EmuSen.Cauldron;

namespace EmuSen.WiseMan.Fixtures
{
    // A fixed ICoreTelemetry, so a dashboard can be tested without a core - see EmuSen_LunaP.md §11.
    public sealed class FakeTelemetry : ICoreTelemetry
    {
        private sealed class Fixed<T> : IRealtimeProvider<T>
        {
            public Fixed(T value) => Current = value;

            public T Current { get; }

            public void Refresh() { }
        }

        public string CoreName { get; set; } = "SNES";

        public long FrameCount { get; set; } = 1071;

        public int MaxSprites { get; set; } = 128;

        public IReadOnlyList<DebugRegisterValue> Registers { get; set; } = new[]
        {
            new DebugRegisterValue("PC", 0x008123, 24),
            new DebugRegisterValue("A", 0x0000, 16),
            new DebugRegisterValue("X", 0x01FF, 16),
        };

        public IReadOnlyList<DebugLoadInfo> Load { get; set; } = new[]
        {
            new DebugLoadInfo("S-CPU", 24, DebugLoadKind.EmulatorCost),
            new DebugLoadInfo("S-PPU", 68, DebugLoadKind.EmulatorCost),
            new DebugLoadInfo("VRAM", 91, DebugLoadKind.GuestUtilization),
        };

        public IReadOnlyList<DebugAudioChannelInfo> Audio { get; set; } = new[]
        {
            new DebugAudioChannelInfo(0, "Voice 0", true, 40, false, ""),
            new DebugAudioChannelInfo(1, "Voice 1", false, 0, true, ""),
        };

        public IReadOnlyList<DebugSpriteInfo> SpriteList { get; set; } = Array.Empty<DebugSpriteInfo>();

        // 0x0 is a legitimate answer - a core with no tile memory reports exactly this.
        public (byte[] Rgba, int Width, int Height) TileSheet { get; set; } = (Array.Empty<byte>(), 0, 0);

        public (byte[] Rgba, int Width, int Height) PaletteSwatch { get; set; } = (Solid(16, 16), 16, 16);

        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CpuRegisters => new Fixed<IReadOnlyList<DebugRegisterValue>>(Registers);

        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> VideoRegisters => new Fixed<IReadOnlyList<DebugRegisterValue>>(Array.Empty<DebugRegisterValue>());

        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> ApuRegisters => new Fixed<IReadOnlyList<DebugRegisterValue>>(Array.Empty<DebugRegisterValue>());

        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CoprocessorRegisters => new Fixed<IReadOnlyList<DebugRegisterValue>>(Array.Empty<DebugRegisterValue>());

        public IRealtimeProvider<IReadOnlyList<DebugSpriteInfo>> Sprites => new Fixed<IReadOnlyList<DebugSpriteInfo>>(SpriteList);

        public IRealtimeProvider<IReadOnlyList<DebugPaletteInfo>> Palettes => new Fixed<IReadOnlyList<DebugPaletteInfo>>(Array.Empty<DebugPaletteInfo>());

        public IRealtimeProvider<IReadOnlyList<DebugAudioChannelInfo>> AudioChannels => new Fixed<IReadOnlyList<DebugAudioChannelInfo>>(Audio);

        public IRealtimeProvider<IReadOnlyList<DebugLoadInfo>> HardwareLoad => new Fixed<IReadOnlyList<DebugLoadInfo>>(Load);

        public void RefreshProviders() { }

        public (byte[] Rgba, int Width, int Height) RenderTileSheet() => TileSheet;

        public (byte[] Rgba, int Width, int Height) RenderPaletteSwatch() => PaletteSwatch;

        public (short[] Samples, int SampleRate) GetAudioSamples() => (Array.Empty<short>(), 32000);

        public string GetSummaryText() => "fake";

        private static byte[] Solid(int width, int height)
        {
            var rgba = new byte[width * height * 4];
            for (int i = 0; i + 3 < rgba.Length; i += 4)
            {
                rgba[i] = (byte)(i % 251);
                rgba[i + 1] = 90;
                rgba[i + 2] = (byte)(255 - (i % 251));
                rgba[i + 3] = 255;
            }

            return rgba;
        }
    }
}
