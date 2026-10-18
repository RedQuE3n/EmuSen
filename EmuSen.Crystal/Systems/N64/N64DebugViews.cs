using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Telemetry;

namespace EmuSen.Crystal.Systems.N64
{
    // The N64's views for any engine of the console, through N64Views: each space's code, the physical map, and the summary's lines - see EmuSen_Backend.md §18.10.
    public sealed class N64DebugViews : ISystemDebugViews
    {
        public static readonly N64DebugViews Instance = new();

        private N64DebugViews() { }

        public string? Summary(IReadOnlyList<DebugRegisterValue> registers, IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units) =>
            N64Views.Summary(registers, units.FirstOrDefault(u => u.Kind == "video" && string.Equals(u.Name, "VI", StringComparison.OrdinalIgnoreCase))?.Registers);

        public PhysicalAddress? ResolvePhysical(int address, IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units) =>
            N64Views.Physical(unchecked((uint)address), SystemDebugSpaces.Find(spaces, N64System.Rdram)?.Size ?? 0);

        public IReadOnlyList<(DisassembledInstruction Instruction, (StaticReferenceKind Kind, int Target)? Reference)> Disassemble(IDebugMemorySpace space, int address, int count) =>
            N64Views.Disassemble(space, address, count).Select(l => (l.Instruction, l.Decoded.Kind is { } kind ? ((StaticReferenceKind, int)?)(kind, unchecked((int)l.Decoded.Target)) : null)).ToArray();
    }
}
