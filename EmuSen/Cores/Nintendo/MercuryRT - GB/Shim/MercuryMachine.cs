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
