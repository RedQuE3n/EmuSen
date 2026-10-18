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

        // The same with the spaces and the units' registers, which a console whose summary needs more than the processor's reads - see EmuSen_CoreAPI.md §31.7.
        string? Summary(IReadOnlyList<DebugRegisterValue> registers, IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units) => Summary(registers);

        // The palettes and their swatch with the units' registers, which a console whose palettes are registers reads - see EmuSen_CoreAPI.md §31.7.
        IReadOnlyList<DebugPaletteInfo> Palettes(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units) => Palettes(spaces);

        (byte[] Rgba, int Width, int Height) PaletteSwatch(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units) => PaletteSwatch(spaces);

        IReadOnlyList<DebugDmaChannel> DmaChannels(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units) => DmaChannels(spaces);

        string DecodeTilemapEntry(IDebugMemorySpace space, int address, IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units) => DecodeTilemapEntry(space, address);

        // The console's sprite table's capacity, 0 for none.
        int MaxSprites => 0;

        IReadOnlyList<DebugSpriteInfo> Sprites(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units) => Array.Empty<DebugSpriteInfo>();

        // Each sound channel from its unit's registers, mutes bit n muting channel n.
        IReadOnlyList<DebugAudioChannelInfo> AudioChannels(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units, uint mutes) => Array.Empty<DebugAudioChannelInfo>();

        // What an address on the main processor's bus decodes to, after the core's own translation where it gives one; null where the console's map does not say.
        PhysicalAddress? ResolvePhysical(int address, IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units) => null;

        // The words a unit is shown under, as a cartridge's board by its name; null for the unit's own name.
        string? UnitLabel(DebugUnit unit) => null;

        // A space's code listed by the processor that runs it, with the address each instruction names, for an engine that decodes none; empty where the console's pack does not.
        IReadOnlyList<(DisassembledInstruction Instruction, (StaticReferenceKind Kind, int Target)? Reference)> Disassemble(IDebugMemorySpace space, int address, int count) =>
            Array.Empty<(DisassembledInstruction, (StaticReferenceKind, int)?)>();
    }

    // A part that runs no code and holds registers, as machine info's units name it with the values read this frame - see EmuSen_CoreAPI.md §31.7.
    public sealed record DebugUnit(uint Id, string Name, string Kind, IReadOnlyList<DebugRegisterValue> Registers);

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
