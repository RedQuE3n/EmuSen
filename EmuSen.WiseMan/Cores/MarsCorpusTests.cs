using System.Text;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Rom;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The hardware corpus, run for real - see Mars_TestOracle.md and Mars_Boot.md.
    public class MarsCorpusTests
    {
        private const int InstructionBudget = 1_000_000;

        // The handoff, the cartridge's own boot code, and the jump into the program it loaded.
        [Fact]
        public void The_corpus_rom_boots_through_its_own_bootcode_into_its_entry_point()
        {
            string? path = N64TestRomLibrary.FindSystemTest();
            if (path is null) return;

            var rom = RomImage.Load(path);
            var bus = new MarsBus();
            var cpu = new Cpu(bus);
            Boot.HandOff(bus, cpu, rom);

            ulong entry = 0xFFFF_FFFF_0000_0000UL | rom.EntryPoint;
            int steps = 0;

            while (steps < InstructionBudget && cpu.CurrentPc != entry && cpu.LastException is null)
            {
                cpu.Step();
                steps++;
            }

            Assert.Null(cpu.LastException);
            Assert.Equal(entry, cpu.CurrentPc);

            // Recorded rather than asserted loosely: the bootcode is real work, not a jump.
            Assert.InRange(steps, 1_000, 50_000);
        }

        // Where the corpus stops today: its first instruction is a coprocessor-1 move - see Mars_Boot.md §4.
        [Fact]
        public void The_first_instruction_the_corpus_reaches_that_mars_cannot_run_is_a_floating_point_move()
        {
            string? path = N64TestRomLibrary.FindSystemTest();
            if (path is null) return;

            var bus = new MarsBus();
            var cpu = new Cpu(bus);
            Boot.HandOff(bus, cpu, RomImage.Load(path));

            int steps = 0;
            while (steps < InstructionBudget && cpu.LastException is null)
            {
                cpu.Step();
                steps++;
            }

            Assert.Equal(ExceptionCode.ReservedInstruction, cpu.LastException!.Code);

            uint word = bus.Read32((uint)(cpu.CurrentPc & 0x1FFF_FFFF));
            Assert.Equal(0x11u, word >> 26);
        }

        private static string Diagnostics(Cpu cpu, MarsBus bus, int steps)
        {
            var text = new StringBuilder();
            text.AppendLine($"no output after {steps} instructions");
            text.AppendLine($"pc={cpu.Pc:X16} currentPc={cpu.CurrentPc:X16}");
            text.AppendLine($"last exception: {cpu.LastException?.Code.ToString() ?? "none"} " +
                            $"at {cpu.LastException?.Address:X16} refill={cpu.LastException?.Refill}");
            text.AppendLine($"status={cpu.Cop0[Cpu.StatusRegister]:X8} cause={cpu.Cop0[Cpu.CauseRegister]:X8} " +
                            $"epc={cpu.Cop0[Cpu.ExceptionPcRegister]:X16}");
            text.AppendLine($"ra={cpu.Gpr[31]:X16} sp={cpu.Gpr[29]:X16}");

            ulong faulted = cpu.CurrentPc;
            if ((faulted >> 29 & 7) is 4 or 5)
            {
                uint word = bus.Read32((uint)(faulted & 0x1FFF_FFFF));
                text.AppendLine($"instruction at {faulted:X16} = {word:X8} (op=0x{word >> 26:X2} rs={(word >> 21) & 0x1F} " +
                                $"rt={(word >> 16) & 0x1F} rd={(word >> 11) & 0x1F} funct=0x{word & 0x3F:X2})");
                text.AppendLine($"rs value={cpu.Gpr[(word >> 21) & 0x1F]:X16}");
            }

            return text.ToString();
        }
    }
}
