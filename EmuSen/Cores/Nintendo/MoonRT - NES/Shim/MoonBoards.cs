using System;
using System.Collections.Generic;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Systems.Nes;

namespace EmuSen.Cores.Nintendo.MoonRT
{
    // The sixteen boards MoonRT builds, by iNES mapper number: each one's name from the NES pack, its mirroring and the registers `regs` shows, read from the state by path - see Moon_Debug.md §3.1, Moon_Memory.md §4.
    public static class MoonBoards
    {
        public sealed record Board(string Name, Func<StateLayout, byte[], NesMirroring, NesMirroring> Mirroring, (string Name, string Path, int Index, int Bits)[] Registers);

        private static readonly (string, string, int, int)[] None = Array.Empty<(string, string, int, int)>();

        // A board's name is the NES pack's; what MoonRT builds of it stays here until B4 - see EmuSen_Backend.md §13.5.
        private static string Name(int mapper) => NesSystem.BoardName(mapper)!;

        private static NesMirroring Header(StateLayout l, byte[] s, NesMirroring header) => header;

        private static NesMirroring Field(StateLayout l, byte[] s, string path) => (NesMirroring)l.Int(s, "Mapper." + path);

        private static NesMirroring Horizontal(StateLayout l, byte[] s, NesMirroring header, bool fourScreen) =>
            fourScreen && header == NesMirroring.FourScreen ? NesMirroring.FourScreen
            : l.Flag(s, "Mapper._horizontal") ? NesMirroring.Horizontal : NesMirroring.Vertical;

        private static (string, string, int, int) R(string name, string path, int bits, int index = 0) => (name, "Mapper." + path, index, bits);

        private static (string, string, int, int)[] Bytes(string prefix, string path, int count, int bits, int first = 0)
        {
            var list = new (string, string, int, int)[count];
            for (int i = 0; i < count; i++) list[i] = R($"{prefix}{first + i}", path, bits, first + i);
            return list;
        }

        private static readonly Dictionary<int, Board> ByMapper = new()
        {
            [0] = new(Name(0), Header, None),
            [1] = new(Name(1), (l, s, h) => (l.Int(s, "Mapper._control") & 3) switch { 0 => NesMirroring.SingleScreenLower, 1 => NesMirroring.SingleScreenUpper, 2 => NesMirroring.Vertical, _ => NesMirroring.Horizontal }, None),
            [2] = new(Name(2), Header, None),
            [3] = new(Name(3), Header, None),
            [4] = new(Name(4), (l, s, h) => h == NesMirroring.FourScreen ? NesMirroring.FourScreen : (l.Int(s, "Mapper._mirroring") & 1) != 0 ? NesMirroring.Horizontal : NesMirroring.Vertical,
                [R("BankSelect", "_bankSelect", 3), R("PrgMode", "_prgMode", 1), R("ChrMode", "_chrMode", 1), .. Bytes("R", "_registers", 8, 8),
                 R("IrqLatch", "_irqReloadValue", 8), R("IrqCounter", "_irqCounter", 8), R("IrqReload", "_irqReload", 1), R("IrqEnabled", "_irqEnabled", 1),
                 R("IrqPending", "_irqPending", 1), R("IrqsFired", "_irqsFired", 32), R("WramEnabled", "_wramEnabled", 1)]),
            [7] = new(Name(7), (l, s, h) => l.Flag(s, "Mapper._upperNametable") ? NesMirroring.SingleScreenUpper : NesMirroring.SingleScreenLower, None),
            [9] = new(Name(9), (l, s, h) => Horizontal(l, s, h, fourScreen: false),
                [R("PrgBank", "_prgBank", 4), R("LeftFD", "_leftBank", 5, 0), R("LeftFE", "_leftBank", 5, 1), R("RightFD", "_rightBank", 5, 0), R("RightFE", "_rightBank", 5, 1),
                 R("LeftLatch", "_leftLatch", 1), R("RightLatch", "_rightLatch", 1), R("Horizontal", "_horizontal", 1)]),
            [11] = new(Name(11), Header, None),
            [64] = new(Name(64), (l, s, h) => Horizontal(l, s, h, fourScreen: true),
                [R("BankSelect", "_bankSelect", 8), .. Bytes("R", "_registers", 10, 8), R("R15", "_registers", 8, 15), R("IrqLatch", "_irqReloadValue", 8),
                 R("IrqCounter", "_irqCounter", 8), R("IrqCycleMode", "_irqCycleMode", 1), R("IrqEnabled", "_irqEnabled", 1), R("IrqPending", "_irqPending", 1)]),
            [65] = new(Name(65), (l, s, h) => Horizontal(l, s, h, fourScreen: false),
                [R("Prg8000", "_prgBanks", 8, 0), R("PrgA000", "_prgBanks", 8, 1), R("PrgC000", "_prgBanks", 8, 2), R("IrqLatch", "_irqReloadValue", 16),
                 R("IrqCounter", "_irqCounter", 16), R("IrqEnabled", "_irqEnabled", 1), R("IrqPending", "_irqPending", 1), R("Horizontal", "_horizontal", 1)]),
            [66] = new(Name(66), Header, None),
            [67] = new(Name(67), (l, s, h) => Field(l, s, "_mirroring"),
                [R("PrgBank", "_prgBank", 8), .. Bytes("Chr", "_chrBanks", 4, 8), R("IrqCounter", "_irqCounter", 16), R("IrqEnabled", "_irqEnabled", 1), R("IrqPending", "_irqPending", 1)]),
            [68] = new(Name(68), (l, s, h) => Field(l, s, "_mirroring"),
                [R("PrgBank", "_prgBank", 3), .. Bytes("Chr", "_chrBanks", 4, 8), .. Bytes("Nt", "_nametableBanks", 2, 8),
                 R("ChrNametables", "_useChrForNametables", 1), R("PrgRamEnabled", "_prgRamEnabled", 1)]),
            [69] = new(Name(69), (l, s, h) => Field(l, s, "_mirroring"),
                [R("Command", "_command", 4), R("Prg8000", "_prgBanks", 6, 0), R("PrgA000", "_prgBanks", 6, 1), R("PrgC000", "_prgBanks", 6, 2), R("Window6000", "_windowControl", 8),
                 R("IrqCounter", "_irqCounter", 16), R("IrqEnabled", "_irqEnabled", 1), R("IrqCounting", "_irqCounterEnabled", 1), R("IrqPending", "_irqPending", 1)]),
            [71] = new(Name(71), (l, s, h) => l.Flag(s, "Mapper._mirroringControlSeen") ? (l.Flag(s, "Mapper._upperPage") ? NesMirroring.SingleScreenUpper : NesMirroring.SingleScreenLower) : h, None),
            [79] = new(Name(79), Header, None),
        };

        public static bool IsImplemented(int mapper) => ByMapper.ContainsKey(mapper);

        public static Board? For(int mapper) => ByMapper.GetValueOrDefault(mapper);

        public static IEnumerable<int> Mappers => ByMapper.Keys;
    }
}
