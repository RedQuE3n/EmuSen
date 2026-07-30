using System;
using System.IO;
using EmuSen.Common;

namespace EmuSen.WiseMan.Common
{
    // The codec RewindBuffer's chain is built on - see EmuSen_Rewind_And_FastForward.md §1.3.
    public class XorDeltaCodecTests
    {
        private static byte[] Bytes(params int[] values) => Array.ConvertAll(values, v => (byte)v);

        [Fact]
        public void Identical_buffers_encode_to_nothing()
        {
            byte[] a = Bytes(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
            Assert.Empty(XorDeltaCodec.Encode(a, (byte[])a.Clone()));
        }

        [Fact]
        public void Applying_a_delta_walks_forward_to_the_target()
        {
            byte[] a = Bytes(1, 2, 3, 4, 5);
            byte[] b = Bytes(1, 9, 3, 7, 5);
            byte[] state = (byte[])a.Clone();
            XorDeltaCodec.Apply(state, XorDeltaCodec.Encode(a, b));
            Assert.Equal(b, state);
        }

        // The property the whole newest-anchored chain depends on - §1.3.
        [Fact]
        public void Applying_a_delta_walks_backward_to_the_baseline()
        {
            byte[] a = Bytes(1, 2, 3, 4, 5);
            byte[] b = Bytes(1, 9, 3, 7, 5);
            byte[] state = (byte[])b.Clone();
            XorDeltaCodec.Apply(state, XorDeltaCodec.Encode(a, b));
            Assert.Equal(a, state);
        }

        [Fact]
        public void Round_trips_a_difference_in_the_very_first_byte()
        {
            byte[] a = Bytes(0, 0, 0, 0);
            byte[] b = Bytes(0xFF, 0, 0, 0);
            byte[] state = (byte[])a.Clone();
            XorDeltaCodec.Apply(state, XorDeltaCodec.Encode(a, b));
            Assert.Equal(b, state);
        }

        [Fact]
        public void Round_trips_a_difference_in_the_very_last_byte()
        {
            byte[] a = new byte[4096];
            byte[] b = new byte[4096];
            b[^1] = 0xAB;
            byte[] state = (byte[])a.Clone();
            XorDeltaCodec.Apply(state, XorDeltaCodec.Encode(a, b));
            Assert.Equal(b, state);
        }

        // Runs longer than 127 bytes force multi-byte varints - §1.3.
        [Fact]
        public void Round_trips_runs_long_enough_to_need_multibyte_varints()
        {
            var rng = new Random(1234);
            byte[] a = new byte[300000];
            rng.NextBytes(a);
            byte[] b = (byte[])a.Clone();
            for (int i = 0; i < b.Length; i += 5000) b[i] ^= 0x5A;

            byte[] state = (byte[])a.Clone();
            XorDeltaCodec.Apply(state, XorDeltaCodec.Encode(a, b));
            Assert.Equal(b, state);
        }

        // The 8-byte fast path must not mis-detect a run starting mid-word - §1.3.
        [Fact]
        public void Round_trips_differences_that_straddle_the_eight_byte_fast_path()
        {
            byte[] a = new byte[64];
            byte[] b = new byte[64];
            foreach (int i in new[] { 0, 7, 8, 9, 15, 16, 31, 62, 63 }) b[i] = 0x11;

            byte[] state = (byte[])a.Clone();
            XorDeltaCodec.Apply(state, XorDeltaCodec.Encode(a, b));
            Assert.Equal(b, state);
        }

        [Fact]
        public void A_mostly_identical_pair_encodes_far_smaller_than_the_state()
        {
            byte[] a = new byte[1_000_000];
            new Random(7).NextBytes(a);
            byte[] b = (byte[])a.Clone();
            for (int i = 0; i < 1000; i++) b[i * 97] ^= 0xFF;

            Assert.True(XorDeltaCodec.Encode(a, b).Length < a.Length / 100);
        }

        [Fact]
        public void Mismatched_lengths_throw_rather_than_silently_truncating()
        {
            Assert.Throws<ArgumentException>(() => XorDeltaCodec.Encode(new byte[4], new byte[5]));
        }

        [Fact]
        public void A_delta_that_overruns_its_state_throws()
        {
            byte[] big = new byte[64];
            big[10] = 0xFF;
            byte[] delta = XorDeltaCodec.Encode(new byte[64], big);
            Assert.Throws<InvalidDataException>(() => XorDeltaCodec.Apply(new byte[4], delta));
        }
    }
}
