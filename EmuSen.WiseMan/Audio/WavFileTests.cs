using EmuSen.Audio;

namespace EmuSen.WiseMan.Audio
{
    // WavFile.Write - a minimal 44-byte-header PCM writer for `audiodump`.
    // What's worth locking down is the RIFF/WAVE/fmt/data header fields
    // for a known sample-rate/channel input, since a wrong byteRate or
    // blockAlign produces a .wav that looks fine by size alone but plays
    // back at the wrong speed or as noise in a real audio tool.
    public class WavFileTests
    {
        [Fact]
        public void Header_fields_match_a_known_sample_rate_and_sample_count()
        {
            short[] samples = { 1, -1, 2, -2, 3, -3 }; // 3 interleaved L/R frames
            const int sampleRate = 32000;
            string path = Path.Combine(Path.GetTempPath(), $"wiseman_wav_{Guid.NewGuid():N}.wav");

            try
            {
                WavFile.Write(path, samples, sampleRate);

                using var fs = new FileStream(path, FileMode.Open, FileAccess.Read);
                using var r = new BinaryReader(fs);

                Assert.Equal("RIFF", new string(r.ReadChars(4)));
                int dataSize = samples.Length * sizeof(short);
                Assert.Equal(36 + dataSize, r.ReadInt32());
                Assert.Equal("WAVE", new string(r.ReadChars(4)));

                Assert.Equal("fmt ", new string(r.ReadChars(4)));
                Assert.Equal(16, r.ReadInt32());
                Assert.Equal(1, r.ReadInt16()); // PCM
                Assert.Equal(2, r.ReadInt16()); // channels
                Assert.Equal(sampleRate, r.ReadInt32());
                Assert.Equal(sampleRate * 2 * 16 / 8, r.ReadInt32()); // byteRate
                Assert.Equal(2 * 16 / 8, r.ReadInt16()); // blockAlign
                Assert.Equal(16, r.ReadInt16()); // bitsPerSample

                Assert.Equal("data", new string(r.ReadChars(4)));
                Assert.Equal(dataSize, r.ReadInt32());

                foreach (short expected in samples) Assert.Equal(expected, r.ReadInt16());
            }
            finally
            {
                File.Delete(path);
            }
        }

        [Fact]
        public void Empty_sample_array_writes_a_zero_length_data_chunk()
        {
            string path = Path.Combine(Path.GetTempPath(), $"wiseman_wav_{Guid.NewGuid():N}.wav");
            try
            {
                WavFile.Write(path, Array.Empty<short>(), 32000);
                Assert.Equal(44, new FileInfo(path).Length); // header only, no data
            }
            finally
            {
                File.Delete(path);
            }
        }
    }
}
