using EmuSen.Cores.Nintendo.Venus.Apu;

namespace EmuSen.WiseMan.Audio
{
    // SDsp's KON handling - see Venus_APU.md §3.3. KON is an event register:
    // each write with a bit set keys that voice once, and a driver is under
    // no obligation to clear it between notes. These pin the two ways the
    // previous level-comparison (`kon & ~_prevKon`) silently dropped notes,
    // both found diagnosing missing title-screen music in BSSMSAS, whose
    // driver never writes 0 to KON at all - it lost 24 of 43 note events in
    // the first 15 seconds.
    public class DspKeyOnLatchTests
    {
        private const int RegKeyOn = 0x4C;
        private const int RegKeyOff = 0x5C;

        // Reset() leaves FLG at the hardware power-on 0xE0 (mute + echo
        // disabled), which is what we want here: key events are processed
        // before the mute gate, so voices still trigger while nothing is
        // written to the echo buffer in RAM.
        private static SDsp NewDsp()
        {
            var dsp = new SDsp();
            dsp.AttachMemory(new byte[0x10000]);
            dsp.Reset();
            return dsp;
        }

        private static void Write(SDsp dsp, int register, byte value)
        {
            dsp.SetRegisterAddress((byte)register);
            dsp.WriteRegister(value);
        }

        // 32 SPC700 cycles per output sample, so this is exactly one
        // GenerateSample()/ProcessKeyEvents() pass.
        private static void OneSample(SDsp dsp) => dsp.Tick(32);

        private static int KeyOns(SDsp dsp, int voice) => dsp.GetVoiceDebugInfo(voice).KeyOnCount;

        [Fact]
        public void Re_writing_a_bit_that_is_already_set_still_keys_the_voice_again()
        {
            var dsp = NewDsp();

            Write(dsp, RegKeyOn, 0x01);
            OneSample(dsp);
            Write(dsp, RegKeyOn, 0x01); // no intervening clear - the BSSMSAS driver's pattern
            OneSample(dsp);

            Assert.Equal(2, KeyOns(dsp, 0));
        }

        // The other half of the same bug: a write is latched, so a KON pulse
        // that both sets and clears between two samples still counts.
        [Fact]
        public void A_key_on_pulse_written_and_cleared_within_one_sample_is_not_lost()
        {
            var dsp = NewDsp();

            Write(dsp, RegKeyOn, 0x01);
            Write(dsp, RegKeyOn, 0x00);
            OneSample(dsp);

            Assert.Equal(1, KeyOns(dsp, 0));
        }

        [Fact]
        public void A_bit_left_set_does_not_retrigger_on_every_following_sample()
        {
            var dsp = NewDsp();

            Write(dsp, RegKeyOn, 0x01);
            for (int i = 0; i < 50; i++) OneSample(dsp);

            Assert.Equal(1, KeyOns(dsp, 0));
        }

        // 0x33 is the exact mask BSSMSAS writes for its four-voice chords -
        // the case that used to arrive with one note missing when a previous
        // write had left one of those bits set.
        [Fact]
        public void Every_voice_named_in_one_mask_keys_on()
        {
            var dsp = NewDsp();

            Write(dsp, RegKeyOn, 0x01);
            OneSample(dsp);
            Write(dsp, RegKeyOn, 0x33);
            OneSample(dsp);

            Assert.Equal(2, KeyOns(dsp, 0)); // bit already set, still retriggered
            Assert.Equal(1, KeyOns(dsp, 1));
            Assert.Equal(1, KeyOns(dsp, 4));
            Assert.Equal(1, KeyOns(dsp, 5));
            Assert.Equal(0, KeyOns(dsp, 2)); // not in the mask
        }

        [Fact]
        public void Two_masks_in_the_same_sample_window_both_take_effect()
        {
            var dsp = NewDsp();

            Write(dsp, RegKeyOn, 0x01);
            Write(dsp, RegKeyOn, 0x02);
            OneSample(dsp);

            Assert.Equal(1, KeyOns(dsp, 0));
            Assert.Equal(1, KeyOns(dsp, 1));
        }

        // Key-off still wins over a key-on landing in the same window, and a
        // released voice can be keyed on again afterwards - the driver's
        // actual note-change sequence is KOFF=mask, KOFF=0, KON=mask.
        [Fact]
        public void Key_off_wins_over_a_key_on_in_the_same_window()
        {
            var dsp = NewDsp();

            Write(dsp, RegKeyOn, 0x01);
            Write(dsp, RegKeyOff, 0x01);
            OneSample(dsp);

            Assert.Equal(1, KeyOns(dsp, 0));
            Assert.Equal("Release", dsp.GetVoiceDebugInfo(0).Stage);
        }

        [Fact]
        public void The_drivers_key_off_then_key_on_sequence_retriggers_the_voice()
        {
            var dsp = NewDsp();

            Write(dsp, RegKeyOn, 0x02);
            OneSample(dsp);

            Write(dsp, RegKeyOff, 0x02);
            OneSample(dsp);
            Write(dsp, RegKeyOff, 0x00);
            Write(dsp, RegKeyOn, 0x02);
            OneSample(dsp);

            Assert.Equal(2, KeyOns(dsp, 1));
            Assert.Equal("Attack", dsp.GetVoiceDebugInfo(1).Stage);
        }
    }
}
