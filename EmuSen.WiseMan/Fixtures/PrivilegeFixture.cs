using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;

namespace EmuSen.WiseMan.Fixtures
{
    // A machine running in a chosen mode, from a page every mode may execute - see Mars_Privilege.md §5.
    public static class PrivilegeFixture
    {
        // The program lives here virtually, mapped onto the physical page the assembler wrote it to.
        public const ulong ProgramPage = 0x0000_0000_0001_0000;

        private const ulong ExtendedAddressing = (1UL << 5) | (1UL << 6) | (1UL << 7);
        private const int KsuShift = 3;

        public static Cpu Load(int mode, bool wide, ulong address, MarsBus? bus = null) =>
            Run(mode, wide, address, a => a.Lw(2, 1, 0), bus);

        public static Cpu Store(int mode, bool wide, ulong address, MarsBus? bus = null) =>
            Run(mode, wide, address, a => a.Sw(2, 1, 0), bus);

        // The three outcomes the corpus's table distinguishes, under the names it gives them.
        public static string Describe(Cpu cpu) => cpu.LastException?.Code switch
        {
            null => "Sys",
            ExceptionCode.TlbLoad => "TLBL",
            ExceptionCode.AddressErrorLoad => "AdEL",
            var other => other.ToString()!,
        };

        public static Cpu Run(
            int mode, bool wide, ulong address, System.Func<MipsAssembler, MipsAssembler> program,
            MarsBus? bus = null, ulong status = 0)
        {
            var cpu = program(new MipsAssembler()).Build(bus ?? new MarsBus());

            MapProgramPage(cpu);

            cpu.Gpr[1] = address;
            cpu.Cop0[Cpu.StatusRegister] = status | ((ulong)mode << KsuShift) | (wide ? ExtendedAddressing : 0);
            cpu.Pc = ProgramPage;
            cpu.NextPc = ProgramPage + 4;
            cpu.Run(1);

            return cpu;
        }

        // One pair, deliberately not the pair containing the addresses under test - see Mars_Privilege.md §5.
        private static void MapProgramPage(Cpu cpu)
        {
            const ulong Usable = Tlb.EntryLoValid | Tlb.EntryLoDirty | Tlb.EntryLoGlobal;

            cpu.Tlb.Entries[0] = new TlbEntry
            {
                EntryHi = ProgramPage,
                PageMask = 0,
                EntryLo0 = Usable,
                EntryLo1 = (1UL << 6) | Usable,
            };
        }
    }
}
