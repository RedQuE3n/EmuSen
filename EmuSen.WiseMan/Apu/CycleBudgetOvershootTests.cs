using EmuSen.Cores.Nintendo.Venus.Apu;

namespace EmuSen.WiseMan.Apu
{
    // The SPC700 must not run an instruction its cycle budget doesn't cover -
    // overshooting made its port writes visible to the 65816 early enough to
    // corrupt audio uploads whose handshake acks before reading the byte. See
    // Venus_APU.md §1.6 for the Illusion of Gaia black-screen hang it caused.
    public class CycleBudgetOvershootTests
    {
        // MOV $F4,Y (4 cycles) - the ack in IoG's relocated upload loop, and
        // the write whose early visibility opened the race.
        private const byte MovDpY = 0xCB;

        private static Spc700 AtProgram(params byte[] code)
        {
            var spc = new Spc700();
            spc.Reset();
            spc.IplRomEnabled = false;
            spc.PC = 0x0200;
            for (int i = 0; i < code.Length; i++) spc.Ram[0x0200 + i] = code[i];
            return spc;
        }

        [Fact]
        public void PeekStepCycles_reports_the_cost_without_executing_anything()
        {
            var spc = AtProgram(MovDpY, 0xF4);

            Assert.Equal(4, spc.PeekStepCycles());
            Assert.Equal(0x0200, spc.PC);
        }

        // Read8 clears the timer counters at $FD-$FF, so a peek must not use it.
        [Fact]
        public void PeekStepCycles_does_not_disturb_read_sensitive_registers()
        {
            var spc = AtProgram(MovDpY, 0xFD);
            spc.Write8(0x00F1, 0x01);  // timer 0 on
            spc.Write8(0x00FA, 0x01);  // fire every 128 cycles
            spc.CycleBudget = 400;
            while (spc.CycleBudget >= spc.PeekStepCycles()) spc.Step();

            spc.PeekStepCycles();

            Assert.NotEqual(0, spc.Read8(0x00FD));
        }

        [Fact]
        public void Budget_short_of_the_next_instruction_leaves_it_unexecuted()
        {
            var spc = AtProgram(MovDpY, 0xF4);
            spc.CycleBudget = 3; // one short of MOV $F4,Y

            while (spc.CycleBudget >= spc.PeekStepCycles()) spc.Step();

            Assert.Equal(0x0200, spc.PC);
            Assert.Equal(0x00, spc.ReadPort(0));
            Assert.Equal(3, spc.CycleBudget);
        }

        [Fact]
        public void Budget_covering_the_instruction_runs_it_and_leaves_no_debt()
        {
            var spc = AtProgram(MovDpY, 0xF4);
            spc.Y = 0x2A;
            spc.CycleBudget = 4;

            while (spc.CycleBudget >= spc.PeekStepCycles()) spc.Step();

            Assert.Equal(0x2A, spc.ReadPort(0));
            Assert.Equal(0, spc.CycleBudget);
        }

        // The unfixed loop condition (budget > 0) ran the whole instruction on a
        // single leftover cycle, publishing the ack 3 cycles before it was due.
        [Fact]
        public void One_leftover_cycle_is_not_enough_to_publish_a_port_write()
        {
            var spc = AtProgram(MovDpY, 0xF4);
            spc.Y = 0x2A;
            spc.CycleBudget = 1;

            while (spc.CycleBudget >= spc.PeekStepCycles()) spc.Step();

            Assert.Equal(0x00, spc.ReadPort(0));
        }
    }
}
