using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;
using static EmuSen.WiseMan.Cores.MercuryCommercialRom;

namespace EmuSen.WiseMan.Cores
{
    public class MercuryCommercialRomAudioTests
    {
        public static TheoryData<string> Cartridges => Roms;

        // Real music, not a synthetic trigger. Start is tapped because some games
        // never call their sound driver on the title screen - see Mercury_RealCartridges.md §4.
        [Theory]
        [MemberData(nameof(Cartridges))]
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
    }
}
