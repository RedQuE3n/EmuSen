using System;
using System.Buffers.Binary;
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

            uint physical = TranslateAccess(Mirrored(address, size), address);

            // An aligned RDRAM load is the bytes at the address, which the bus would also hand back - see Mars_Performance.md §17.
            byte[] rdram = _bus.Rdram;
            ulong raw = physical < (uint)rdram.Length ? ReadRdram(rdram, (int)physical, size) : _bus.Load(physical, size);
            ulong value = !signed ? raw : size switch
            {
                1 => (ulong)(long)(sbyte)raw,
                2 => (ulong)(long)(short)raw,
                4 => (ulong)(long)(int)raw,
                _ => raw,
            };

            Write(Rt(instruction), value);
        }

        private void Store(uint instruction, int size)
        {
            ulong address = EffectiveAddress(instruction);
            RequireAlignment(address, size, ExceptionCode.AddressErrorStore);

            uint physical = TranslateAccess(Mirrored(address, size), address, store: true);

            // An aligned RDRAM store is the bytes named, unless the MI repeats it or a watcher wants it reported - see Mars_Performance.md §17.
            byte[] rdram = _bus.Rdram;
            if (physical < (uint)rdram.Length && !_bus.Mi.Repeating && !_bus.StoresWatched)
            {
                WriteRdram(rdram, (int)physical, Read(Rt(instruction)), size);
                return;
            }

            // The whole register goes to the bus, because not every device takes only the bytes named - see Mars_Memory.md §2.4.
            _bus.Store(physical, Read(Rt(instruction)), size);
        }

        private static ulong ReadRdram(byte[] rdram, int at, int size) => size switch
        {
            1 => rdram[at],
            2 => BinaryPrimitives.ReadUInt16BigEndian(rdram.AsSpan(at)),
            4 => BinaryPrimitives.ReadUInt32BigEndian(rdram.AsSpan(at)),
            _ => BinaryPrimitives.ReadUInt64BigEndian(rdram.AsSpan(at)),
        };

        private static void WriteRdram(byte[] rdram, int at, ulong value, int size)
        {
            switch (size)
            {
                case 1: rdram[at] = (byte)value; return;
                case 2: BinaryPrimitives.WriteUInt16BigEndian(rdram.AsSpan(at), (ushort)value); return;
                case 4: BinaryPrimitives.WriteUInt32BigEndian(rdram.AsSpan(at), (uint)value); return;
                default: BinaryPrimitives.WriteUInt64BigEndian(rdram.AsSpan(at), value); return;
            }
        }

        // The pair a lock is built from: the load arms it, the store only lands if nothing disarmed it.
        private void LoadLinked(uint instruction, int size)
        {
            Load(instruction, size, signed: size == 4);

            LinkedFlag = true;

            // The physical line rather than the virtual address, and no store ever compares it - see Mars_Cop0.md §6.
            Cop0[LinkedAddressRegister] = Translate(EffectiveAddress(instruction)) >> 4;
        }

        private void StoreConditional(uint instruction, int size)
        {
            if (LinkedFlag) Store(instruction, size);

            Write(Rt(instruction), LinkedFlag ? 1UL : 0UL);
            LinkedFlag = false;
        }

        // The operation is already complete; what remains of it is the address check - see Mars_Cpu.md §13.
        private void Cache(uint instruction)
        {
            ulong address = EffectiveAddress(instruction);
            RequireAlignment(address, 4, ExceptionCode.AddressErrorLoad);

            Translate(address);
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
