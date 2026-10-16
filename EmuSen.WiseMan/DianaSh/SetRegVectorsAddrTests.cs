using EmuSen.Cores;
using System.Linq;
using EmuSen.DianaSh;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Debugger;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaSh
{
    // Writing registers, dereferencing vectors and decoding addresses - see `man setreg`, `man vectors`, `man addr`.
    public class SetRegVectorsAddrTests
    {
        private static IDebugTarget BuildTarget()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            return SyntheticRom.TargetOf(core);
        }

        private static DianaShInterpreter Shell(IDebugTarget target) => DianaShInterpreter.CreateDefault(target);

        private static ulong Register(IDebugTarget target, string cpu, string name)
            => DebugCpus.Find(target.DebugCpus, cpu)!.Registers!.Current
                .Single(r => r.Name.Equals(name, System.StringComparison.OrdinalIgnoreCase)).Value;

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
        public void Addr_rejects_a_non_address()
        {
            var target = BuildTarget();

            Assert.Contains("is not an address", Shell(target).Submit("addr zzz").Output);
        }
    }
}
