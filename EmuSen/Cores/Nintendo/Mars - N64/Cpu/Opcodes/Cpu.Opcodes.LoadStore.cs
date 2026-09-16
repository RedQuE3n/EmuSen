using System;
using EmuSen.Cores.Nintendo.Mars.Memory;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // Aligned loads and stores; the unaligned family is not here yet - see Mars_Cpu.md §7.
    public sealed partial class Cpu
    {
        private void Load(uint instruction, int size, bool signed)
        {
            ulong address = EffectiveAddress(instruction);
            RequireAlignment(address, size, ExceptionCode.AddressErrorLoad);

            uint physical = Translate(address);

            ulong value = size switch
            {
                1 => signed ? (ulong)(long)(sbyte)_bus.Read8(physical) : _bus.Read8(physical),
                2 => signed ? (ulong)(long)(short)_bus.Read16(physical) : _bus.Read16(physical),
                4 => signed ? (ulong)(long)(int)_bus.Read32(physical) : _bus.Read32(physical),
                _ => _bus.Read64(physical),
            };

            Write(Rt(instruction), value);
        }

        private void Store(uint instruction, int size)
        {
            ulong address = EffectiveAddress(instruction);
            RequireAlignment(address, size, ExceptionCode.AddressErrorStore);

            uint physical = Translate(address);
            ulong value = Read(Rt(instruction));

            switch (size)
            {
                case 1: _bus.Write8(physical, (byte)value); return;
                case 2: _bus.Write16(physical, (ushort)value); return;
                case 4: _bus.Write32(physical, (uint)value); return;
                default: _bus.Write64(physical, value); return;
            }
        }

        private ulong EffectiveAddress(uint instruction) =>
            unchecked(Read(Rs(instruction)) + (ulong)SignedImmediate(instruction));

        // A misaligned access faults before anything is read or written, so no register is half updated.
        private void RequireAlignment(ulong address, int size, ExceptionCode code)
        {
            if ((address & (ulong)(size - 1)) != 0) throw Raise(code, address);
        }
    }
}
