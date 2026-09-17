using System;
using System.IO;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Rom;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The hardware corpus, run for real - see Mars_Corpus.md, Mars_TestOracle.md and Mars_Boot.md.
    public class MarsCorpusTests
    {
        private const int InstructionBudget = 1_000_000;

        // Past where the run stops, so the tally is not a function of the budget - see Mars_Corpus.md §4.
        private const int VerdictBudget = 190_000_000;

        // The handoff, the cartridge's own boot code, and the jump into the program it loaded.
        [Fact]
        public void The_corpus_rom_boots_through_its_own_bootcode_into_its_entry_point()
        {
            if (!Installed(out var bus, out var cpu, out var rom)) return;

            ulong entry = 0xFFFF_FFFF_0000_0000UL | rom!.EntryPoint;
            int steps = 0;

            while (steps < InstructionBudget && cpu!.CurrentPc != entry && cpu.LastException is null)
            {
                cpu.Step();
                steps++;
            }

            Assert.Null(cpu!.LastException);
            Assert.Equal(entry, cpu.CurrentPc);

            // Recorded rather than asserted loosely: the bootcode is real work, not a jump.
            Assert.InRange(steps, 1_000, 50_000);
        }

        // The oracle speaking at all, which is the thing every graded claim about Mars now rests on.
        [Fact]
        public void The_corpus_reports_its_verdicts_through_the_debug_port()
        {
            if (!Installed(out var bus, out var cpu, out _)) return;

            RunUntilMarsRunsOut(cpu!);
            string verdicts = bus!.IsViewer.Text;

            string report = Report(verdicts);

            Assert.Contains("Running StartupTest...", verdicts);
            Assert.Contains("Running ADDIOpcodeTest...", verdicts);

            // A ratchet, not a description: both halves are asserted - see Mars_Corpus.md §5.
            Assert.Equal($"720 started, 167 failed ({report})", $"{Tally(verdicts)} ({report})");
        }

        // The inverse of what stood here for two slices: no scaffold left to reach - see Mars_Fpu.md §6.
        [Fact]
        public void The_corpus_no_longer_reaches_an_instruction_mars_has_not_built()
        {
            if (!Installed(out _, out var cpu, out _)) return;

            RunUntilMarsRunsOut(cpu!, rethrow: true);
        }

        private static void RunUntilMarsRunsOut(Cpu cpu, bool rethrow = false)
        {
            try
            {
                for (int steps = 0; steps < VerdictBudget; steps++) cpu.Step();
            }
            catch (NotImplementedException) when (!rethrow)
            {
            }
        }

        private static bool Installed(out MarsBus? bus, out Cpu? cpu, out RomImage? rom)
        {
            bus = null;
            cpu = null;
            rom = null;

            string? path = N64TestRomLibrary.FindSystemTest();
            if (path is null) return false;

            rom = RomImage.Load(path);
            bus = new MarsBus();
            cpu = new Cpu(bus);
            Boot.HandOff(bus, cpu, rom);

            return true;
        }

        private static int Occurrences(string text, string marker)
        {
            int count = 0;
            for (int i = text.IndexOf(marker, StringComparison.Ordinal); i >= 0;
                 i = text.IndexOf(marker, i + 1, StringComparison.Ordinal))
            {
                count++;
            }

            return count;
        }

        private static string Tally(string verdicts) =>
            $"{Occurrences(verdicts, "Running ")} started, {Occurrences(verdicts, "' failed:")} failed";

        // The whole report beside the test assembly, because the tally alone cannot say what moved.
        private static string Report(string verdicts)
        {
            string path = Path.Combine(AppContext.BaseDirectory, "mars-corpus-verdicts.txt");

            try
            {
                File.WriteAllText(path, verdicts);
                return path;
            }
            catch (IOException)
            {
                return "report not written";
            }
        }
    }
}
