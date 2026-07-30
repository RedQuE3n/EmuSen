using System;
using System.IO;

namespace EmuSen.Common
{
    // Delta codec behind RewindBuffer's chain - see EmuSen_Rewind_And_FastForward.md §1.3.
    public static class XorDeltaCodec
    {
        // [varint identicalRun][varint diffLen][diffLen XOR bytes], repeated - see §1.3.
        public static byte[] Encode(byte[] baseline, byte[] target)
        {
            if (baseline.Length != target.Length)
            {
                throw new ArgumentException($"XorDeltaCodec: length mismatch ({baseline.Length} vs {target.Length}).");
            }

            var output = new MemoryStream(Math.Max(64, target.Length / 8));
            int i = 0;
            int len = target.Length;

            while (i < len)
            {
                int runStart = i;
                // 8 at a time first - the identical run is the hot path.
                while (i + 8 <= len && BitConverter.ToUInt64(baseline, i) == BitConverter.ToUInt64(target, i)) i += 8;
                while (i < len && baseline[i] == target[i]) i++;
                int identicalRun = i - runStart;

                int diffStart = i;
                while (i < len && baseline[i] != target[i]) i++;
                int diffLen = i - diffStart;

                if (diffLen == 0) break; // identical tail - nothing left to record

                WriteVarInt(output, identicalRun);
                WriteVarInt(output, diffLen);
                for (int k = 0; k < diffLen; k++)
                {
                    output.WriteByte((byte)(baseline[diffStart + k] ^ target[diffStart + k]));
                }
            }

            return output.ToArray();
        }

        // In place, and its own inverse - see §1.3.
        public static void Apply(byte[] state, byte[] delta)
        {
            int i = 0;
            int pos = 0;
            while (pos < delta.Length)
            {
                i += ReadVarInt(delta, ref pos);
                int diffLen = ReadVarInt(delta, ref pos);
                if (i < 0 || diffLen < 0 || i + diffLen > state.Length || pos + diffLen > delta.Length)
                {
                    throw new InvalidDataException("XorDeltaCodec: delta does not fit the state it was applied to.");
                }
                for (int k = 0; k < diffLen; k++) state[i + k] ^= delta[pos + k];
                i += diffLen;
                pos += diffLen;
            }
        }

        private static void WriteVarInt(Stream s, int value)
        {
            uint v = (uint)value;
            while (v >= 0x80)
            {
                s.WriteByte((byte)(v | 0x80));
                v >>= 7;
            }
            s.WriteByte((byte)v);
        }

        private static int ReadVarInt(byte[] buffer, ref int pos)
        {
            int result = 0;
            int shift = 0;
            while (true)
            {
                if (pos >= buffer.Length) throw new InvalidDataException("XorDeltaCodec: truncated varint.");
                byte b = buffer[pos++];
                result |= (b & 0x7F) << shift;
                if ((b & 0x80) == 0) return result;
                shift += 7;
                if (shift > 28) throw new InvalidDataException("XorDeltaCodec: varint too long.");
            }
        }
    }
}
