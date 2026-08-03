using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Cores.Nintendo.Venus.Processor;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.Cores.Nintendo.Venus.Debug
{
    // The 65816's half of the expression language - see `man eval`.
    public sealed class SnesExpressionContext : IExpressionContext
    {
        private readonly Cpu _cpu;
        private readonly MemoryBus _bus;
        private readonly LabelRegistry _labels;
        private readonly Func<long> _frameCount;
        private readonly Func<IReadOnlyList<IDebugMemorySpace>> _spaces;

        public SnesExpressionContext(Cpu cpu, MemoryBus bus, LabelRegistry labels, Func<long> frameCount, Func<IReadOnlyList<IDebugMemorySpace>> spaces)
        {
            _cpu = cpu;
            _bus = bus;
            _labels = labels;
            _frameCount = frameCount;
            _spaces = spaces;
        }

        private static readonly string[] Names =
        {
            "a", "x", "y", "s", "sp", "d", "dp", "pc", "pb", "pbr", "db", "dbr", "p", "e",
            "flag.c", "flag.z", "flag.i", "flag.d", "flag.x", "flag.m", "flag.v", "flag.n",
            "frame", "scanline", "cycle", "opaddr", "stackdepth",
        };

        public IReadOnlyList<string> SymbolNames
            => Names.Concat(_labels.All().Select(l => l.Name)).ToList();

        public bool TryGetSymbol(string name, out long value)
        {
            switch (name.ToLowerInvariant())
            {
                case "a": value = _cpu.A; return true;
                case "x": value = _cpu.X; return true;
                case "y": value = _cpu.Y; return true;
                case "s":
                case "sp": value = _cpu.S; return true;
                case "d":
                case "dp": value = _cpu.D; return true;
                case "pc": value = _cpu.PC; return true;
                case "pb":
                case "pbr": value = _cpu.PB; return true;
                case "db":
                case "dbr": value = _cpu.DB; return true;
                case "p": value = _cpu.P; return true;
                case "e": value = _cpu.E ? 1 : 0; return true;
                case "flag.c": value = Flag(CpuFlags.C); return true;
                case "flag.z": value = Flag(CpuFlags.Z); return true;
                case "flag.i": value = Flag(CpuFlags.I); return true;
                case "flag.d": value = Flag(CpuFlags.D); return true;
                case "flag.x": value = Flag(CpuFlags.X); return true;
                case "flag.m": value = Flag(CpuFlags.M); return true;
                case "flag.v": value = Flag(CpuFlags.V); return true;
                case "flag.n": value = Flag(CpuFlags.N); return true;
                case "frame": value = _frameCount(); return true;
                case "scanline": value = _bus.Ppu.CurrentScanline; return true;
                case "cycle": value = _bus.LineCycles; return true;
                // The instruction a halt sits in front of, as a 24-bit address.
                case "opaddr": value = (_cpu.LastInstructionPB << 16) | _cpu.LastInstructionPC; return true;
                case "stackdepth": value = _cpu.CallStack?.Depth ?? 0; return true;
            }

            if (_labels.TryGetAddress(name, out int labelAddress))
            {
                value = labelAddress;
                return true;
            }

            value = 0;
            return false;
        }

        private long Flag(CpuFlags flag) => (_cpu.P & (byte)flag) != 0 ? 1 : 0;

        public bool TryReadMemory(string? space, int address, int width, out long value)
        {
            value = 0;
            if (address < 0) return false;

            // Unnamed means the CPU bus, WRAM served from the array - see `man eval`.
            if (space == null)
            {
                for (int i = 0; i < width; i++) value |= (long)ReadCpuAddress(address + i) << (8 * i);
                return true;
            }

            var named = _spaces().FirstOrDefault(s => string.Equals(s.Name, space, StringComparison.OrdinalIgnoreCase));
            if (named == null) return false;
            for (int i = 0; i < width; i++)
            {
                int at = address + i;
                if (at < 0 || at >= named.Size) return false;
                value |= (long)named.Read(at) << (8 * i);
            }
            return true;
        }

        private byte ReadCpuAddress(int address)
        {
            if (TryWramOffset(address, out int offset)) return _bus.Ram[offset];
            return _bus.Read8((uint)(address & 0xFFFFFF));
        }

        private bool TryWramOffset(int address, out int offset)
        {
            int bank = (address >> 16) & 0xFF;
            int low = address & 0xFFFF;
            if (bank is 0x7E or 0x7F)
            {
                offset = ((bank - 0x7E) << 16) | low;
                return offset < _bus.Ram.Length;
            }
            if (low < 0x2000 && (bank <= 0x3F || (bank >= 0x80 && bank <= 0xBF)))
            {
                offset = low;
                return offset < _bus.Ram.Length;
            }
            offset = 0;
            return false;
        }

    }
}
