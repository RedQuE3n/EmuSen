using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Moon.Validation;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The blargg-protocol harness, proved against ROMs this file builds - see Moon_TestRoms.md §4.
    public class MoonTestRomTests : IDisposable
    {
        private readonly List<string> _temporaryFiles = new();

        public void Dispose()
        {
            foreach (string path in _temporaryFiles)
            {
                try { File.Delete(path); } catch (IOException) { }
            }
        }

        private string Rom(byte[] image)
        {
            string path = SyntheticNesRom.WriteTemp(image);
            _temporaryFiles.Add(path);
            return path;
        }

        [Fact]
        public void A_passing_rom_reports_its_result_code_and_its_text()
        {
            var result = NesTestRomRunner.Run(Rom(SyntheticNesRom.BuildProtocolRom(0x00, "01-basics")));

            Assert.Equal(NesTestRomOutcome.Passed, result.Outcome);
            Assert.Equal(0, result.Code);
            Assert.Equal("01-basics", result.Text);
            Assert.Equal(0, result.Resets);
        }

        // The running status has to be held long enough that the poll actually sees it - see §2.3.
        [Fact]
        public void A_verdict_is_only_taken_after_the_running_status_has_been_observed()
        {
            var result = NesTestRomRunner.Run(Rom(SyntheticNesRom.BuildProtocolRom(0x00)));

            Assert.Equal(NesTestRomOutcome.Passed, result.Outcome);
            Assert.InRange(result.Frames, 2, 60);
        }

        [Fact]
        public void A_failing_rom_carries_its_code_out_rather_than_being_flattened_to_a_boolean()
        {
            var result = NesTestRomRunner.Run(Rom(SyntheticNesRom.BuildProtocolRom(0x03, "Wrong flags")));

            Assert.Equal(NesTestRomOutcome.Failed, result.Outcome);
            Assert.Equal(3, result.Code);
            Assert.Equal("Wrong flags", result.Text);
        }

        // A screen-only ROM, or one this core hangs, must not be mistaken for a pass.
        [Fact]
        public void A_rom_that_never_speaks_the_protocol_reports_no_verdict()
        {
            var result = NesTestRomRunner.Run(Rom(SyntheticNesRom.Build()), frameBudget: 30);

            Assert.Equal(NesTestRomOutcome.NoResult, result.Outcome);
            Assert.Equal(30, result.Frames);
        }

        [Fact]
        public void A_rom_that_asks_for_a_reset_gets_one_and_then_reports()
        {
            var result = NesTestRomRunner.Run(Rom(SyntheticNesRom.BuildResetProtocolRom()));

            Assert.Equal(NesTestRomOutcome.Passed, result.Outcome);
            Assert.Equal(1, result.Resets);
        }

        // An unimplemented board is a known gap, not a harness failure - see Moon_Memory.md §4.
        [Fact]
        public void An_unimplemented_mapper_is_reported_rather_than_thrown()
        {
            var result = NesTestRomRunner.Run(Rom(SyntheticNesRom.Build(mapper: 5)));

            Assert.Equal(NesTestRomOutcome.Unsupported, result.Outcome);
            Assert.Contains("5", result.Text);
        }

        [Fact]
        public void A_soft_reset_restarts_the_cpu_without_clearing_ram()
        {
            var core = new MoonCore();
            core.LoadRom(Rom(SyntheticNesRom.Build()));

            core.RunFrame();
            core.WriteSpace(MoonCore.SpaceRam, 0x0123, 0x5A);
            core.WriteSpace(MoonCore.SpacePrgRam, 0x6100, 0xA5);

            core.Reset();

            Assert.Equal(0x5A, core.ReadSpace(MoonCore.SpaceRam, 0x0123));
            Assert.Equal(0xA5, core.ReadSpace(MoonCore.SpacePrgRam, 0x6100));
            Assert.Equal(0x8000, core.Cpu!.PC);
        }

        // A hard reset is the power switch, and it does clear both.
        [Fact]
        public void A_hard_reset_clears_what_a_soft_one_keeps()
        {
            var core = new MoonCore();
            string path = Rom(SyntheticNesRom.Build());
            core.LoadRom(path);

            core.WriteSpace(MoonCore.SpaceRam, 0x0123, 0x5A);
            core.LoadRom(path);

            Assert.Equal(0x00, core.ReadSpace(MoonCore.SpaceRam, 0x0123));
        }

    }
}
