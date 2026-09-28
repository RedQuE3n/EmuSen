using System;
using System.IO;
using System.Text;

namespace EmuSen.Cores.Nintendo.MoonRT
{
    // MoonRT's machine behind its handle, its state read and written in the C# Moon's own format - see Moon_Native.md §3.2.
    public sealed unsafe class MoonMachine : IDisposable
    {
        private static readonly delegate* unmanaged<byte*, nuint, int*, nint> New = (delegate* unmanaged<byte*, nuint, int*, nint>)MoonNative.Export("moon_machine_new");
        private static readonly delegate* unmanaged<nint, void> Free = (delegate* unmanaged<nint, void>)MoonNative.Export("moon_machine_free");
        private static readonly delegate* unmanaged<nint, byte*, nuint, int> LoadState = (delegate* unmanaged<nint, byte*, nuint, int>)MoonNative.Export("moon_machine_load_state");
        private static readonly delegate* unmanaged<nint, long> SizeOf = (delegate* unmanaged<nint, long>)MoonNative.Export("moon_machine_save_state_size");
        private static readonly delegate* unmanaged<nint, byte*, nuint, long> SaveState = (delegate* unmanaged<nint, byte*, nuint, long>)MoonNative.Export("moon_machine_save_state");
        private static readonly delegate* unmanaged<nint, byte*, nuint, long> LayoutOf = (delegate* unmanaged<nint, byte*, nuint, long>)MoonNative.Export("moon_machine_state_layout");

        private static readonly delegate* unmanaged<nint, int> RunFrameOf = (delegate* unmanaged<nint, int>)MoonNative.Export("moon_machine_run_frame");
        private static readonly delegate* unmanaged<nint, int> ResetOf = (delegate* unmanaged<nint, int>)MoonNative.Export("moon_machine_reset");
        private static readonly delegate* unmanaged<nint, int> StepOf = (delegate* unmanaged<nint, int>)MoonNative.Export("moon_machine_step");
        private static readonly delegate* unmanaged<nint, int, uint, void> SetButtonsOf = (delegate* unmanaged<nint, int, uint, void>)MoonNative.Export("moon_machine_set_buttons");
        private static readonly delegate* unmanaged<nint, uint, void> SetOptionsOf = (delegate* unmanaged<nint, uint, void>)MoonNative.Export("moon_machine_set_options");
        private static readonly delegate* unmanaged<nint, uint, void> SetMutesOf = (delegate* unmanaged<nint, uint, void>)MoonNative.Export("moon_machine_set_mutes");
        private static readonly delegate* unmanaged<nint, byte*, nuint, long> FrameOf = (delegate* unmanaged<nint, byte*, nuint, long>)MoonNative.Export("moon_machine_frame");
        private static readonly delegate* unmanaged<nint, long> AudioBufferedOf = (delegate* unmanaged<nint, long>)MoonNative.Export("moon_machine_audio_buffered");
        private static readonly delegate* unmanaged<nint, short*, nuint, long, long> DrainOf = (delegate* unmanaged<nint, short*, nuint, long, long>)MoonNative.Export("moon_machine_drain_audio");

        private static readonly delegate* unmanaged<nint, long> TotalFramesOf = (delegate* unmanaged<nint, long>)MoonNative.Export("moon_machine_total_frames");
        private static readonly delegate* unmanaged<nint, uint, long> SpaceSizeOf = (delegate* unmanaged<nint, uint, long>)MoonNative.Export("moon_machine_space_size");
        private static readonly delegate* unmanaged<nint, uint, int, byte*, nuint, long> ReadSpaceOf = (delegate* unmanaged<nint, uint, int, byte*, nuint, long>)MoonNative.Export("moon_machine_read_space");
        private static readonly delegate* unmanaged<nint, uint, int, byte*, nuint, long> WriteSpaceOf = (delegate* unmanaged<nint, uint, int, byte*, nuint, long>)MoonNative.Export("moon_machine_write_space");
        private static readonly delegate* unmanaged<nint, ushort*, ushort*, nuint, void> SetRomPatchesOf = (delegate* unmanaged<nint, ushort*, ushort*, nuint, void>)MoonNative.Export("moon_machine_set_rom_patches");

        public const int FrameBytes = 256 * 240 * 4;

        // MoonCore.Spaces' names, numbered as the C ABI numbers them.
        public static readonly string[] SpaceNames = { "RAM", "PRGROM", "PRGRAM", "CHR", "CIRAM", "OAM", "PALETTE", "CPUBUS" };

        public static bool Available => New != null;

        public static bool Complete => Available && SetRomPatchesOf != null && ResetOf != null;

        private nint _handle;

        // The iNES image; the board is built from the header as C# builds it.
        public MoonMachine(ReadOnlySpan<byte> image)
        {
            if (!Available) throw new InvalidOperationException($"MoonRT is not in use: {MoonNative.Report}");
            int status;
            fixed (byte* data = image) _handle = New(data, (nuint)image.Length, &status);
            if (_handle == 0) throw Refusal(status);
        }

        // The C# exception MoonCore.LoadRom or RunFrame would have thrown for this status - see Moon_Native.md §6.2, D4.
        public static Exception Refusal(int status) => status switch
        {
            -9 => new InvalidDataException("Not an iNES image: missing the \"NES\\x1A\" magic."),
            -10 => new NotSupportedException("The iNES mapper is not implemented - see Moon_Memory.md §4 for what is."),
            -11 => new InvalidDataException("The header claims more PRG than the file holds."),
            -31 => new IndexOutOfRangeException("Index was outside the bounds of the array."),
            -32 => new DivideByZeroException("Attempted to divide by zero."),
            -33 => new ArgumentOutOfRangeException("masterDelta", "A clock cannot run backwards."),
            _ => new InvalidOperationException($"MoonRT refused: {Describe(status)}."),
        };

        public void RunFrame()
        {
            int status = RunFrameOf(Handle);
            if (status != 0) throw Refusal(status);
        }

        public void Reset()
        {
            int status = ResetOf(Handle);
            if (status != 0) throw Refusal(status);
        }

        // One instruction with the frame loop's DMA charge, NMI edge and stolen cycles; the cycles, or a negative status.
        public int Step() => StepOf(Handle);

        // Bit n for NesButton n: A, B, Select, Start, Up, Down, Left, Right.
        public void SetButtons(int port, uint mask) => SetButtonsOf(Handle, port, mask);

        public void SetOptions(bool skipRendering) => SetOptionsOf(Handle, skipRendering ? 1u : 0u);

        public void SetMutes(uint mask) => SetMutesOf(Handle, mask);

        public long TotalFrames => TotalFramesOf(Handle);

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

        public int SpaceSize(int space) => (int)SpaceSizeOf(Handle, (uint)space);

        // Read as MoonCore.ReadSpace reads, a byte at a time from address on; CPUBUS reads have their side effects.
        public void ReadSpace(int space, int address, Span<byte> into)
        {
            fixed (byte* data = into) ReadSpaceOf(Handle, (uint)space, address, data, (nuint)into.Length);
        }

        public void WriteSpace(int space, int address, ReadOnlySpan<byte> from)
        {
            fixed (byte* data = from) WriteSpaceOf(Handle, (uint)space, address, data, (nuint)from.Length);
        }

        // CPU addresses, then 256 entries each: 0x100 | patched for an original byte a patch replaces, 0 where none does.
        public void SetRomPatches(ushort[] addresses, ushort[] tables)
        {
            fixed (ushort* a = addresses)
            fixed (ushort* t = tables)
            {
                SetRomPatchesOf(Handle, a, t, (nuint)addresses.Length);
            }
        }

        // A failed load leaves the machine as it was.
        public void Load(ReadOnlySpan<byte> state)
        {
            int status;
            fixed (byte* data = state) status = LoadState(Handle, data, (nuint)state.Length);
            if (status != 0) throw new InvalidDataException($"MoonRT refused the state: {Describe(status)}.");
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
            -3 => "not a Moon save state",
            -4 => "an unknown state version",
            -7 => "the buffer is too small",
            -9 => "not an iNES image",
            -10 => "a mapper no board implements",
            -11 => "a header claiming more PRG than the file holds",
            -31 => "an index outside an array",
            -32 => "a division by zero",
            -33 => "a clock running backwards",
            _ => $"status {status}",
        };

        private static long Check(long result) => result >= 0 ? result : throw new InvalidDataException($"MoonRT could not write the state: {Describe(result)}.");

        private nint Handle => _handle != 0 ? _handle : throw new ObjectDisposedException(nameof(MoonMachine));

        public void Dispose()
        {
            if (_handle != 0) Free(_handle);
            _handle = 0;
            GC.SuppressFinalize(this);
        }

        ~MoonMachine()
        {
            if (_handle != 0) Free(_handle);
        }
    }
}
