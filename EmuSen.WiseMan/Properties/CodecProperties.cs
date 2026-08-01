using System;
using System.Linq;
using CsCheck;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Venus.Cheats;

namespace EmuSen.WiseMan.Properties
{
    // Laws XorDeltaCodec and the cheat codecs state in prose - see EmuSen_Debugging_Tools_Reference_v5.md §3.18b.
    public class CodecProperties
    {
        private static bool RoundTrips(byte[] baseline, byte[] target)
        {
            byte[] delta = XorDeltaCodec.Encode(baseline, target);
            var restored = (byte[])baseline.Clone();
            XorDeltaCodec.Apply(restored, delta);
            return restored.SequenceEqual(target);
        }

        // Independently random pairs - the dense-diff case, where almost every byte differs.
        [Fact]
        public void Encode_then_Apply_reconstructs_the_target() =>
            Gen.Select(Gen.Byte, Gen.Byte).Array.Sample(pairs =>
                RoundTrips(pairs.Select(p => p.Item1).ToArray(), pairs.Select(p => p.Item2).ToArray()),
                iter: 2000);

        // A few bytes changed - the long-identical-run case the varint encoding exists for.
        [Fact]
        public void A_sparsely_patched_baseline_round_trips() =>
            Gen.Select(Gen.Byte.Array[1, 400], Gen.Select(Gen.UInt, Gen.Byte).Array).Sample((baseline, patches) =>
            {
                var target = (byte[])baseline.Clone();
                foreach (var (idx, value) in patches) target[(int)(idx % (uint)baseline.Length)] = value;
                return RoundTrips(baseline, target);
            }, iter: 2000);

        [Fact]
        public void Apply_is_its_own_inverse() =>
            Gen.Select(Gen.Byte, Gen.Byte).Array.Sample(pairs =>
            {
                var baseline = pairs.Select(p => p.Item1).ToArray();
                var target = pairs.Select(p => p.Item2).ToArray();
                byte[] delta = XorDeltaCodec.Encode(baseline, target);
                var state = (byte[])baseline.Clone();
                XorDeltaCodec.Apply(state, delta);
                XorDeltaCodec.Apply(state, delta);
                return state.SequenceEqual(baseline);
            }, iter: 2000);

        [Fact]
        public void An_unchanged_state_encodes_to_an_empty_delta() =>
            Gen.Byte.Array.Sample(state => XorDeltaCodec.Encode(state, state).Length == 0, iter: 500);

        [Fact]
        public void A_length_mismatch_is_rejected_rather_than_truncating() =>
            Gen.Select(Gen.Byte.Array, Gen.Byte.Array).Sample((a, b) =>
            {
                if (a.Length == b.Length) return true;
                try { XorDeltaCodec.Encode(a, b); return false; }
                catch (ArgumentException) { return true; }
            }, iter: 500);

        // The one that matters: CanDecode must never promise a decode that then throws.
        [Fact]
        public void CanDecode_never_promises_a_decode_that_throws() =>
            Gen.String.Sample(code =>
            {
                if (ActionReplayCodec.CanDecode(code)) ActionReplayCodec.Decode(code);
                if (GameGenieCodec.CanDecode(code)) GameGenieCodec.Decode(code);
                return true;
            }, iter: 5000);

        [Fact]
        public void A_well_formed_code_decodes_to_a_24_bit_address() =>
            Gen.Select(Gen.Char["0123456789abcdefABCDEF"].Array[8], Gen.Char["DF4709156BC8A23E"].Array[8])
               .Sample((ar, gg) =>
               {
                   (int arAddr, _) = ActionReplayCodec.Decode(new string(ar));
                   (int ggAddr, _) = GameGenieCodec.Decode(new string(gg));
                   return arAddr >= 0 && arAddr <= 0xFFFFFF && ggAddr >= 0 && ggAddr <= 0xFFFFFF;
               }, iter: 2000);

        [Fact]
        public void Separators_in_a_code_are_cosmetic() =>
            Gen.Select(Gen.Char["0123456789abcdefABCDEF"].Array[8], Gen.Int[0, 8], Gen.Char["-: /"])
               .Sample((code, at, sep) =>
               {
                   string plain = new(code);
                   return ActionReplayCodec.Decode(plain.Insert(at, sep.ToString())) == ActionReplayCodec.Decode(plain);
               }, iter: 2000);
    }
}
