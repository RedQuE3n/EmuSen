using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // A disposed Rust shim takes no picture back into its pool, as MarsRT's does - see EmuSen_Settings_Reference.md §4.85.3.
    public class RustShimDisposeTests : IDisposable
    {
        private readonly List<string> _files = new();

        public void Dispose()
        {
            foreach (string path in _files) try { File.Delete(path); } catch (IOException) { }
        }

        private string Keep(string path)
        {
            _files.Add(path);
            return path;
        }

        // Lent before the core goes, returned after: dropped, and the pool holds nothing.
        private static void ReturnedAfterDispose(ICore core, FrameBufferLending lending)
        {
            core.RunFrame();
            byte[] kept = core.GetFrameBufferRgba();
            byte[] returned = core.GetFrameBufferRgba();
            ((IFrameBufferPool)core).ReturnFrameBuffer(returned);
            Assert.Equal(1, lending.Free);

            ((IDisposable)core).Dispose();
            Assert.Equal(0, lending.Free);
            ((IFrameBufferPool)core).ReturnFrameBuffer(kept);
            Assert.Equal(0, lending.Free);
            Assert.Equal(1, lending.Dropped);
        }

        [Fact]
        public void MoonRT_closes_its_lending_when_disposed()
        {
            Assert.True(MoonRtCore.Available, MoonNative.Report);
            var core = new MoonRtCore();
            core.LoadRom(Keep(SyntheticNesRom.WriteTemp(SyntheticNesRom.Build(patches: (0, new byte[] { 0x4C, 0x00, 0x80 })))));
            ReturnedAfterDispose(core, core.FrameBuffers);
        }

        [Fact]
        public void MercuryRT_closes_its_lending_when_disposed()
        {
            Assert.True(MercuryRtCore.Available, MercuryNative.Report);
            var core = new MercuryRtCore();
            core.LoadRom(Keep(SyntheticGbRom.WriteTemp(SyntheticGbRom.Build())));
            ReturnedAfterDispose(core, core.FrameBuffers);
        }
    }
}
