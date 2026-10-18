using EmuSen.Crystal.Systems.N64;
using EmuSen.Cores.Native;
using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MercuryRT;
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
            Assert.True(MoonEngines.Available, MoonEngines.Report);
            using var core = MoonEngines.Create();
            core.LoadRom(Keep(SyntheticNesRom.WriteTemp(SyntheticNesRom.Build())));
            Assert.Equal((BitConverter.ToUInt32("MOON"u8), 6), Header(core));
            Assert.Equal(6, ((IStateFormat)core).StateVersion);
        }

        [Fact]
        public void MercuryRT_writes_Mercurys_header()
        {
            Assert.True(MercuryRtCore.Available, MercuryNative.Report);
            using var core = new MercuryRtCore();
            core.LoadRom(Keep(SyntheticGbRom.WriteTemp(SyntheticGbRom.Build())));
            Assert.Equal((MercuryRtCore.StateMagic, MercuryRtCore.StateFormatVersion), Header(core));
            Assert.Equal(MercuryRtCore.StateFormatVersion, ((IStateFormat)core).StateVersion);
        }

        [Fact]
        public void MarsRT_writes_Mars_header_for_a_state_and_a_snapshot()
        {
            Assert.True(MarsRt.Available, MarsRt.Report);
            using var core = new MarsRtTestCore() { Options = TestCores.NoBattery };
            string rom = Keep(Path.Combine(Path.GetTempPath(), $"mars-header-{Guid.NewGuid():N}.z64"));
            File.WriteAllBytes(rom, SyntheticN64System.Build());
            core.LoadRom(rom);
            Assert.Equal((MarsMachine.StateMagic, MarsMachine.StateKind), Header(core));
            Assert.Equal(MarsMachine.StateKind, core.StateVersion);

            using var snapshot = new MemoryStream();
            core.SaveSnapshot(snapshot);
            Assert.Equal(MarsMachine.StateMagic, BitConverter.ToUInt32(snapshot.ToArray(), 0));
            Assert.Equal(MarsMachine.SnapshotKind, BitConverter.ToInt32(snapshot.ToArray(), 4));
        }
    }
}
