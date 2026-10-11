using System;
using System.Collections.Generic;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Telemetry;

namespace EmuSen.Crystal.Systems
{
    // A console's debugger views, decoded from the spaces and registers any engine of it publishes; each default is "none" - see EmuSen_CoreAPI.md §31.
    public interface ISystemDebugViews
    {
        IReadOnlyList<DebugPaletteInfo> Palettes(IReadOnlyList<IDebugMemorySpace> spaces) => Array.Empty<DebugPaletteInfo>();

        (byte[] Rgba, int Width, int Height) PaletteSwatch(IReadOnlyList<IDebugMemorySpace> spaces) => (Array.Empty<byte>(), 0, 0);

        (byte[] Rgba, int Width, int Height) TileSheet(IReadOnlyList<IDebugMemorySpace> spaces) => (Array.Empty<byte>(), 0, 0);

        int TilemapEntryStride => 0;

        string DecodeTilemapEntry(IDebugMemorySpace space, int address) => "";

        byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp) => Array.Empty<byte>();

        IReadOnlyList<DebugDmaChannel> DmaChannels(IReadOnlyList<IDebugMemorySpace> spaces) => Array.Empty<DebugDmaChannel>();

        string? NameDmaDestination(byte register) => null;

        IReadOnlyList<InterruptVector> InterruptVectors => Array.Empty<InterruptVector>();

        // Expression symbols the console derives from the main processor's registers.
        IReadOnlyList<string> SymbolNames => Array.Empty<string>();

        long? Symbol(string name, IReadOnlyList<DebugRegisterValue> registers) => null;

        // Lines the console adds to a summary from the main processor's registers; null for none.
        string? Summary(IReadOnlyList<DebugRegisterValue> registers) => null;
    }

    public static class SystemDebugSpaces
    {
        public static IDebugMemorySpace? Find(IReadOnlyList<IDebugMemorySpace> spaces, string name)
        {
            foreach (var s in spaces) if (string.Equals(s.Name, name, StringComparison.OrdinalIgnoreCase)) return s;
            return null;
        }

        public static ulong? Register(IReadOnlyList<DebugRegisterValue> registers, string name)
        {
            foreach (var r in registers) if (string.Equals(r.Name, name, StringComparison.OrdinalIgnoreCase)) return r.Value;
            return null;
        }
    }
}
