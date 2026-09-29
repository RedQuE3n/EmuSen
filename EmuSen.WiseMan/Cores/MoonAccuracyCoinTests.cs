using System;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // AccuracyCoin on the C# Moon: the recorded baseline, result byte for result byte - see Moon_Native.md §3.4.1.
    public class MoonAccuracyCoinTests
    {
        // The path of AccuracyCoin.nes, a third-party ROM never committed; absent, the baseline passes unrun.
        public const string RomVariable = "EMUSEN_ACCURACYCOIN";

        // $0400-$0495 after the table is drawn at AccuracyCoin 673ef55, after stage 2b's DMA (Moon_Native.md §3.9).
        public const string Baseline =
            "000000010101010101010101010101010101010101010101000101010101010101000101010101010101010101010101010101010101010101010101010101010101010101011E1E1E1E1E01010101010101010606060606060606121201050101010A060001011E0101560A01010101010101010101410A0A0A0506010101010A0601060A010A0A050606010A060A0E0A06060A060E";

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

        [Fact]
        public void The_csharp_core_scores_its_recorded_baseline()
        {
            string? path = Environment.GetEnvironmentVariable(RomVariable);
            if (path is null || !File.Exists(path))
            {
                _output.WriteLine($"{RomVariable} unset, not run");
                return;
            }
            Assert.Equal(AccuracyCoinRun.RomSha256, Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(path))).ToLowerInvariant());

            CoreOptions.BatteryRamDisabled = true;
            var core = new MoonCore();
            core.LoadRom(path);
            var run = AccuracyCoinRun.Run(core.RunFrame, a => core.ReadSpace(MoonCore.SpaceRam, a), down => core.SetButton(0, PadButton.Start, down));

            _output.WriteLine($"menu at frame {run.MenuFrame}, table at {run.TableFrame}: {run.PassedTally} of {run.Tested} passed, {run.SkippedTally} skipped");
            foreach (var failed in run.Results.Where(r => r.Failed)) _output.WriteLine($"${failed.Address:X4} fail {failed.Code}");
            Assert.Equal((17, 3061), (run.MenuFrame, run.TableFrame));
            Assert.Equal((144, 102, 0), (run.Tested, run.PassedTally, run.SkippedTally));
            Assert.Equal((144, 102, 42), (run.Results.Count(), run.Results.Count(r => r.Passed), run.Results.Count(r => r.Failed)));
            Assert.Equal(Baseline, Convert.ToHexString(run.Block));
        }
        // MoonRT through the same run: every result byte, and the frame the table is drawn, as the C# core's.
        [Fact]
        public void MoonRT_scores_as_the_csharp_core_does()
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
            var run = AccuracyCoinRun.Run(core.RunFrame, a => core.ReadSpace(MoonCore.SpaceRam, a), down => core.SetButton(0, PadButton.Start, down));
            _output.WriteLine($"menu at frame {run.MenuFrame}, table at {run.TableFrame}: {run.PassedTally} of {run.Tested} passed");
            var expected = Convert.FromHexString(Baseline);
            foreach (var r in run.Results) if (expected[r.Address - AccuracyCoinRun.FirstResult] != r.Raw) _output.WriteLine($"${r.Address:X4}: C# {expected[r.Address - AccuracyCoinRun.FirstResult]:X2}, MoonRT {r.Raw:X2}");
            Assert.Equal((17, 3061), (run.MenuFrame, run.TableFrame));
            Assert.Equal(Baseline, Convert.ToHexString(run.Block));
        }
    }
}
