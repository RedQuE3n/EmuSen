using System;
using System.Collections.Generic;
using System.IO;
using System.Text;

namespace EmuSen.Cores.Native
{
    // A machine behind the common native interface: its handle, the shared statuses mapped, one method per export - see EmuSen_NativeCores.md §4.2.
    public abstract unsafe class NativeMachine : IDisposable
    {
        private nint _handle;
        private readonly NativeInterface _api;
        private readonly Func<int, Exception?> _refusal;
        private readonly Func<long, string?> _describe;
        private readonly string _stateName;

        // engine names the library in messages ("MoonRT"); refusal and describe are the console's own band, asked before the shared codes.
        protected NativeMachine(NativeInterface api, string engine, string stateName, Func<int, Exception?> refusal, Func<long, string?> describe,
            ReadOnlySpan<byte> image, string settings, IReadOnlyList<(uint Which, byte[] Data)> files)
        {
            _api = api;
            Engine = engine;
            _stateName = stateName;
            _refusal = refusal;
            _describe = describe;
            if (!api.Library.Available || !api.Complete) throw new InvalidOperationException($"{engine} is not in use: {api.Library.Report}");

            byte[] text = Encoding.UTF8.GetBytes(settings);
            var pins = new System.Runtime.InteropServices.GCHandle[files.Count];
            var entries = new NativeInterface.NativeFile[files.Count];
            try
            {
                for (int i = 0; i < files.Count; i++)
                {
                    pins[i] = System.Runtime.InteropServices.GCHandle.Alloc(files[i].Data, System.Runtime.InteropServices.GCHandleType.Pinned);
                    entries[i] = new NativeInterface.NativeFile { Which = files[i].Which, Data = (byte*)pins[i].AddrOfPinnedObject(), Length = (nuint)files[i].Data.Length };
                }
                int status;
                nint handle;
                fixed (byte* data = image)
                fixed (byte* s = text)
                fixed (NativeInterface.NativeFile* f = entries)
                {
                    handle = api.Create(data, (nuint)image.Length, s, (nuint)text.Length, f, (nuint)entries.Length, &status);
                }
                if (handle == 0) throw ExceptionFor(status);
                _handle = handle;
            }
            finally
            {
                foreach (var pin in pins) if (pin.IsAllocated) pin.Free();
            }
        }

        public string Engine { get; }

        public NativeInterface Api => _api;

        public nint Handle => _handle != 0 ? _handle : throw new ObjectDisposedException(GetType().Name);

        // The shared codes' words: emusen-native's -1 to -8, then the interface's own.
        public static string? Shared(long status, string stateName) => status switch
        {
            -1 => "no machine",
            -2 => "the state is truncated",
            -3 => $"not a {stateName} save state",
            -4 => "an unknown state version",
            -5 => "a string length the C# reader refuses",
            -7 => "the buffer is too small",
            NativeInterface.NotSupported => "not supported by this core",
            NativeInterface.NoSuchSpace => "no such memory space",
            NativeInterface.ReadOnly => "a read-only memory space",
            NativeInterface.UnknownSetting => "an unknown setting",
            NativeInterface.BadSetting => "a setting value the core cannot take",
            NativeInterface.NoSuchPort => "no such port",
            NativeInterface.BadFile => "a file the core refuses",
            <= NativeInterface.FaultBase - 1 and >= -383 => $"the C# exception of kind {NativeInterface.FaultBase - status}",
            _ => null,
        };

        public string Words(long status) => _describe(status) ?? Shared(status, _stateName) ?? $"status {status}";

        // The console's exception for its own band first; then the reproduced C# exceptions, whose types are the oracle's - see EmuSen_NativeCores.md §3.3.
        public Exception ExceptionFor(int status) => _refusal(status) ?? status switch
        {
            NativeInterface.FaultBase - 1 => new IndexOutOfRangeException("Index was outside the bounds of the array."),
            NativeInterface.FaultBase - 2 => new DivideByZeroException("Attempted to divide by zero."),
            NativeInterface.FaultBase - 3 => new ArgumentOutOfRangeException(),
            NativeInterface.FaultBase - 4 => new InvalidOperationException($"{Engine} reproduced an InvalidOperationException."),
            NativeInterface.FaultBase - 5 => new OverflowException(),
            _ => new InvalidOperationException($"{Engine} refused: {Words(status)}."),
        };

        private long Check(long result) => result >= 0 ? result : throw new InvalidDataException($"{Engine} could not write the state: {Words(result)}.");

        private void Ok(int status)
        {
            if (status != 0) throw ExceptionFor(status);
        }

        public void Reset() => Ok(_api.ResetOf != null ? _api.ResetOf(Handle) : NativeInterface.NotSupported);

        // The machine to the frame's end; a failure is thrown as the oracle's exception.
        public void Advance()
        {
            ulong detail;
            int status = _api.Advance(Handle, &detail);
            if (status != 0) throw FrameFailure(status, detail);
        }

        // A frame's failure with the core's detail word, which some of a console's exceptions are built from.
        public Exception FrameFailure(int status, ulong detail) => FrameException(status, detail) ?? ExceptionFor(status);

        protected virtual Exception? FrameException(int status, ulong detail) => null;

        public void SetOptions(bool skipRendering) => _api.SetOptions(Handle, skipRendering ? 1u : 0u);

        public long TotalFrames => _api.FrameCount(Handle);

        public NativeInterface.FrameInfo FrameInfo
        {
            get
            {
                NativeInterface.FrameInfo info;
                Ok(_api.FrameInfoOf(Handle, &info));
                return info;
            }
        }

        public void CopyFrame(byte[] into)
        {
            fixed (byte* data = into) _api.FrameCopy(Handle, data, (nuint)into.Length);
        }

        public int AudioSampleRate => _api.AudioRate(Handle);

        public int BufferedSamples => (int)_api.AudioBuffered(Handle);

        public short[] DrainAudio(int maxFrames)
        {
            int wanted = (int)Math.Min((long)maxFrames * 2, BufferedSamples);
            wanted -= wanted & 1;
            if (wanted <= 0) return Array.Empty<short>();
            var samples = new short[wanted];
            fixed (short* data = samples) _api.AudioDrain(Handle, data, (nuint)samples.Length, maxFrames);
            return samples;
        }

        // The most samples the queue holds before the oldest pair goes, as C#'s SampleQueue.
        public void SetAudioLimit(int samples) => _api.SetAudioLimit(Handle, (ulong)Math.Max(0, samples));

        public void SetMutes(uint mask)
        {
            if (_api.SetMutes != null) _api.SetMutes(Handle, mask);
        }

        // Bits in changed take mask's values: a console whose pad is in no state sends 0xFF, one whose pad is sends the bit that moved - see EmuSen_NativeCores.md §3.8.
        public void SetButtons(int port, uint mask, uint changed = 0xFF) => _api.SetButtons(Handle, (uint)port, mask, changed);

        public void Load(ReadOnlySpan<byte> state)
        {
            int status;
            fixed (byte* data = state) status = _api.StateLoad(Handle, data, (nuint)state.Length);
            if (status != 0) throw new InvalidDataException($"{Engine} refused the state: {Words(status)}.");
        }

        public int StateSize => (int)Check(_api.StateSize(Handle, 0));

        public byte[] Save()
        {
            var state = new byte[StateSize];
            Save(state);
            return state;
        }

        // Into a caller's array of StateSize bytes, so a rewind capture allocates nothing - see EmuSen_NativeCores.md §3.9.
        public void Save(byte[] into)
        {
            fixed (byte* data = into) Check(_api.StateSave(Handle, 0, data, (nuint)into.Length));
        }

        // One line per field, "offset length type path", in the order the state writes them.
        public string Layout()
        {
            long size = Check(_api.StateLayout(Handle, 0, null, 0));
            var bytes = new byte[size];
            fixed (byte* data = bytes) Check(_api.StateLayout(Handle, 0, data, (nuint)bytes.Length));
            return Encoding.UTF8.GetString(bytes);
        }

        public int SpaceSize(int space) => (int)Math.Max(0, _api.SpaceSize(Handle, (uint)space));

        // Read as the C# core's ReadSpace reads, a byte at a time from address on; a bus space's reads have their side effects.
        public void ReadSpace(int space, int address, Span<byte> into)
        {
            fixed (byte* data = into) _api.SpaceRead(Handle, (uint)space, (uint)address, data, (nuint)into.Length);
        }

        public void WriteSpace(int space, int address, ReadOnlySpan<byte> from)
        {
            fixed (byte* data = from) _api.SpaceWrite(Handle, (uint)space, (uint)address, data, (nuint)from.Length);
        }

        // A battery file's bytes, empty where the cartridge has none, with its flags: bit 0 changed, bit 1 tracked.
        public (byte[] Data, uint Flags) Battery(uint which)
        {
            uint flags;
            long length = _api.Battery(Handle, which, null, 0, &flags);
            if (length <= 0) return (Array.Empty<byte>(), flags);
            var data = new byte[length];
            fixed (byte* d = data) _api.Battery(Handle, which, d, (nuint)data.Length, &flags);
            return (data, flags);
        }

        public void BatterySaved(uint which) => _api.BatterySaved(Handle, which);

        // CheatRegistry.ResolveRomPatches' list as (address, value, compare) triples, uint.MaxValue for no compare.
        public void SetRomPatches(uint[] triples)
        {
            if (_api.SetRomPatches == null) return;
            fixed (uint* words = triples) _api.SetRomPatches(Handle, words, (nuint)(triples.Length / 3));
        }

        public void Dispose()
        {
            if (_handle != 0) _api.Free(_handle);
            _handle = 0;
            GC.SuppressFinalize(this);
        }

        ~NativeMachine()
        {
            if (_handle != 0) _api.Free(_handle);
        }
    }
}
