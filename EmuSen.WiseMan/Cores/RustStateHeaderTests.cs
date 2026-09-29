using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Each Rust library's copy of its C# core's state magic and version, which Rust cannot reference, held to the C# constants - see EmuSen_Settings_Reference.md §4.85.5.
    public class RustStateHeaderTests : IDisposable
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

        private static (uint Magic, int Version) Header(ICore core)
        {
            using var stream = new MemoryStream();
            core.SaveState(stream);
            byte[] state = stream.ToArray();
            return (BitConverter.ToUInt32(state, 0), BitConverter.ToInt32(state, 4));
        }

        [Fact]
        public void MoonRT_writes_Moons_header()
        {
            Assert.True(MoonRtCore.Available, MoonNative.Report);
            using var core = new MoonRtCore();
            core.LoadRom(Keep(SyntheticNesRom.WriteTemp(SyntheticNesRom.Build())));
            Assert.Equal((MoonCore.StateMagic, MoonCore.StateVersion), Header(core));
            Assert.Equal(MoonCore.StateVersion, ((IStateFormat)core).StateVersion);
        }

        [Fact]
        public void MercuryRT_writes_Mercurys_header()
        {
            Assert.True(MercuryRtCore.Available, MercuryNative.Report);
            using var core = new MercuryRtCore();
            core.LoadRom(Keep(SyntheticGbRom.WriteTemp(SyntheticGbRom.Build())));
            Assert.Equal((MercuryCore.StateMagic, MercuryCore.StateVersion), Header(core));
            Assert.Equal(MercuryCore.StateVersion, ((IStateFormat)core).StateVersion);
        }

        [Fact]
        public void MarsRT_writes_Mars_header_for_a_state_and_a_snapshot()
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            using var core = new MarsRtCore(batteryRamDisabled: true);
            string rom = Keep(Path.Combine(Path.GetTempPath(), $"mars-header-{Guid.NewGuid():N}.z64"));
            File.WriteAllBytes(rom, SyntheticN64System.Build());
            core.LoadRom(rom);
            Assert.Equal((MarsCore.StateMagic, MarsCore.StateVersion), Header(core));
            Assert.Equal(MarsCore.StateVersion, core.StateVersion);

            using var snapshot = new MemoryStream();
            core.SaveSnapshot(snapshot);
            Assert.Equal(MarsCore.StateMagic, BitConverter.ToUInt32(snapshot.ToArray(), 0));
            Assert.Equal(MarsCore.SnapshotVersion, BitConverter.ToInt32(snapshot.ToArray(), 4));
        }
    }
}
