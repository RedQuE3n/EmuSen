using EmuSen.Cores.Native;
using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Audio
{
    // CoreOptions.AudioBufferMaxSamples on every core that queues its own sound - see EmuSen_Settings_Reference.md §4.85.2.
    [Collection(TestCollections.ProcessGlobals)]
    public class AudioBufferLimitTests : IDisposable
    {
        private const int Limit = 4096;
        private static readonly CoreOptions Limited = new() { AudioBufferMaxSamples = Limit };
        private readonly List<string> _files = new();

        public AudioBufferLimitTests()
        {
            DataStore.OverrideDirectory = Path.Combine(Path.GetTempPath(), "EmuSenAudioLimit_" + Guid.NewGuid().ToString("N"));
        }

        public void Dispose()
        {
            try { Directory.Delete(DataStore.OverrideDirectory!, recursive: true); } catch (IOException) { }
            DataStore.OverrideDirectory = null;
            foreach (string path in _files) try { File.Delete(path); } catch (IOException) { }
        }

        private string Nes()
        {
            string path = SyntheticNesRom.WriteTemp(SyntheticNesRom.Build(patches: (0, new byte[] { 0x4C, 0x00, 0x80 })));
            _files.Add(path);
            return path;
        }

        private string Gb()
        {
            string path = SyntheticGbRom.WriteTemp(SyntheticGbRom.Build());
            _files.Add(path);
            return path;
        }

        // Thirty frames undrained are far more than the limit on every console.
        private static short[] Undrained(ICore core, string rom)
        {
            core.LoadRom(rom);
            for (int i = 0; i < 30; i++) core.RunFrame();
            short[] samples = core.DequeueAudioSamples(int.MaxValue);
            (core as IDisposable)?.Dispose();
            return samples;
        }

        [Fact]
        public void MoonRT_keeps_no_more_than_the_setting()
        {
            Assert.True(MoonEngines.Available, MoonEngines.Report);
            Assert.Equal(Limit, Undrained(MoonEngines.Create(Limited), Nes()).Length);
        }

        [Fact]
        public void MercuryRT_keeps_no_more_than_the_setting()
        {
            Assert.True(MercuryEngines.Available, MercuryEngines.Report);
            Assert.Equal(Limit, Undrained(MercuryEngines.Create(Limited), Gb()).Length);
        }

        // Venus honoured it before this pass; kept so all five are held to one rule.
        [Fact]
        public void Venus_keeps_no_more_than_the_setting()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank(), Limited);
            for (int i = 0; i < 30; i++) core.RunFrame();
            Assert.Equal(Limit, core.DequeueAudioSamples(int.MaxValue).Length);
        }

        // The synthetic system's audio interface, slower than the other consoles' sound, left undrained past the limit.
        [Fact]
        public void MarsRT_keeps_no_more_than_the_setting()
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            string path = SyntheticN64Rom.WriteTemp(SyntheticN64System.Build(rsp: false));
            _files.Add(path);
            using var core = new MarsRtCore { Options = Limited with { BatteryRamDisabled = true } };
            core.LoadRom(path);
            for (int i = 0; i < 120; i++) core.RunFrame();
            Assert.Equal(Limit, core.DequeueAudioSamples(int.MaxValue).Length);
        }

        // A lowered limit takes hold at the next pair; one raised leaves what is queued alone.
        [Fact]
        public void The_queue_trims_after_each_pair_until_within_the_limit()
        {
            var queue = new SampleQueue { Limit = 8 };
            for (short i = 0; i < 10; i++) queue.Enqueue(i, (short)-i);
            Assert.Equal(new short[] { 6, -6, 7, -7, 8, -8, 9, -9 }, queue.Peek());

            queue.Limit = 4;
            queue.Enqueue(10, -10);
            Assert.Equal(new short[] { 9, -9, 10, -10 }, queue.Peek());

            queue.Limit = 9;
            for (short i = 11; i < 20; i++) queue.Enqueue(i, (short)-i);
            Assert.Equal(8, queue.Count);
            Assert.Empty(queue.Drain(-1));
            Assert.Equal(new short[] { 16, -16 }, queue.Drain(1));
        }

        // Null keeps the default, which the options' limit starts at.
        [Fact]
        public void A_queue_without_a_limit_of_its_own_keeps_the_default()
        {
            var queue = new SampleQueue();
            Assert.Equal(SampleQueue.DefaultLimit, queue.MaxSamples);
            Assert.Equal(SampleQueue.DefaultLimit, CoreOptions.Default.AudioBufferMaxSamples);
        }
    }
}
