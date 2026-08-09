namespace EmuSen.WiseMan.Fixtures
{
    // A core's mixed output as a WAV, to compare against the probe's - see Mercury_HardwareTests.md §6.
    public static class AudioCapture
    {
        // Set to a directory to have the commercial-ROM suite leave its captures there.
        public const string DumpDirectoryVariable = "EMUSEN_AUDIO_DUMP";

        public static string? DumpDirectory
        {
            get
            {
                string? folder = Environment.GetEnvironmentVariable(DumpDirectoryVariable);
                return string.IsNullOrWhiteSpace(folder) ? null : folder;
            }
        }

        public static void WriteWav(string path, IReadOnlyList<short> interleaved, int sampleRate, int channels = 2)
        {
            const int bits = 16;
            int blockAlign = channels * (bits / 8);
            int dataBytes = interleaved.Count * 2;

            using var stream = File.Create(path);
            using var w = new BinaryWriter(stream);

            w.Write("RIFF"u8.ToArray());
            w.Write(36 + dataBytes);
            w.Write("WAVE"u8.ToArray());
            w.Write("fmt "u8.ToArray());
            w.Write(16);
            w.Write((short)1);
            w.Write((short)channels);
            w.Write(sampleRate);
            w.Write(sampleRate * blockAlign);
            w.Write((short)blockAlign);
            w.Write((short)bits);
            w.Write("data"u8.ToArray());
            w.Write(dataBytes);

            foreach (short sample in interleaved) w.Write(sample);
        }
    }
}
