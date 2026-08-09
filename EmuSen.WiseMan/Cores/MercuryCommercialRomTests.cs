using EmuSen.Common.Imaging;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Invariants a real cartridge can be held to without a reference emulator - see Mercury_RealCartridges.md.
    public class MercuryCommercialRomTests
    {
        // Long enough for every ROM tried so far to clear its logo and reach a title screen.
        private const int BootFrames = 420;

        // Long enough for a title screen tapped with Start to reach something that plays music.
        private const int PlayFrames = 900;

        public static TheoryData<string> Roms => CommercialRomLibrary.AsTheoryData(".gb", ".gbc");

        private static MercuryCore Load(string path)
        {
            // The battery save would make every run depend on the last one - see Mercury_RealCartridges.md §2.
            CoreOptions.BatteryRamDisabled = true;

            var core = new MercuryCore();
            core.LoadRom(path);
            return core;
        }

        private static ulong RunTo(MercuryCore core, int frames)
        {
            for (int i = 0; i < frames; i++) core.RunFrame();
            return FrameHash.Compute(core.GetFrameBufferRgba());
        }

        [Theory]
        [MemberData(nameof(Roms))]
        public void A_real_cartridge_boots_and_reaches_a_title_screen(string path)
        {
            if (path.Length == 0) return;

            var core = Load(path);
            RunTo(core, BootFrames);

            Assert.Equal(BootFrames, core.TotalFrames);

            // A frame of one flat colour means the PPU never drew anything the game asked for.
            byte[] frame = core.GetFrameBufferRgba();
            var shades = new HashSet<byte>();
            for (int i = 0; i < frame.Length; i += 4) shades.Add(frame[i]);

            Assert.True(shades.Count > 1, $"{Path.GetFileName(path)} rendered a single flat colour after {BootFrames} frames.");
        }

        // Two fresh runs of the same image must agree exactly, or nothing below means anything.
        [Theory]
        [MemberData(nameof(Roms))]
        public void The_same_cartridge_run_twice_produces_the_same_frame(string path)
        {
            if (path.Length == 0) return;

            ulong first = RunTo(Load(path), BootFrames);
            ulong second = RunTo(Load(path), BootFrames);

            Assert.Equal(first, second);
        }

        // The strongest check available without a reference: state that is not saved shows up here.
        [Theory]
        [MemberData(nameof(Roms))]
        public void A_save_state_taken_mid_game_resumes_into_the_same_future(string path)
        {
            if (path.Length == 0) return;

            const int resumeFrames = 120;

            var core = Load(path);
            RunTo(core, BootFrames);

            using var stream = new MemoryStream();
            core.SaveState(stream);

            ulong straightThrough = RunTo(core, resumeFrames);

            stream.Position = 0;
            core.LoadState(stream);
            ulong afterReload = RunTo(core, resumeFrames);

            Assert.Equal(straightThrough, afterReload);
        }

        // Fast-forward drops pixel writes and nothing else; the machine must not notice - see ICore.SkipRendering.
        [Theory]
        [MemberData(nameof(Roms))]
        public void Skipping_the_renderer_does_not_change_the_machine(string path)
        {
            if (path.Length == 0) return;

            Assert.Equal(RunAndHashMemory(path, skipRendering: false), RunAndHashMemory(path, skipRendering: true));
        }

        private static ulong RunAndHashMemory(string path, bool skipRendering)
        {
            var core = Load(path);
            core.SkipRendering = skipRendering;
            for (int i = 0; i < BootFrames; i++) core.RunFrame();

            var bus = core.Bus!;
            var state = new List<byte>(bus.Vram.Length + bus.Wram.Length + bus.Oam.Length + bus.HighRam.Length + 4);
            state.AddRange(bus.Vram);
            state.AddRange(bus.Wram);
            state.AddRange(bus.Oam);
            state.AddRange(bus.HighRam);

            // The window's own line counter is render-derived and deliberately kept live - see Mercury_Ppu.md §4.2.
            state.Add((byte)bus.Ppu.WindowLine);
            state.Add(bus.Ppu.Ly);
            state.Add(bus.Ppu.Lcdc);
            state.Add(bus.InterruptFlags);

            return FrameHash.Compute(state.ToArray());
        }

        // Real music, not a synthetic trigger. Start is tapped because some games
        // never call their sound driver on the title screen - see Mercury_RealCartridges.md §4.
        [Theory]
        [MemberData(nameof(Roms))]
        public void A_real_cartridge_plays_music_rather_than_holding_one_note(string path)
        {
            if (path.Length == 0) return;

            var core = Load(path);
            var samples = new List<short>();

            // A stuck note from the boot-time channel setup would satisfy "not silent" on its own,
            // so what is asserted is that the note *changes* - see Mercury_RealCartridges.md §4.1.
            var notes = new HashSet<(int, int, int, int)>();

            for (int frame = 0; frame < PlayFrames; frame++)
            {
                bool press = frame >= 150 && (frame / 8) % 2 == 0;
                core.SetButton(0, PadButton.Start, press);
                core.SetButton(0, PadButton.A, press);

                core.RunFrame();
                samples.AddRange(core.DequeueAudioSamples(4096));

                var apu = core.Bus!.Apu;
                notes.Add((apu.Pulse1.Frequency, apu.Pulse2.Frequency, apu.Wave.Frequency, apu.Noise.ClockShift));
            }

            string name = Path.GetFileName(path);

            // Opt-in, so the suite writes nothing by default - see Mercury_HardwareTests.md §6.
            if (AudioCapture.DumpDirectory is string folder)
            {
                Directory.CreateDirectory(folder);
                AudioCapture.WriteWav(
                    Path.Combine(folder, Path.ChangeExtension(name, ".wav")), samples, core.AudioSampleRate);
            }

            Assert.True(notes.Count > 4,
                $"{name} only ever held {notes.Count} distinct channel settings - its sound driver is not running.");

            Assert.Contains(samples, s => s != 0);

            // A DC offset that never got filtered shows up as a mean far from zero - see Mercury_Apu.md §5.2.
            Assert.InRange(samples.Average(s => (double)s), -1500, 1500);

            // Pinning at the rails means the mixer is clipping rather than mixing.
            int pinned = samples.Count(s => s is short.MinValue or short.MaxValue);
            Assert.True(pinned < samples.Count / 50, $"{name} pinned {pinned} of {samples.Count} samples at the rails.");
        }

        // The switch is documented as honoured by every core, and Mercury did not honour it - see §2.
        [Theory]
        [MemberData(nameof(Roms))]
        public void Disabling_the_battery_leaves_no_save_beside_the_rom(string path)
        {
            if (path.Length == 0) return;

            string savePath = Path.ChangeExtension(path, ".srm");
            File.Delete(savePath);

            var core = Load(path);
            for (int i = 0; i < 320; i++) core.RunFrame();
            core.SaveSram();

            Assert.False(File.Exists(savePath), $"{Path.GetFileName(path)} wrote {Path.GetFileName(savePath)} despite BatteryRamDisabled.");
        }
    }
}
