using System;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // AccuracyCoin on MoonRT: the recorded baseline, result byte for result byte - see Moon_Native.md §3.4.1.
    public class MoonAccuracyCoinTests
    {
        // The path of AccuracyCoin.nes, a third-party ROM never committed; absent, the baseline passes unrun.
        public const string RomVariable = "EMUSEN_ACCURACYCOIN";

        // $0400-$0495 after the table is drawn at AccuracyCoin 673ef55, recorded on the C# core and matched by MoonRT since (Moon_Native.md §3.14).
        public const string Baseline =
            "000000010101010101010101010101010101010101010101000101010101010101000101010101010101010101010101010101010101010101010101010101010101010101010505050101010101010101010101010101010A01061A090105010101010100010101010101010101010101010101010141010901050A010101010101010E0A01010A05160A010A060A0E010A0A0A010E";

        private readonly ITestOutputHelper _output;

        public MoonAccuracyCoinTests(ITestOutputHelper output) => _output = output;

        [Fact]
        public void A_result_byte_decodes_as_the_rom_encodes_it()
        {
            var pass = new AccuracyCoinResult(0x403, 0x01);
            var behaviour = new AccuracyCoinResult(0x476, 0x41);
            var fail = new AccuracyCoinResult(0x446, 0x1E);
            var skip = new AccuracyCoinResult(0x450, 0xFF);
            Assert.True(pass.Passed && !pass.Failed && pass.Code == 0);
            Assert.True(behaviour.Passed && behaviour.Code == 16);
            Assert.True(fail.Failed && !fail.Passed && fail.Code == 7);
            Assert.True(skip.Skipped && !skip.Passed && !skip.Failed);

            var run = AccuracyCoinRun.Run(() => { }, a => a == AccuracyCoinRun.MenuProgress ? (byte)AccuracyCoinRun.MenuReady : (byte)0, _ => { }, budget: 10);
            Assert.Equal(1, run.MenuFrame);
            Assert.Equal(-1, run.TableFrame);
            Assert.Empty(run.Results);
        }

        // MoonRT's run: every result byte, and the frames the menu is ready and the table drawn, as recorded.
        [Fact]
        public void MoonRT_scores_its_recorded_baseline()
        {
            string? path = Environment.GetEnvironmentVariable(RomVariable);
            if (path is null || !File.Exists(path))
            {
                _output.WriteLine($"{RomVariable} unset, not run");
                return;
            }
            Assert.True(MoonRtCore.Available, "MoonRT is not built");
            CoreOptions.BatteryRamDisabled = true;
            using var core = new MoonRtCore();
            core.LoadRom(path);
            var run = AccuracyCoinRun.Run(core.RunFrame, a => core.ReadSpace("RAM", a), down => core.SetButton(0, PadButton.Start, down));
            _output.WriteLine($"menu at frame {run.MenuFrame}, table at {run.TableFrame}: {run.PassedTally} of {run.Tested} passed");
            var expected = Convert.FromHexString(Baseline);
            foreach (var r in run.Results) if (expected[r.Address - AccuracyCoinRun.FirstResult] != r.Raw) _output.WriteLine($"${r.Address:X4}: recorded {expected[r.Address - AccuracyCoinRun.FirstResult]:X2}, now {r.Raw:X2}");
            Assert.Equal((24, 3650), (run.MenuFrame, run.TableFrame));
            Assert.Equal((144, 127, 0), (run.Tested, run.PassedTally, run.SkippedTally));
            Assert.Equal(Baseline, Convert.ToHexString(run.Block));
        }
    }
}
