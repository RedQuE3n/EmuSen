using System.Linq;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // Writing registers, dereferencing vectors and decoding addresses - see `man setreg`, `man vectors`, `man addr`.
    public class SetRegVectorsAddrTests
    {
        private static SnesDebugTarget BuildTarget()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            return new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
        }

        private static DianaOSInterpreter Shell(SnesDebugTarget target) => DianaOSInterpreter.CreateDefault(target);

        private static ulong Register(SnesDebugTarget target, string cpu, string name)
            => DebugCpus.Find(target.DebugCpus, cpu)!.Registers!.Current
                .Single(r => r.Name.Equals(name, System.StringComparison.OrdinalIgnoreCase)).Value;

        [Fact]
        public void Setreg_writes_a_main_cpu_register()
        {
            var target = BuildTarget();

            string output = Shell(target).Submit("setreg a 1234").Output;

            Assert.Contains("cpu A = 0x1234", output);
            Assert.Equal(0x1234UL, Register(target, "cpu", "A"));
        }

        [Fact]
        public void Setreg_reports_the_previous_value()
        {
            var target = BuildTarget();
            var shell = Shell(target);
            shell.Submit("setreg x 00AA");

            Assert.Contains("was 0x00AA", shell.Submit("setreg x 00BB").Output);
        }

        [Fact]
        public void Setreg_can_move_the_program_counter()
        {
            var target = BuildTarget();

            Shell(target).Submit("setreg pc 9000");

            Assert.Equal(0x9000UL, Register(target, "cpu", "PC"));
        }

        [Fact]
        public void Setreg_scopes_to_another_processor()
        {
            var target = BuildTarget();

            string output = Shell(target).Submit("setreg spc a 42").Output;

            Assert.Contains("spc A = 0x42", output);
            Assert.Equal(0x42UL, Register(target, "spc", "A"));
        }

        [Fact]
        public void Setreg_refuses_a_value_too_wide_for_the_register()
        {
            var target = BuildTarget();

            string output = Shell(target).Submit("setreg db 1FF").Output;

            Assert.Contains("8-bit", output);
            Assert.Contains("does not fit", output);
            Assert.NotEqual(0xFFUL, Register(target, "cpu", "DB"));
        }

        [Fact]
        public void Setreg_lists_the_real_registers_for_an_unknown_name()
        {
            var target = BuildTarget();

            string output = Shell(target).Submit("setreg wibble 1").Output;

            Assert.Contains("no register named", output);
            Assert.Contains("A", output);
        }

        [Fact]
        public void Setreg_refuses_a_non_numeric_value()
        {
            var target = BuildTarget();

            Assert.Contains("is not a value", Shell(target).Submit("setreg a zzz").Output);
        }

        [Fact]
        public void The_e_flag_is_writable_as_a_one_bit_register()
        {
            var target = BuildTarget();

            Shell(target).Submit("setreg e 1");

            Assert.Equal(1UL, Register(target, "cpu", "E"));
        }

        [Fact]
        public void Cpus_reports_setreg_only_for_writable_chips()
        {
            var target = BuildTarget();

            string listing = Shell(target).Submit("cpus").Output;

            // A blank synthetic cartridge carries no coprocessor, so cpu and spc are the whole list.
            Assert.Contains("setreg", listing);
            Assert.All(target.DebugCpus, cpu => Assert.NotNull(cpu.RegisterWriter));
        }

        [Fact]
        public void Vectors_dereferences_the_table_and_groups_by_mode()
        {
            var target = BuildTarget();

            string output = Shell(target).Submit("vectors").Output;

            Assert.Contains("Native (E=0)", output);
            Assert.Contains("Emulation (E=1)", output);
            Assert.Contains("NMI", output);
            Assert.Contains("RESET", output);
            Assert.Contains("$00FFEA", output);
        }

        [Fact]
        public void Vectors_reads_the_pointer_live_from_the_bus()
        {
            var target = BuildTarget();
            var bus = target.GetMemorySpaces().Single(s => s.Name == "CpuBus");

            var reset = target.InterruptVectors.Single(v => v.Name == "RESET");
            int expected = bus.Read(reset.Address) | (bus.Read(reset.Address + 1) << 8);

            Assert.Contains($"${expected:X6}", Shell(target).Submit("vectors").Output);
        }

        [Fact]
        public void Vectors_marks_a_target_that_actually_ran()
        {
            var target = BuildTarget();
            var shell = Shell(target);
            shell.Submit("cov on");

            var bus = target.GetMemorySpaces().Single(s => s.Name == "CpuBus");
            var reset = target.InterruptVectors.Single(v => v.Name == "RESET");
            target.Coverage!.Record(bus.Read(reset.Address) | (bus.Read(reset.Address + 1) << 8));

            Assert.Contains("(ran)", shell.Submit("vectors").Output);
        }

        [Fact]
        public void Addr_decodes_a_rom_address_to_a_file_offset()
        {
            var target = BuildTarget();

            string output = Shell(target).Submit("addr 808000").Output;

            Assert.Contains("ROM", output);
            Assert.Contains("mem ROM", output);
        }

        [Fact]
        public void Addr_decodes_wram_both_directly_and_through_its_mirror()
        {
            var target = BuildTarget();
            var shell = Shell(target);

            Assert.Contains("WRAM 0xDB3", shell.Submit("addr 7E0DB3").Output);
            Assert.Contains("WRAM 0x10", shell.Submit("addr 000010").Output);
        }

        [Fact]
        public void Addr_names_a_hardware_register_rather_than_inventing_an_offset()
        {
            var target = BuildTarget();

            string output = Shell(target).Submit("addr 002100").Output;

            Assert.Contains("register", output);
            Assert.DoesNotContain("mem ", output);
        }

        [Fact]
        public void Addr_rejects_a_non_address()
        {
            var target = BuildTarget();

            Assert.Contains("is not an address", Shell(target).Submit("addr zzz").Output);
        }
    }
}
