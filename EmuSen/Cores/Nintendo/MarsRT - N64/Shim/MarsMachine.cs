using System;
using System.IO;
using System.Text;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Native;

namespace EmuSen.Cores.Nintendo.MarsRT
{
    // MarsRT's machine behind its handle: for now its state, read and written in the C# Mars's own format - see Mars_Native.md §5.1.
    public sealed unsafe class MarsMachine : NativeMachine
    {
        private static readonly NativeExports Exports = new(MarsNative.Library, "mars_machine_", lifecycleOnly: true);
        private static readonly delegate* unmanaged<uint, nint> New = (delegate* unmanaged<uint, nint>)MarsNative.Export("mars_machine_new");
        private static readonly delegate* unmanaged<nint, uint> RdramOf = (delegate* unmanaged<nint, uint>)MarsNative.Export("mars_machine_rdram_bytes");
        private static readonly delegate* unmanaged<nint, int> KindOf = (delegate* unmanaged<nint, int>)MarsNative.Export("mars_machine_state_kind");
        private static readonly delegate* unmanaged<nint, uint, long> SizeOf = (delegate* unmanaged<nint, uint, long>)MarsNative.Export("mars_machine_save_state_size");
        private static readonly delegate* unmanaged<nint, byte*, nuint, uint, long> SaveState = (delegate* unmanaged<nint, byte*, nuint, uint, long>)MarsNative.Export("mars_machine_save_state");
        private static readonly delegate* unmanaged<nint, uint, byte*, nuint, long> LayoutOf = (delegate* unmanaged<nint, uint, byte*, nuint, long>)MarsNative.Export("mars_machine_state_layout");

        // The version a loaded state had; the numbers are MarsCore's own.
        public const int StateKind = MarsCore.StateVersion, SnapshotKind = MarsCore.SnapshotVersion;

        public static bool Available => New != null;

        public MarsMachine(int rdramBytes) : base(Exports, "MarsRT", Describe)
        {
            if (!Available) throw new InvalidOperationException($"MarsRT is not in use: {MarsNative.Report}");
            nint handle = New((uint)rdramBytes);
            if (handle == 0) throw new ArgumentOutOfRangeException(nameof(rdramBytes), rdramBytes, "RDRAM is 4 MB or 8 MB.");
            Attach(handle);
        }

        public int RdramBytes => (int)RdramOf(Handle);

        // 1 after a state, 2 after a snapshot, 0 before any load.
        public int LoadedKind => KindOf(Handle);

        // A state or a snapshot; the size and the write each take the flag, so the shared two-argument form does not fit.
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

        public static string Describe(long status) => Describe(status, "Mars", status => status switch
        {
            -12 => "an RDRAM size that is neither 4 MB nor 8 MB",
            -13 => "more pending display-processor words than a snapshot holds",
            -14 => "pending display-processor words, which only a snapshot can carry",
            _ => null,
        });
    }
}
