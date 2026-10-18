using EmuSen.Cores.Native;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The link port as a one-way sink, and the verdict reader built on it - see Mercury_Memory.md §10; held on MercuryRT - see Mercury_Native.md §12, §15.
    public sealed class MercuryRtSerialTests : IDisposable
    {
        private readonly List<MercuryRig> _rigs = new();


        public void Dispose()
        {
            foreach (MercuryRig rig in _rigs) rig.Dispose();
        }

        private MercuryRig Load(byte cgbFlag = 0x00, params (int Offset, byte[] Bytes)[] patches)
        {
            var core = MercuryRig.Create(SyntheticGbRom.Build(cgbFlag: cgbFlag, patches: patches));
            _rigs.Add(core);
            return core;
        }

        // LD A,c / LDH ($01),A / LD A,$81 / LDH ($02),A per character, then spin.
        private static byte[] PrintProgram(string text)
        {
            var code = new List<byte>();
            foreach (char c in text)
            {
                code.AddRange(new byte[] { 0x3E, (byte)c, 0xE0, 0x01, 0x3E, 0x81, 0xE0, 0x02 });
            }

            code.AddRange(new byte[] { 0x18, 0xFE });
            return code.ToArray();
        }

        [Fact]
        public void An_internally_clocked_transfer_captures_the_byte_and_shifts_in_ones()
        {
            var core = Load();

            core.Write(0xFF01, 0x41);
            core.Write(0xFF02, 0x81);

            Assert.Equal(new byte[] { 0x41 }, core.SerialLog.ToArray());

            // Nothing is connected, so the byte clocked in from the wire is all ones.
            Assert.Equal(0xFF, core.Read(0xFF01));

            // The transfer is over, so the start bit has cleared.
            Assert.Equal(0x00, core.Read(0xFF02) & 0x80);
        }

        [Fact]
        public void An_internally_clocked_transfer_requests_the_serial_interrupt()
        {
            var core = Load();

            core.Write(0xFF01, 0x41);
            core.Write(0xFF02, 0x81);

            Assert.Equal(1 << MercuryRig.Serial, core.Field("Bus.InterruptFlags") & (1 << MercuryRig.Serial));
        }

        // The half that is deliberately not implemented: with no cable there is no external clock.
        [Fact]
        public void An_externally_clocked_transfer_never_completes()
        {
            var core = Load();

            core.Write(0xFF01, 0x41);
            core.Write(0xFF02, 0x80);

            Assert.Empty(core.SerialLog);

            // Still pending, and still holding the byte the ROM put there.
            Assert.Equal(0x80, core.Read(0xFF02) & 0x80);
            Assert.Equal(0x41, core.Read(0xFF01));
            Assert.Equal(0, core.Field("Bus.InterruptFlags") & (1 << MercuryRig.Serial));
        }

        [Fact]
        public void Unused_control_bits_read_back_as_ones_on_a_dmg()
        {
            var core = Load();

            core.Write(0xFF02, 0x00);
            Assert.Equal(0x7E, core.Read(0xFF02));
        }

        // Bit 1 is the CGB's clock-speed select, so it reads back as written rather than as one.
        [Fact]
        public void The_clock_speed_bit_is_real_on_a_cgb()
        {
            var core = Load(cgbFlag: 0xC0);

            core.Write(0xFF02, 0x02);
            Assert.Equal(0x7E, core.Read(0xFF02));

            core.Write(0xFF02, 0x00);
            Assert.Equal(0x7C, core.Read(0xFF02));
        }

        // End to end: real SM83 code, the real bus decode, and the verdict reader on top.
        [Fact]
        public void A_rom_that_prints_over_serial_is_read_back_as_text()
        {
            var core = Load(patches: (0, PrintProgram("Passed\n")));
            for (int i = 0; i < 4; i++) core.RunFrame();

            Assert.Equal("Passed\n", core.SerialText);
            Assert.Equal(TestRomVerdict.Passed, HardwareTestRomLibrary.ReadVerdict(core.SerialLog));
        }

        [Fact]
        public void A_rom_that_reports_a_failure_is_not_read_as_a_pass()
        {
            var core = Load(patches: (0, PrintProgram("Failed 3\n")));
            for (int i = 0; i < 4; i++) core.RunFrame();

            Assert.Equal(TestRomVerdict.Failed, HardwareTestRomLibrary.ReadVerdict(core.SerialLog));
        }

        // The log is an observation about the run, not part of it: a load leaves it as it was - see Mercury_Memory.md §10.2.
        [Fact]
        public void A_save_state_carries_the_port_registers_but_not_the_transcript()
        {
            var core = Load();

            core.Write(0xFF01, 0x41);
            core.Write(0xFF02, 0x81);
            core.Write(0xFF01, 0x5A);

            using var stream = new MemoryStream();
            core.SaveState(stream);

            core.Write(0xFF01, 0x42);
            core.Write(0xFF02, 0x81);
            core.Write(0xFF01, 0x00);

            stream.Position = 0;
            core.LoadState(stream);

            Assert.Equal(0x5A, core.Read(0xFF01));
            Assert.Equal(new byte[] { 0x41, 0x42 }, core.SerialLog);
        }

        // Loading the game again is the bus's reset.
        [Fact]
        public void Reset_clears_both_the_port_and_the_transcript()
        {
            var core = Load();

            core.Write(0xFF01, 0x41);
            core.Write(0xFF02, 0x81);
            core.Reload();

            Assert.Empty(core.SerialLog);
            Assert.Equal(0x00, core.Read(0xFF01));
        }
    }

    // The verdict reader's own cases, which run on no engine.
    public sealed class MercurySerialTests
    {
        [Theory]
        [InlineData(new byte[0], TestRomVerdict.NoVerdict)]
        [InlineData(new byte[] { 3, 5, 8, 13, 21, 34 }, TestRomVerdict.Passed)]
        [InlineData(new byte[] { 0x42, 0x42, 0x42, 0x42, 0x42, 0x42 }, TestRomVerdict.Failed)]
        [InlineData(new byte[] { (byte)'h', (byte)'i' }, TestRomVerdict.NoVerdict)]
        public void The_verdict_reader_knows_both_corpora(byte[] serial, TestRomVerdict expected)
        {
            Assert.Equal(expected, HardwareTestRomLibrary.ReadVerdict(serial));
        }

        // A blargg summary lists each failing subtest and *then* says Failed, so "Passed" can appear in a failing log.
        [Fact]
        public void A_failure_wins_over_an_earlier_pass_line_in_the_same_log()
        {
            byte[] serial = System.Text.Encoding.ASCII.GetBytes("01:ok  02:Passed  03:Failed\n");
            Assert.Equal(TestRomVerdict.Failed, HardwareTestRomLibrary.ReadVerdict(serial));
        }
    }
}
