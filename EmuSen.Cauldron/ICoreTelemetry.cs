using System.Collections.Generic;

namespace EmuSen.Cauldron
{
    // Everything a live dashboard can read from a running core, and nothing that writes to one - see EmuSen_Cauldron.md §3.
    public interface ICoreTelemetry
    {
        // Short console name for display ("SNES", "NES", ...).
        string CoreName { get; }

        // Monotonic frame counter - the shared "what moment is this" reference.
        long FrameCount { get; }

        // Real OAM capacity, or 0 when this core models no fixed limit - see §3.
        int MaxSprites { get; }

        IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CpuRegisters { get; }
        IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> VideoRegisters { get; }

        // The sound coprocessor's registers plus the CPU<->APU ports - see §3.
        IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> ApuRegisters { get; }

        // Whatever cartridge coprocessor is present; reads must be side-effect-free - see §3.
        IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CoprocessorRegisters { get; }

        IRealtimeProvider<IReadOnlyList<DebugSpriteInfo>> Sprites { get; }
        IRealtimeProvider<IReadOnlyList<DebugPaletteInfo>> Palettes { get; }
        IRealtimeProvider<IReadOnlyList<DebugAudioChannelInfo>> AudioChannels { get; }

        // Per-subsystem load bars; an empty list means this core models none - see §3.
        IRealtimeProvider<IReadOnlyList<DebugLoadInfo>> HardwareLoad { get; }

        // Republishes every provider above. Host-driven, emulation thread only - see §5.
        void RefreshProviders();

        // Tile memory decoded to an RGBA sheet, or 0x0 when a core has nothing analogous - see §3.
        (byte[] Rgba, int Width, int Height) RenderTileSheet();

        // Palette memory as an RGBA swatch grid, or 0x0 when a core has none - see §3.
        (byte[] Rgba, int Width, int Height) RenderPaletteSwatch();

        // Non-destructive copy of the buffered output samples - see §3.
        (short[] Samples, int SampleRate) GetAudioSamples();

        // Free-text core state, for whatever is too core-specific to model - see §3.
        string GetSummaryText();
    }
}
