using System;
using System.IO;
using EmuSen.Cores.Native;

namespace EmuSen.Cores.Nintendo.MarsRT
{
    // MarsRT's machine behind a v1 handle, for its state alone: read and written in the Mars state format, nothing derived - see Mars_Native.md §5.1, §7.2.
    public sealed unsafe class MarsMachine : IDisposable
    {
        private static readonly delegate* unmanaged<nint, byte*, nuint, int> LoadFields = (delegate* unmanaged<nint, byte*, nuint, int>)MarsNative.Export("marsrt_core_load_fields");
        private static readonly delegate* unmanaged<nint, int> KindOf = (delegate* unmanaged<nint, int>)MarsNative.Export("marsrt_core_state_kind");

        // "MARS" little-endian, then the format's version: a state, or a snapshot with the display processor's pending words - see Mars_SaveStates.md §1.
        public const uint StateMagic = 0x5352_414D;
        public const int StateKind = 1, SnapshotKind = 2;

        // The display processor's pending words a snapshot's fixed tail holds - see Mars_SaveStates.md §1.
        public const int SnapshotWords = 1 << 15;

        private const int Rdram = 4 * 1024 * 1024, ExpandedRdram = 8 * 1024 * 1024;

        // An image the library takes for a cartridge, which no state names: the magic word and an empty boot code.
        private static readonly byte[] Image = MakeImage();

        private readonly CoreMachine _machine;

        public static bool Available => MarsNative.Available && LoadFields != null;

        public MarsMachine(int rdramBytes)
        {
            if (!Available) throw new InvalidOperationException($"MarsRT is not in use: {MarsNative.Report}");
            if (rdramBytes != Rdram && rdramBytes != ExpandedRdram) throw new ArgumentOutOfRangeException(nameof(rdramBytes), rdramBytes, "RDRAM is 4 MB or 8 MB.");
            string settings = $"ExpansionPak={(rdramBytes == ExpandedRdram ? "true" : "false")}\nThreadedRdp=false\nDeferredPresentation=false\nRecompiler=false";
            _machine = new CoreMachine(MarsNative.Library.Library!, Image, settings, Array.Empty<(uint, byte[])>());
        }

        public int RdramBytes => (int)_machine.SpaceSize((uint)MarsRtSpace.Rdram);

        // 1 after a state, 2 after a snapshot, 0 before any load.
        public int LoadedKind => KindOf(_machine.Handle);

        // A failed load leaves the machine as it was.
        public void Load(ReadOnlySpan<byte> state)
        {
            int status;
            lock (_machine.Gate)
                fixed (byte* data = state) status = LoadFields(_machine.Handle, data, (nuint)state.Length);
            if (status != 0) throw new InvalidDataException($"MarsRT refused the state: {Describe(status)}.");
        }

        // A state or a snapshot, the clocks settled first.
        public byte[] Save(bool snapshot)
        {
            uint kind = snapshot ? 1u : 0;
            try
            {
                var state = new byte[_machine.StateSize(kind)];
                _machine.Save(state, kind);
                return state;
            }
            catch (CoreRefusedException e) { throw new InvalidDataException($"MarsRT could not write the state: {Describe(e.Status)}.", e); }
        }

        // One line per field, "offset length type path", in the order the state writes them.
        public string Layout(bool snapshot) => _machine.Layout(snapshot ? 1u : 0);

        // emusen-native's shared codes first, then MarsRT's own band, else "status n".
        public static string Describe(long status) => status switch
        {
            -1 => "no machine",
            -2 => "the state is truncated",
            -3 => "not a Mars save state",
            -4 => "an unknown state version",
            -5 => "a string length the reader refuses",
            -7 => "the buffer is too small",
            -12 => "an RDRAM size that is neither 4 MB nor 8 MB",
            -13 => "more pending display-processor words than a snapshot holds",
            -14 => "pending display-processor words, which only a snapshot can carry",
            _ => $"status {status}",
        };

        private static byte[] MakeImage()
        {
            var image = new byte[0x2000];
            image[0] = 0x80;
            image[1] = 0x37;
            image[2] = 0x12;
            image[3] = 0x40;
            return image;
        }

        public void Dispose() => _machine.Dispose();
    }
}
