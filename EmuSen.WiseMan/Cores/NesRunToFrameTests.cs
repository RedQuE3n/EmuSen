using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // `runto frame` on both NES engines, which never told the registry a frame had ended - see EmuSen_Settings_Reference.md §4.85.1.
    public class NesRunToFrameTests : IDisposable
    {
        private readonly List<string> _files = new();

        public void Dispose()
        {
            foreach (string path in _files) try { File.Delete(path); } catch (IOException) { }
        }

        // JMP $8000 forever: every frame is the same loop, so where a halt lands is known.
        private string Loop()
        {
            string path = SyntheticNesRom.WriteTemp(SyntheticNesRom.Build(patches: (0, new byte[] { 0x4C, 0x00, 0x80 })));
            _files.Add(path);
            return path;
        }

        [Fact]
        public void Moon_halts_at_the_start_of_the_frame_after_the_one_run_to()
        {
            var core = new MoonCore();
            core.LoadRom(Loop());
            core.Breakpoints.ArmRunToFrame(2);

            core.RunFrame();
            core.RunFrame();
            Assert.False(core.IsHaltedAtBreakpoint);
            Assert.True(core.Breakpoints.HasPendingBreak);

            core.RunFrame();
            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(2, core.TotalFrames);
            Assert.Equal("frame 2 reached", core.Breakpoints.LastBreakReason);
        }

        // MoonRT's observed frame stops where Moon's does, since its stage 5 - see Moon_Native.md §8.4.
        [Fact]
        public void MoonRT_halts_at_the_start_of_the_frame_after_the_one_run_to()
        {
            Assert.True(MoonRtCore.Available, MoonNative.Report);
            using var core = new MoonRtCore();
            core.LoadRom(Loop());
            core.Breakpoints.ArmRunToFrame(2);

            core.RunFrame();
            core.RunFrame();
            Assert.False(core.IsHaltedAtBreakpoint);
            Assert.True(core.Breakpoints.HasPendingBreak);

            core.RunFrame();
            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(2, core.TotalFrames);
            Assert.Equal(0x8000, core.HaltedAddress);
            Assert.Equal("frame 2 reached", core.Breakpoints.LastBreakReason);
        }
    }
}
