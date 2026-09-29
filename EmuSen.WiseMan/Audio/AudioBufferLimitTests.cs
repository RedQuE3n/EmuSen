using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Audio;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Audio
{
    // AudioSettings.AudioBufferMaxSamples on every core that queues its own sound, which only Venus honoured - see EmuSen_Settings_Reference.md §4.85.2.
    [Collection(TestCollections.ProcessGlobals)]
    public class AudioBufferLimitTests : IDisposable
    {
        private const int Limit = 4096;
        private readonly int _before = AudioSettings.AudioBufferMaxSamples;
        private readonly List<string> _files = new();

        public AudioBufferLimitTests() => AudioSettings.AudioBufferMaxSamples = Limit;

        public void Dispose()
        {
            AudioSettings.AudioBufferMaxSamples = _before;
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
        public void Moon_keeps_no_more_than_the_setting() => Assert.Equal(Limit, Undrained(new MoonCore(), Nes()).Length);

        [Fact]
        public void MoonRT_keeps_no_more_than_the_setting()
        {
            Assert.True(MoonRtCore.Available, MoonNative.Report);
            Assert.Equal(Limit, Undrained(new MoonRtCore(), Nes()).Length);
        }

        [Fact]
        public void Mercury_keeps_no_more_than_the_setting() => Assert.Equal(Limit, Undrained(new MercuryCore(), Gb()).Length);

        [Fact]
        public void MercuryRT_keeps_no_more_than_the_setting()
        {
            Assert.True(MercuryRtCore.Available, MercuryNative.Report);
            Assert.Equal(Limit, Undrained(new MercuryRtCore(), Gb()).Length);
        }

        // Venus honoured it before this pass; kept so all five are held to one rule.
        [Fact]
        public void Venus_keeps_no_more_than_the_setting()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            for (int i = 0; i < 30; i++) core.RunFrame();
            Assert.Equal(Limit, core.DequeueAudioSamples(int.MaxValue).Length);
        }

        // The audio interface played past the limit, as MarsAudioTests plays it.
        [Fact]
        public void Mars_keeps_no_more_than_the_setting()
        {
            var bus = new MemoryBus();
            bus.Write32(MemoryMap.AiBase + AiInterface.DacRate, 0);
            bus.Write32(MemoryMap.AiBase + AiInterface.Control, 1);
            bus.Write32(MemoryMap.AiBase + AiInterface.DramAddress, 0x0010_0000);
            bus.Write32(MemoryMap.AiBase + AiInterface.Length, 0x3_FFF8);

            long samples = Limit / 2 + 10;
            long period = Math.Max(1, AiInterface.ShortestPeriod);
            bus.Tick((samples * period * 93_750_000 + bus.Vi.VideoClock - 1) / bus.Vi.VideoClock);

            Assert.Equal(samples, bus.Ai.SamplesPlayed);
            Assert.Equal(Limit, bus.Ai.BufferedSamples);
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

        // Null follows the setting, read at every pair.
        [Fact]
        public void A_queue_without_a_limit_of_its_own_follows_the_setting()
        {
            var queue = new SampleQueue();
            Assert.Equal(Limit, queue.MaxSamples);
            AudioSettings.AudioBufferMaxSamples = 6;
            for (short i = 0; i < 5; i++) queue.Enqueue(i, i);
            Assert.Equal(6, queue.Count);
        }
    }
}
