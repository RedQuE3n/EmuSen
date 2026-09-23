using System;
using System.IO;
using System.Text;

namespace EmuSen.Cores.Nintendo.MercuryRT
{
    // MercuryRT's machine behind its handle, its state read and written in the C# Mercury's own format - see Mercury_Native.md §3.2.
    public sealed unsafe class MercuryMachine : IDisposable
    {
        private static readonly delegate* unmanaged<byte*, nuint, byte*, nint, int*, nint> New = (delegate* unmanaged<byte*, nuint, byte*, nint, int*, nint>)MercuryNative.Export("mercury_machine_new");
        private static readonly delegate* unmanaged<nint, void> Free = (delegate* unmanaged<nint, void>)MercuryNative.Export("mercury_machine_free");
        private static readonly delegate* unmanaged<nint, byte*, nuint, int> LoadState = (delegate* unmanaged<nint, byte*, nuint, int>)MercuryNative.Export("mercury_machine_load_state");
        private static readonly delegate* unmanaged<nint, long> SizeOf = (delegate* unmanaged<nint, long>)MercuryNative.Export("mercury_machine_save_state_size");
        private static readonly delegate* unmanaged<nint, byte*, nuint, long> SaveState = (delegate* unmanaged<nint, byte*, nuint, long>)MercuryNative.Export("mercury_machine_save_state");
        private static readonly delegate* unmanaged<nint, byte*, nuint, long> LayoutOf = (delegate* unmanaged<nint, byte*, nuint, long>)MercuryNative.Export("mercury_machine_state_layout");

        private static readonly delegate* unmanaged<nint, uint*, int> RunFrameOf = (delegate* unmanaged<nint, uint*, int>)MercuryNative.Export("mercury_machine_run_frame");
        private static readonly delegate* unmanaged<nint, int> StepOf = (delegate* unmanaged<nint, int>)MercuryNative.Export("mercury_machine_step");
        private static readonly delegate* unmanaged<nint, uint, void> SetButtonsOf = (delegate* unmanaged<nint, uint, void>)MercuryNative.Export("mercury_machine_set_buttons");
        private static readonly delegate* unmanaged<nint, uint, void> SetOptionsOf = (delegate* unmanaged<nint, uint, void>)MercuryNative.Export("mercury_machine_set_options");
        private static readonly delegate* unmanaged<nint, uint, void> SetMutesOf = (delegate* unmanaged<nint, uint, void>)MercuryNative.Export("mercury_machine_set_mutes");
        private static readonly delegate* unmanaged<nint, double, double, void> SetSampleRateOf = (delegate* unmanaged<nint, double, double, void>)MercuryNative.Export("mercury_machine_set_sample_rate");
        private static readonly delegate* unmanaged<nint, byte*, nuint, long> FrameOf = (delegate* unmanaged<nint, byte*, nuint, long>)MercuryNative.Export("mercury_machine_frame");
        private static readonly delegate* unmanaged<nint, long> AudioBufferedOf = (delegate* unmanaged<nint, long>)MercuryNative.Export("mercury_machine_audio_buffered");
        private static readonly delegate* unmanaged<nint, short*, nuint, long, long> DrainOf = (delegate* unmanaged<nint, short*, nuint, long, long>)MercuryNative.Export("mercury_machine_drain_audio");
        private static readonly delegate* unmanaged<nint, byte*, nuint, long> SerialOf = (delegate* unmanaged<nint, byte*, nuint, long>)MercuryNative.Export("mercury_machine_serial");

        public const int FrameBytes = 160 * 144 * 4;

        private nint _handle;

        public static bool Available => New != null;

        // The ROM image and C#'s save path, null as C# has it under --nobattery; the board is built from the header as C# builds it.
        public MercuryMachine(ReadOnlySpan<byte> rom, string? savePath)
        {
            if (!Available) throw new InvalidOperationException($"MercuryRT is not in use: {MercuryNative.Report}");
            byte[]? path = savePath is null ? null : Encoding.UTF8.GetBytes(savePath);
            int status;
            fixed (byte* image = rom)
            fixed (byte* text = path)
            {
                _handle = New(image, (nuint)rom.Length, text, path is null ? -1 : path.Length, &status);
            }
            if (_handle == 0) throw status == -10 ? new NotSupportedException($"Cartridge type ${rom[0x147]:X2} is not implemented - see Mercury_Memory.md §4.") : new InvalidDataException($"MercuryRT refused the image: {Describe(status)}.");
            SetSampleRate(44100);
        }

        // C#'s Apu.SetSampleRate, integer division and Math.Pow both evaluated here - see Mercury_Native.md §3.3.
        public void SetSampleRate(int sampleRate)
        {
            double cyclesPerSample = 4194304 / sampleRate;
            SetSampleRateOf(Handle, cyclesPerSample, Math.Pow(EmuSen.Cores.Nintendo.Mercury.Audio.Apu.HighPassSeed, cyclesPerSample));
        }

        // A frame, or the C# core's exception for an opcode no SM83 has.
        public void RunFrame()
        {
            uint detail = 0;
            int status = RunFrameOf(Handle, &detail);
            if (status == -20) throw IllegalOpcode(detail);
            if (status != 0) throw new InvalidOperationException($"MercuryRT could not run: {Describe(status)}.");
        }

        // One instruction with the frame loop's acknowledge and stall; the T-cycles, or -20 for an illegal opcode.
        public int Step() => StepOf(Handle);

        public static NotSupportedException IllegalOpcode(uint detail) =>
            new($"Opcode ${detail >> 16:X2} at ${detail & 0xFFFF:X4} is not a real SM83 instruction - see Mercury_Cpu.md §6.1.");

        // Bit order right, left, up, down, A, B, select, start.
        public void SetButtons(uint mask) => SetButtonsOf(Handle, mask);

        public void SetOptions(bool skipRendering) => SetOptionsOf(Handle, skipRendering ? 1u : 0u);

        public void SetMutes(uint mask) => SetMutesOf(Handle, mask);

        public void CopyFrame(byte[] into)
        {
            fixed (byte* data = into) FrameOf(Handle, data, (nuint)into.Length);
        }

        public int BufferedSamples => (int)AudioBufferedOf(Handle);

        public short[] DrainAudio(int maxFrames)
        {
            int wanted = (int)Math.Min((long)maxFrames * 2, BufferedSamples);
            wanted -= wanted & 1;
            if (wanted <= 0) return Array.Empty<short>();
            var samples = new short[wanted];
            fixed (short* data = samples) DrainOf(Handle, data, (nuint)samples.Length, maxFrames);
            return samples;
        }

        public byte[] SerialLog()
        {
            long n = SerialOf(Handle, null, 0);
            var log = new byte[n];
            fixed (byte* data = log) SerialOf(Handle, data, (nuint)log.Length);
            return log;
        }

        // A failed load leaves the machine as it was.
        public void Load(ReadOnlySpan<byte> state)
        {
            int status;
            fixed (byte* data = state) status = LoadState(Handle, data, (nuint)state.Length);
            if (status != 0) throw new InvalidDataException($"MercuryRT refused the state: {Describe(status)}.");
        }

        public byte[] Save()
        {
            long size = Check(SizeOf(Handle));
            var state = new byte[size];
            fixed (byte* data = state) Check(SaveState(Handle, data, (nuint)state.Length));
            return state;
        }

        // One line per field, "offset length type path", in the order the state writes them.
        public string Layout()
        {
            long size = Check(LayoutOf(Handle, null, 0));
            var text = new byte[size];
            fixed (byte* data = text) Check(LayoutOf(Handle, data, (nuint)text.Length));
            return Encoding.UTF8.GetString(text);
        }

        public static string Describe(long status) => status switch
        {
            -1 => "no machine",
            -2 => "the state is truncated",
            -3 => "not a Mercury save state",
            -4 => "an unknown state version",
            -5 => "a string length the C# reader refuses",
            -7 => "the buffer is too small",
            -9 => "an image shorter than the 336-byte header",
            -10 => "a cartridge type no board implements",
            _ => $"status {status}",
        };

        private static long Check(long result) => result >= 0 ? result : throw new InvalidDataException($"MercuryRT could not write the state: {Describe(result)}.");

        private nint Handle => _handle != 0 ? _handle : throw new ObjectDisposedException(nameof(MercuryMachine));

        public void Dispose()
        {
            if (_handle != 0) Free(_handle);
            _handle = 0;
            GC.SuppressFinalize(this);
        }

        ~MercuryMachine()
        {
            if (_handle != 0) Free(_handle);
        }
    }
}
