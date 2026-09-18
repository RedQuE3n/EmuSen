using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.Cores.Nintendo.Mars.Debug
{
    // The machine's memories by name, read and written without disturbing anything - see Mars_Debug.md §4.
    public static class MarsDebugSpaces
    {
        public const string Rdram = "RDRAM";
        public const string Dmem = "DMEM";
        public const string Imem = "IMEM";
        public const string PifRam = "PIFRAM";
        public const string Rom = "ROM";

        // The processor's 32-bit virtual addresses, as an int's bits - see Mars_Debug.md §4.
        public const string Cpu = "CPU";

        public static byte Read(MarsCore core, string space, int address)
        {
            if (Array(core, space) is { Length: > 0 } bytes) return bytes[Wrap(address, bytes.Length)];
            if (space != Cpu || core.Bus is not { } bus) return 0;

            return TryPhysical(core, (uint)address, out uint physical) && Window(bus, physical, out byte[]? window, out int offset)
                ? window![offset]
                : (byte)0;
        }

        public static void Write(MarsCore core, string space, int address, byte value)
        {
            if (space == Rom) return;

            if (Array(core, space) is { Length: > 0 } bytes)
            {
                bytes[Wrap(address, bytes.Length)] = value;
                return;
            }

            if (space == Cpu && core.Bus is { } bus && TryPhysical(core, (uint)address, out uint physical) &&
                Window(bus, physical, out byte[]? window, out int offset) && window != bus.Cart?.Rom)
            {
                window![offset] = value;
            }
        }

        // Big-endian, as the machine stores it, for the frame log's wider reads.
        public static long ReadWidth(MarsCore core, string space, int address, int width)
        {
            long value = 0;
            for (int i = 0; i < width; i++) value = (value << 8) | Read(core, space, address + i);
            return value;
        }

        // Kernel mode's view, with the TLB consulted but never faulted - see Mars_Debug.md §4.
        public static bool TryPhysical(MarsCore core, uint address, out uint physical)
        {
            physical = 0;
            if (core.Cpu is not { } cpu) return false;

            ulong wide = unchecked((ulong)(int)address);
            Segment segment = Segments.Decode(wide, PrivilegeMode.Kernel, wide: false);

            if (segment.Access == SegmentAccess.Direct)
            {
                physical = segment.Physical;
                return true;
            }

            return segment.Access == SegmentAccess.Mapped &&
                   cpu.Tlb.TryTranslate(wide, cpu.Cop0[global::EmuSen.Cores.Nintendo.Mars.Cpu.Core.Cpu.EntryHiRegister], false, out physical, out _) == TlbResult.Mapped;
        }

        // Which memory a physical address names, or none for a register - see Mars_Debug.md §4.
        public static PhysicalAddress? Resolve(MarsCore core, uint physical)
        {
            if (core.Bus is not { } bus) return null;

            if (physical < bus.Rdram.Length) return new PhysicalAddress(Rdram, (int)physical);
            if (physical < MemoryMap.RdramRegistersBase) return new PhysicalAddress("RDRAM (not installed)", (int)physical, false);
            if (physical < MemoryMap.SpDmemBase) return new PhysicalAddress("RDRAM registers", (int)(physical - MemoryMap.RdramRegistersBase), false);

            if (physical < MemoryMap.SpRegistersBase)
            {
                uint local = (physical - MemoryMap.SpDmemBase) % (2 * MemoryMap.SpMemSize);
                return local < MemoryMap.SpMemSize
                    ? new PhysicalAddress(Dmem, (int)local)
                    : new PhysicalAddress(Imem, (int)(local - MemoryMap.SpMemSize));
            }

            if (physical < MemoryMap.CartDomain2Address1) return new PhysicalAddress("interface registers", (int)physical, false);
            if (physical < MemoryMap.CartDomain2Address2) return new PhysicalAddress("64DD", (int)physical, false);
            if (physical < MemoryMap.CartDomain1Address2) return new PhysicalAddress("save chip", (int)(physical - MemoryMap.CartDomain2Address2), false);
            if (physical >= MemoryMap.PifRamBase && physical < MemoryMap.PifRamBase + MemoryMap.PifRamSize) return new PhysicalAddress(PifRam, (int)(physical - MemoryMap.PifRamBase));
            if (physical < MemoryMap.PifRomBase) return new PhysicalAddress(Rom, (int)(physical - MemoryMap.CartDomain1Address2));

            return new PhysicalAddress("PIF ROM", (int)(physical - MemoryMap.PifRomBase), false);
        }

        private static byte[]? Array(MarsCore core, string space) => space switch
        {
            Rdram => core.Bus?.Rdram,
            Dmem => core.Bus?.SpDmem,
            Imem => core.Bus?.SpImem,
            PifRam => core.Bus?.PifRam,
            Rom => core.Rom?.Rom,
            _ => null,
        };

        // Memories only: a register can answer a read by changing, as the RSP's semaphore does - see Mars_Debug.md §4.
        private static bool Window(MemoryBus bus, uint physical, out byte[]? window, out int offset)
        {
            window = null;
            offset = 0;

            if (physical < bus.Rdram.Length)
            {
                window = bus.Rdram;
                offset = (int)physical;
            }
            else if (physical >= MemoryMap.SpDmemBase && physical < MemoryMap.SpRegistersBase)
            {
                uint local = (physical - MemoryMap.SpDmemBase) % (2 * MemoryMap.SpMemSize);
                window = local < MemoryMap.SpMemSize ? bus.SpDmem : bus.SpImem;
                offset = (int)(local % MemoryMap.SpMemSize);
            }
            else if (physical >= MemoryMap.PifRamBase && physical < MemoryMap.PifRamBase + MemoryMap.PifRamSize)
            {
                window = bus.PifRam;
                offset = (int)(physical - MemoryMap.PifRamBase);
            }
            else if (physical >= MemoryMap.CartDomain1Address2 && physical < MemoryMap.PifRomBase && bus.Cart is { } cart &&
                     physical - MemoryMap.CartDomain1Address2 < cart.Rom.Length)
            {
                window = cart.Rom;
                offset = (int)(physical - MemoryMap.CartDomain1Address2);
            }

            return window != null;
        }

        private static int Wrap(int address, int size) => ((address % size) + size) % size;
    }
}
