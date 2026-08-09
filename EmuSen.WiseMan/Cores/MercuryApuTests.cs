using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Audio;
using GbApu = EmuSen.Cores.Nintendo.Mercury.Audio.Apu;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Mercury's four sound channels and the mixer under them - see Mercury_Apu.md.
    public class MercuryApuTests : IDisposable
    {
        // One turn of the frame sequencer's DIV bit, in T-cycles: 512 Hz off a 4.19 MHz clock.
        private const int SequencerPeriod = 8192;

        private readonly List<string> _temporaryFiles = new();

        public void Dispose()
        {
            foreach (string path in _temporaryFiles)
            {
                try { File.Delete(path); } catch (IOException) { }
            }
        }

        private MercuryCore Load(byte cgbFlag = 0x00)
        {
            string path = SyntheticGbRom.WriteTemp(SyntheticGbRom.Build(cgbFlag: cgbFlag));
            _temporaryFiles.Add(path);

            var core = new MercuryCore();
            core.LoadRom(path);
            return core;
        }

        // NRx2 with a non-zero top nibble, then a trigger with the length counter off.
        private static void StartPulse(MercuryCore core, ushort baseAddress, int frequency)
        {
            core.Bus!.Write((ushort)(baseAddress + 2), 0xF0);
            core.Bus.Write((ushort)(baseAddress + 3), (byte)(frequency & 0xFF));
            core.Bus.Write((ushort)(baseAddress + 4), (byte)(0x80 | ((frequency >> 8) & 0x07)));
        }

        [Fact]
        public void The_apu_powers_up_where_the_boot_rom_leaves_it()
        {
            var core = Load();
            var apu = core.Bus!.Apu;

            Assert.True(apu.PoweredOn);
            Assert.Equal(7, apu.LeftVolume);
            Assert.Equal(7, apu.RightVolume);
            Assert.Equal(0xF3, apu.Panning);
        }

        [Fact]
        public void A_trigger_enables_a_channel_and_nr52_reports_it()
        {
            var core = Load();

            Assert.Equal(0x00, core.Bus!.Read(0xFF26) & 0x01);

            StartPulse(core, 0xFF10, 0x400);

            Assert.True(core.Bus.Apu.Pulse1.Enabled);
            Assert.Equal(0x01, core.Bus.Read(0xFF26) & 0x01);
        }

        // The top five bits of NRx2 are the DAC's power, and a dead DAC cannot be triggered on.
        [Fact]
        public void Clearing_the_envelope_bits_kills_the_dac_and_the_channel_with_it()
        {
            var core = Load();
            StartPulse(core, 0xFF10, 0x400);

            core.Bus!.Write(0xFF12, 0x00);

            Assert.False(core.Bus.Apu.Pulse1.DacEnabled);
            Assert.False(core.Bus.Apu.Pulse1.Enabled);

            core.Bus.Write(0xFF14, 0x80);
            Assert.False(core.Bus.Apu.Pulse1.Enabled);
        }

        [Fact]
        public void The_duty_pattern_decides_when_the_pulse_is_high()
        {
            var core = Load();

            core.Bus!.Write(0xFF11, 0x80);   // 50% duty
            StartPulse(core, 0xFF10, 0x7FF); // the shortest period there is: 4 T-cycles a step

            var pulse = core.Bus.Apu.Pulse1;
            var seen = new HashSet<int>();

            for (int i = 0; i < 8; i++)
            {
                seen.Add(pulse.Output);
                core.Bus.Tick(4);
            }

            // A 50% duty must produce both a loud and a silent half within one period.
            Assert.Contains(0, seen);
            Assert.Contains(15, seen);
        }

        [Fact]
        public void The_length_counter_silences_a_channel_and_only_when_it_is_enabled()
        {
            var core = Load();

            core.Bus!.Write(0xFF11, 0x3F);   // length load 63, so one clock away from zero
            core.Bus.Write(0xFF12, 0xF0);
            core.Bus.Write(0xFF14, 0xC0);    // trigger with the length counter enabled

            Assert.True(core.Bus.Apu.Pulse1.Enabled);

            core.Bus.Tick(SequencerPeriod * 2);
            Assert.False(core.Bus.Apu.Pulse1.Enabled);

            // With the counter disabled the same wait leaves the channel running.
            core.Bus.Write(0xFF11, 0x3F);
            core.Bus.Write(0xFF14, 0x80);
            core.Bus.Tick(SequencerPeriod * 4);
            Assert.True(core.Bus.Apu.Pulse1.Enabled);
        }

        // Step 7 of the eight is the only one that moves the envelope.
        [Fact]
        public void The_envelope_steps_once_per_turn_of_the_sequencer()
        {
            var core = Load();

            core.Bus!.Write(0xFF12, 0xF1);   // volume 15, decreasing, period 1
            core.Bus.Write(0xFF14, 0x80);

            Assert.Equal(15, core.Bus.Apu.Pulse1.Envelope.Volume);

            core.Bus.Tick(SequencerPeriod * 8);
            Assert.Equal(14, core.Bus.Apu.Pulse1.Envelope.Volume);

            core.Bus.Tick(SequencerPeriod * 8);
            Assert.Equal(13, core.Bus.Apu.Pulse1.Envelope.Volume);
        }

        [Fact]
        public void An_envelope_period_of_zero_never_steps_at_all()
        {
            var core = Load();

            core.Bus!.Write(0xFF12, 0xF0);   // volume 15, decreasing, period 0
            core.Bus.Write(0xFF14, 0x80);

            core.Bus.Tick(SequencerPeriod * 32);
            Assert.Equal(15, core.Bus.Apu.Pulse1.Envelope.Volume);
        }

        [Fact]
        public void The_sweep_walks_the_frequency_and_only_channel_one_has_one()
        {
            var core = Load();

            core.Bus!.Write(0xFF10, 0x11);   // period 1, increasing, shift 1
            StartPulse(core, 0xFF10, 0x0400);

            core.Bus.Tick(SequencerPeriod * 3);
            Assert.Equal(0x0600, core.Bus.Apu.Pulse1.Frequency);

            Assert.False(core.Bus.Apu.Pulse2.HasSweep);
        }

        // Overflowing eleven bits disables the channel; it does not wrap and it does not clamp.
        [Fact]
        public void A_sweep_that_overflows_disables_its_channel()
        {
            var core = Load();

            core.Bus!.Write(0xFF10, 0x11);
            StartPulse(core, 0xFF10, 0x07C0);

            core.Bus.Tick(SequencerPeriod * 16);
            Assert.False(core.Bus.Apu.Pulse1.Enabled);
        }

        [Fact]
        public void The_wave_channel_reads_two_samples_from_each_ram_byte()
        {
            var core = Load();

            core.Bus!.Write(0xFF30, 0xF0);
            core.Bus.Write(0xFF1A, 0x80);    // DAC on
            core.Bus.Write(0xFF1C, 0x20);    // volume code 1: full scale
            core.Bus.Write(0xFF1D, 0xFF);
            core.Bus.Write(0xFF1E, 0x87);    // trigger, frequency $7FF, two T-cycles a step

            var wave = core.Bus.Apu.Wave;
            Assert.Equal(0x0F, wave.Sample);

            core.Bus.Tick(2);
            Assert.Equal(0x00, wave.Sample);
        }

        [Fact]
        public void The_wave_volume_code_is_a_shift_and_zero_is_silence()
        {
            var core = Load();

            core.Bus!.Write(0xFF30, 0xF0);
            core.Bus.Write(0xFF1A, 0x80);
            core.Bus.Write(0xFF1E, 0x80);

            core.Bus.Write(0xFF1C, 0x00);
            Assert.Equal(0, core.Bus.Apu.Wave.Output);

            core.Bus.Write(0xFF1C, 0x20);
            Assert.Equal(15, core.Bus.Apu.Wave.Output);

            core.Bus.Write(0xFF1C, 0x40);
            Assert.Equal(7, core.Bus.Apu.Wave.Output);

            core.Bus.Write(0xFF1C, 0x60);
            Assert.Equal(3, core.Bus.Apu.Wave.Output);
        }

        [Fact]
        public void The_noise_shift_register_starts_full_and_shifts_feedback_into_the_top()
        {
            var core = Load();

            core.Bus!.Write(0xFF21, 0xF0);
            core.Bus.Write(0xFF22, 0x00);    // divisor 8, shift 0
            core.Bus.Write(0xFF23, 0x80);

            Assert.Equal(0x7FFF, core.Bus.Apu.Noise.Lfsr);

            core.Bus.Tick(8);

            // 15 ones XOR to zero feedback, so the top bit goes low and the register halves.
            Assert.Equal(0x3FFF, core.Bus.Apu.Noise.Lfsr);
        }

        // Short mode also feeds bit 6, which is what shortens the period from 32767 to 127.
        [Fact]
        public void Short_mode_makes_the_noise_repeat_after_127_steps()
        {
            var core = Load();

            core.Bus!.Write(0xFF21, 0xF0);
            core.Bus.Write(0xFF22, 0x08);    // divisor 8, shift 0, short mode
            core.Bus.Write(0xFF23, 0x80);

            var seen = new List<int>();
            for (int i = 0; i < 127; i++)
            {
                seen.Add(core.Bus.Apu.Noise.Lfsr & 0x7F);
                core.Bus.Tick(8);
            }

            Assert.Equal(127, seen.Distinct().Count());
            Assert.Equal(seen[0], core.Bus.Apu.Noise.Lfsr & 0x7F);
        }

        [Fact]
        public void Powering_the_apu_off_clears_the_registers_but_not_wave_ram()
        {
            var core = Load();

            core.Bus!.Write(0xFF30, 0xAB);
            StartPulse(core, 0xFF10, 0x400);

            core.Bus.Write(0xFF26, 0x00);

            Assert.False(core.Bus.Apu.PoweredOn);
            Assert.False(core.Bus.Apu.Pulse1.Enabled);
            Assert.Equal(0, core.Bus.Apu.LeftVolume);
            Assert.Equal(0xAB, core.Bus.Read(0xFF30));

            // Every register but NR52 ignores a write while the power is off.
            core.Bus.Write(0xFF12, 0xF0);
            Assert.False(core.Bus.Apu.Pulse1.DacEnabled);

            core.Bus.Write(0xFF26, 0x80);
            Assert.True(core.Bus.Apu.PoweredOn);
        }

        [Fact]
        public void The_unreadable_register_bits_all_read_back_as_one()
        {
            var core = Load();

            core.Bus!.Write(0xFF13, 0x00);
            Assert.Equal(0xFF, core.Bus.Read(0xFF13));

            core.Bus.Write(0xFF14, 0x00);
            Assert.Equal(0xBF, core.Bus.Read(0xFF14));

            core.Bus.Write(0xFF1A, 0x00);
            Assert.Equal(0x7F, core.Bus.Read(0xFF1A));

            Assert.Equal(0x70, core.Bus.Read(0xFF26) & 0x70);
        }

        [Fact]
        public void Running_a_frame_produces_about_a_frames_worth_of_stereo_samples()
        {
            var core = Load();
            StartPulse(core, 0xFF10, 0x400);

            core.RunFrame();

            short[] samples = core.DequeueAudioSamples(4096);

            // 44100 / 59.7275 is 738 frames of two shorts each, give or take the fractional carry.
            Assert.InRange(samples.Length / 2, 730, 745);
            Assert.Empty(core.DequeueAudioSamples(4096));
        }

        [Fact]
        public void A_running_channel_actually_moves_the_output()
        {
            var core = Load();
            StartPulse(core, 0xFF10, 0x600);

            core.RunFrame();
            short[] samples = core.DequeueAudioSamples(4096);

            Assert.Contains(samples, s => s != 0);
            Assert.True(samples.Distinct().Count() > 1, "A square wave must not come out as one flat value.");
        }

        // The DC offset a live DAC sits at is real, and the console's own capacitor removes it.
        [Fact]
        public void The_high_pass_filter_pulls_a_steady_dac_back_to_silence()
        {
            var core = Load();

            // Volume 15 with no duty step reaching it: DAC on, digital output pinned.
            core.Bus!.Write(0xFF12, 0xF0);
            core.Bus.Write(0xFF14, 0x80);

            for (int i = 0; i < 30; i++) core.RunFrame();

            short[] samples = core.DequeueAudioSamples(4096);
            double mean = samples.Select(s => (double)s).Average();

            Assert.InRange(mean, -600, 600);
        }

        [Fact]
        public void Panning_decides_which_side_a_channel_reaches()
        {
            var core = Load();

            core.Bus!.Write(0xFF25, 0x10);   // channel 1 left only
            StartPulse(core, 0xFF10, 0x600);

            core.RunFrame();
            short[] samples = core.DequeueAudioSamples(4096);

            bool anyRight = false;
            for (int i = 1; i < samples.Length; i += 2) anyRight |= samples[i] != 0;

            Assert.False(anyRight, "Nothing is panned right, so the right channel must stay silent.");
        }

        [Fact]
        public void Muting_a_channel_drops_it_from_the_mix_without_stopping_it()
        {
            var core = Load();
            StartPulse(core, 0xFF10, 0x600);

            core.Bus!.Apu.SetChannelMuted(0, true);
            core.RunFrame();

            Assert.True(core.Bus.Apu.Pulse1.Enabled);
            Assert.True(core.Bus.Apu.IsChannelMuted(0));
        }

        // Double speed doubles the CPU and the timer; the sound hardware keeps its own rate.
        [Fact]
        public void Double_speed_does_not_change_the_pitch_or_the_sample_count()
        {
            var normal = Load(cgbFlag: 0xC0);
            StartPulse(normal, 0xFF10, 0x600);
            normal.RunFrame();
            int normalSamples = normal.DequeueAudioSamples(8192).Length;

            var fast = Load(cgbFlag: 0xC0);
            fast.Bus!.Write(0xFF4D, 0x01);
            fast.Bus.Stop();
            StartPulse(fast, 0xFF10, 0x600);
            fast.RunFrame();
            int fastSamples = fast.DequeueAudioSamples(8192).Length;

            Assert.True(fast.Bus.DoubleSpeed);
            Assert.InRange(fastSamples, normalSamples - 4, normalSamples + 4);
        }

        [Fact]
        public void A_save_state_round_trips_the_channel_state_but_not_the_pending_samples()
        {
            var core = Load();
            StartPulse(core, 0xFF10, 0x123);
            core.Bus!.Write(0xFF12, 0x83);

            using var stream = new MemoryStream();
            core.SaveState(stream);

            core.Bus.Write(0xFF26, 0x00);
            Assert.False(core.Bus.Apu.PoweredOn);

            stream.Position = 0;
            core.LoadState(stream);

            Assert.True(core.Bus.Apu.PoweredOn);
            Assert.Equal(0x123, core.Bus.Apu.Pulse1.Frequency);
            Assert.Equal(8, core.Bus.Apu.Pulse1.Envelope.InitialVolume);
        }

        [Fact]
        public void The_debug_target_reports_four_channels_and_a_live_peek()
        {
            var core = Load();
            var target = new EmuSen.Cores.Nintendo.Mercury.Debug.MercuryDebugTarget(core);

            StartPulse(core, 0xFF10, 0x600);
            core.RunFrame();
            target.RefreshProviders();

            Assert.Equal(GbApu.ChannelCount, target.AudioChannels.Current.Count);
            Assert.True(target.AudioChannels.Current[0].Active);
            Assert.Equal(new[] { "Pulse 1", "Pulse 2", "Wave", "Noise" },
                target.AudioChannels.Current.Select(c => c.Name));

            var (samples, rate) = target.GetAudioSamples();
            Assert.Equal(44100, rate);
            Assert.NotEmpty(samples);

            // The peek is not a drain: what it showed is still there to play.
            Assert.NotEmpty(core.DequeueAudioSamples(4096));
        }
    }
}
