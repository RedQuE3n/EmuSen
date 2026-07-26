namespace EmuSen.Audio
{
    // Minimal uncompressed PCM WAV writer, mirroring BmpFile's style - no
    // external audio library needed just to inspect what the DSP's buffer
    // currently holds. Samples are already interleaved L/R 16-bit PCM (see
    // IDebugTarget.GetAudioSamples), so this is a fixed 44-byte header plus
    // the raw sample bytes, nothing more.
    public static class WavFile
    {
        public static void Write(string path, short[] samples, int sampleRate)
        {
            const int channels = 2;
            const int bitsPerSample = 16;
            int byteRate = sampleRate * channels * bitsPerSample / 8;
            int blockAlign = channels * bitsPerSample / 8;
            int dataSize = samples.Length * sizeof(short);

            using var fs = new FileStream(path, FileMode.Create);
            using var w = new BinaryWriter(fs);

            w.Write(new[] { 'R', 'I', 'F', 'F' });
            w.Write(36 + dataSize);
            w.Write(new[] { 'W', 'A', 'V', 'E' });

            w.Write(new[] { 'f', 'm', 't', ' ' });
            w.Write(16); // fmt chunk size
            w.Write((short)1); // PCM
            w.Write((short)channels);
            w.Write(sampleRate);
            w.Write(byteRate);
            w.Write((short)blockAlign);
            w.Write((short)bitsPerSample);

            w.Write(new[] { 'd', 'a', 't', 'a' });
            w.Write(dataSize);
            foreach (short sample in samples) w.Write(sample);
        }
    }
}
