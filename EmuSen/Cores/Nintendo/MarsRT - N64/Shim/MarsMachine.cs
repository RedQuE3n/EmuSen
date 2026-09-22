using System;
using System.IO;
using System.Text;
using EmuSen.Cores.Nintendo.Mars.Native;

namespace EmuSen.Cores.Nintendo.MarsRT
{
    // MarsRT's machine behind its handle: for now its state, read and written in the C# Mars's own format - see Mars_Native.md §5.1.
    public sealed unsafe class MarsMachine : IDisposable
    {
        private static readonly delegate* unmanaged<uint, nint> New = (delegate* unmanaged<uint, nint>)MarsNative.Export("mars_machine_new");
        private static readonly delegate* unmanaged<nint, void> Free = (delegate* unmanaged<nint, void>)MarsNative.Export("mars_machine_free");
        private static readonly delegate* unmanaged<nint, uint> RdramOf = (delegate* unmanaged<nint, uint>)MarsNative.Export("mars_machine_rdram_bytes");
        private static readonly delegate* unmanaged<nint, byte*, nuint, int> LoadState = (delegate* unmanaged<nint, byte*, nuint, int>)MarsNative.Export("mars_machine_load_state");
        private static readonly delegate* unmanaged<nint, int> KindOf = (delegate* unmanaged<nint, int>)MarsNative.Export("mars_machine_state_kind");
        private static readonly delegate* unmanaged<nint, uint, long> SizeOf = (delegate* unmanaged<nint, uint, long>)MarsNative.Export("mars_machine_save_state_size");
        private static readonly delegate* unmanaged<nint, byte*, nuint, uint, long> SaveState = (delegate* unmanaged<nint, byte*, nuint, uint, long>)MarsNative.Export("mars_machine_save_state");
        private static readonly delegate* unmanaged<nint, uint, byte*, nuint, long> LayoutOf = (delegate* unmanaged<nint, uint, byte*, nuint, long>)MarsNative.Export("mars_machine_state_layout");

        // The version a loaded state had; the numbers are MarsCore's own.
        public const int StateKind = 1, SnapshotKind = 2;

        private nint _handle;

        public static bool Available => New != null;

        public MarsMachine(int rdramBytes)
        {
            if (!Available) throw new InvalidOperationException($"MarsRT is not in use: {MarsNative.Report}");
            _handle = New((uint)rdramBytes);
            if (_handle == 0) throw new ArgumentOutOfRangeException(nameof(rdramBytes), rdramBytes, "RDRAM is 4 MB or 8 MB.");
        }

        public int RdramBytes => (int)RdramOf(Handle);

        // 1 after a state, 2 after a snapshot, 0 before any load.
        public int LoadedKind => KindOf(Handle);

        // A failed load leaves the machine as it was.
        public void Load(ReadOnlySpan<byte> state)
        {
            int status;
            fixed (byte* data = state) status = LoadState(Handle, data, (nuint)state.Length);
            if (status != 0) throw new InvalidDataException($"MarsRT refused the state: {Describe(status)}.");
        }

        public byte[] Save(bool snapshot)
        {
            long size = Check(SizeOf(Handle, snapshot ? 1u : 0u));
            var state = new byte[size];
            fixed (byte* data = state) Check(SaveState(Handle, data, (nuint)state.Length, snapshot ? 1u : 0u));
            return state;
        }

        // One line per field, "offset length type path", in the order the state writes them.
        public string Layout(bool snapshot)
        {
            long size = Check(LayoutOf(Handle, snapshot ? 1u : 0u, null, 0));
            var text = new byte[size];
            fixed (byte* data = text) Check(LayoutOf(Handle, snapshot ? 1u : 0u, data, (nuint)text.Length));
            return Encoding.UTF8.GetString(text);
        }

        public static string Describe(long status) => status switch
        {
            -1 => "no machine",
            -2 => "the state is truncated",
            -3 => "not a Mars save state",
            -4 => "an unknown state version",
            -5 => "an RDRAM size that is neither 4 MB nor 8 MB",
            -6 => "more pending display-processor words than a snapshot holds",
            -7 => "the buffer is too small",
            -8 => "pending display-processor words, which only a snapshot can carry",
            _ => $"status {status}",
        };

        private static long Check(long result) => result >= 0 ? result : throw new InvalidDataException($"MarsRT could not write the state: {Describe(result)}.");

        private nint Handle => _handle != 0 ? _handle : throw new ObjectDisposedException(nameof(MarsMachine));

        public void Dispose()
        {
            if (_handle != 0) Free(_handle);
            _handle = 0;
            GC.SuppressFinalize(this);
        }

        ~MarsMachine()
        {
            if (_handle != 0) Free(_handle);
        }
    }
}
