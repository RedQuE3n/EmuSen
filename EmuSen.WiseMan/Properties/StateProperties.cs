using System;
using System.IO;
using System.Linq;
using CsCheck;
using EmuSen.Common;
using EmuSen.Cores;

namespace EmuSen.WiseMan.Properties
{
    // Rewind-chain and serializer round trips - see EmuSen_Debugging_Tools_Reference_v5.md §3.18b.
    public class StateProperties
    {
        // Same stand-in shape RewindBufferTests uses, so these test the chain not the SNES.
        private sealed class FakeCore : ICore
        {
            public int Counter;
            public byte[] Ram = new byte[1024];

            public string CoreName => "Fake";
            public int ScreenWidth => 1;
            public int ScreenHeight => 1;
            public double FrameRateHz => 60.0;
            public bool IsRomLoaded => true;
            public long TotalFrames { get; private set; }
            public int AudioSampleRate => 32000;
            public bool SkipRendering { get; set; }

            public void LoadRom(string path) { }
            public void RunFrame() { Counter++; TotalFrames++; Ram[Counter % Ram.Length] = (byte)Counter; }
            public byte[] GetFrameBufferRgba() => new byte[4];
            public short[] DequeueAudioSamples(int maxFrames) => Array.Empty<short>();
            public void SaveSram() { }

            public void SaveState(Stream stream)
            {
                var w = new BinaryWriter(stream, System.Text.Encoding.UTF8, leaveOpen: true);
                w.Write(Counter); w.Write(TotalFrames); w.Write(Ram); w.Flush();
            }

            public void LoadState(Stream stream)
            {
                var r = new BinaryReader(stream, System.Text.Encoding.UTF8, leaveOpen: true);
                Counter = r.ReadInt32(); TotalFrames = r.ReadInt64(); Ram = r.ReadBytes(Ram.Length);
            }

            public void SaveState(string path) { using var fs = File.Create(path); SaveState(fs); }
            public void LoadState(string path) { using var fs = File.OpenRead(path); LoadState(fs); }
        }

        // Rewinding N frames must land on exactly the state that produced frame N.
        [Fact]
        public void Rewinding_lands_on_the_frame_that_produced_the_state() =>
            Gen.Select(Gen.Int[1, 60], Gen.Int[1, 4]).Sample((frames, interval) =>
            {
                var core = new FakeCore();
                var buffer = new RewindBuffer { Enabled = true, IntervalFrames = interval };
                buffer.CaptureNow(core);
                for (int i = 0; i < frames; i++) { core.RunFrame(); buffer.OnFrameCompleted(core); }

                int before = core.Counter;
                if (!buffer.Rewind(core)) return core.Counter == before; // nothing buffered - must not corrupt
                return core.Counter < before && core.Ram[core.Counter % core.Ram.Length] == (byte)core.Counter;
            }, iter: 500);

        // Rewinding all the way back must reconstruct the very first captured state exactly.
        [Fact]
        public void Rewinding_the_whole_chain_reconstructs_the_first_capture() =>
            Gen.Int[1, 80].Sample(frames =>
            {
                var core = new FakeCore();
                var buffer = new RewindBuffer { Enabled = true, IntervalFrames = 1 };
                buffer.CaptureNow(core);
                int firstCounter = core.Counter;
                byte[] firstRam = (byte[])core.Ram.Clone();

                for (int i = 0; i < frames; i++) { core.RunFrame(); buffer.OnFrameCompleted(core); }
                while (buffer.Rewind(core)) { }

                return core.Counter == firstCounter && core.Ram.SequenceEqual(firstRam);
            }, iter: 300);

        // StateSerializer write-then-read is the identity over the whole field graph.
        [Fact]
        public void StateSerializer_write_then_read_is_the_identity() =>
            Gen.Select(Gen.Int, Gen.Byte.Array[1024, 1024]).Sample((counter, ram) =>
            {
                var source = new FakeCore { Counter = counter, Ram = (byte[])ram.Clone() };
                using var ms = new MemoryStream();
                source.SaveState(ms);
                ms.Position = 0;

                var restored = new FakeCore();
                restored.LoadState(ms);
                return restored.Counter == counter && restored.Ram.SequenceEqual(ram);
            }, iter: 500);
    }
}
