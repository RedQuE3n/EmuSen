using EmuSen.Cores.Nintendo.Venus.Memory;
using Gsu = EmuSen.Cores.Nintendo.Venus.Coprocessors.SuperFx.SuperFx;

namespace EmuSen.WiseMan.Coprocessors
{
    // Game Pak RAM the GSU touches itself never reaches the S-CPU's bus, so
    // `watch`/`counters`/`bp write` were blind to it - see Venus_SuperFX.md §8.4.
    public class SuperFxRamObservationTests
    {
        private const ushort Clsr = 0x3039, R15Low = 0x301E, R15High = 0x301F;

        private sealed class Recorder : IWriteObserver, IReadObserver
        {
            public readonly List<(string Space, int Address, byte Value)> Writes = new();
            public readonly List<(string Space, int Address, byte Value)> Reads = new();

            public void OnWrite(string space, int address, byte value) => Writes.Add((space, address, value));
            public void OnCoprocessorWrite(string space, int address, byte value) => Writes.Add((space, address, value));
            public void OnRead(string space, int address, byte value) => Reads.Add((space, address, value));
            public void OnCoprocessorRead(string space, int address, byte value) => Reads.Add((space, address, value));
        }

        private static (Gsu Gsu, Recorder Observed) RunProgram(byte[] program)
        {
            byte[] rom = new byte[0x10000];
            program.CopyTo(rom, 0);

            var gsu = new Gsu(rom, new byte[0x8000]);
            var observed = new Recorder();
            gsu.WriteObserver = observed;
            gsu.ReadObserver = observed;

            gsu.WriteRegister(Clsr, 0x01);
            gsu.WriteRegister(R15Low, 0x00);
            gsu.WriteRegister(R15High, 0x00);
            gsu.Run(20000);
            return (gsu, observed);
        }

        [Fact]
        public void A_store_the_gsu_makes_is_reported_as_gsuram()
        {
            var (_, observed) = RunProgram(
            [
                0xF1, 0x34, 0x12,   // IWT R1, #$1234
                0xF0, 0xAB, 0x00,   // IWT R0, #$00AB
                0x31,               // STW (R1)
                0x00,               // STOP
            ]);

            Assert.Equal(("GSURAM", 0x1234, (byte)0xAB), observed.Writes[0]);
            Assert.Equal(("GSURAM", 0x1235, (byte)0x00), observed.Writes[1]);
        }

        // The space name is what tells the two sides of one chip apart: the
        // S-CPU's own window onto this same array reports as SRAM.
        [Fact]
        public void A_load_the_gsu_makes_is_reported_as_gsuram()
        {
            var (gsu, observed) = RunProgram(
            [
                0xF1, 0x00, 0x02,   // IWT R1, #$0200
                0x41,               // LDW (R1)
                0x00,               // STOP
            ]);

            Assert.Contains(observed.Reads, e => e.Space == "GSURAM" && e.Address == 0x0200);
            Assert.Contains(observed.Reads, e => e.Space == "GSURAM" && e.Address == 0x0201);
            Assert.Empty(observed.Writes);
            Assert.Equal(0, gsu.R[0]);
        }

        // Without this the recorded site is whatever the S-CPU happened to be
        // running, which says nothing about a write the GSU made on its own.
        [Fact]
        public void The_reported_instruction_address_is_the_gsus_own_pc()
        {
            var (gsu, _) = RunProgram(
            [
                0xF1, 0x34, 0x12,   // IWT R1, #$1234
                0x31,               // STW (R1)
                0x00,               // STOP
            ]);

            Assert.Equal(0x000004, gsu.DebugInstructionAddress);
        }

        [Fact]
        public void A_chip_with_no_observer_attached_still_runs()
        {
            byte[] rom = new byte[0x10000];
            new byte[] { 0xF1, 0x34, 0x12, 0xF0, 0xAB, 0x00, 0x31, 0x00 }.CopyTo(rom, 0);

            var gsu = new Gsu(rom, new byte[0x8000]);
            gsu.WriteRegister(Clsr, 0x01);
            gsu.WriteRegister(R15Low, 0x00);
            gsu.WriteRegister(R15High, 0x00);
            gsu.Run(20000);

            Assert.Equal(0xAB, gsu.ReadRam(0x1234));
        }
    }
}
