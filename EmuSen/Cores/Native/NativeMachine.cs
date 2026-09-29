using System;
using System.IO;
using System.Text;

namespace EmuSen.Cores.Native
{
    // The exports every Rust core has under today's per-core names, resolved once per library; zero where a library lacks one - see EmuSen_NativeCores.md §4.2.
    public sealed unsafe class NativeExports
    {
        public readonly delegate* unmanaged<nint, void> Free;
        public readonly delegate* unmanaged<nint, byte*, nuint, int> LoadState;
        public readonly delegate* unmanaged<nint, long> SaveStateSize;
        public readonly delegate* unmanaged<nint, byte*, nuint, long> SaveState;
        public readonly delegate* unmanaged<nint, byte*, nuint, long> StateLayout;
        public readonly delegate* unmanaged<nint, uint, void> SetOptions;
        public readonly delegate* unmanaged<nint, uint, void> SetMutes;
        public readonly delegate* unmanaged<nint, ulong, void> SetAudioLimit;
        public readonly delegate* unmanaged<nint, byte*, nuint, long> Frame;
        public readonly delegate* unmanaged<nint, long> AudioBuffered;
        public readonly delegate* unmanaged<nint, short*, nuint, long, long> DrainAudio;
        public readonly delegate* unmanaged<nint, long> TotalFrames;
        public readonly delegate* unmanaged<nint, uint, long> SpaceSize;
        public readonly delegate* unmanaged<nint, uint, int, byte*, nuint, long> ReadSpace;
        public readonly delegate* unmanaged<nint, uint, int, byte*, nuint, long> WriteSpace;
        public readonly delegate* unmanaged<nint, ushort*, ushort*, nuint, void> SetRomPatches;
        public readonly bool LifecycleOnly;

        // prefix is "moon_machine_" or "mercury_machine_"; lifecycleOnly resolves free and load_state alone, for MarsRT, whose other exports differ in shape.
        public NativeExports(NativeCoreLibrary library, string prefix, bool lifecycleOnly = false)
        {
            Free = (delegate* unmanaged<nint, void>)library.Export(prefix + "free");
            LoadState = (delegate* unmanaged<nint, byte*, nuint, int>)library.Export(prefix + "load_state");
            LifecycleOnly = lifecycleOnly;
            if (lifecycleOnly) return;
            SaveStateSize = (delegate* unmanaged<nint, long>)library.Export(prefix + "save_state_size");
            SaveState = (delegate* unmanaged<nint, byte*, nuint, long>)library.Export(prefix + "save_state");
            StateLayout = (delegate* unmanaged<nint, byte*, nuint, long>)library.Export(prefix + "state_layout");
            SetOptions = (delegate* unmanaged<nint, uint, void>)library.Export(prefix + "set_options");
            SetMutes = (delegate* unmanaged<nint, uint, void>)library.Export(prefix + "set_mutes");
            SetAudioLimit = (delegate* unmanaged<nint, ulong, void>)library.Export(prefix + "set_audio_limit");
            Frame = (delegate* unmanaged<nint, byte*, nuint, long>)library.Export(prefix + "frame");
            AudioBuffered = (delegate* unmanaged<nint, long>)library.Export(prefix + "audio_buffered");
            DrainAudio = (delegate* unmanaged<nint, short*, nuint, long, long>)library.Export(prefix + "drain_audio");
            TotalFrames = (delegate* unmanaged<nint, long>)library.Export(prefix + "total_frames");
            SpaceSize = (delegate* unmanaged<nint, uint, long>)library.Export(prefix + "space_size");
            ReadSpace = (delegate* unmanaged<nint, uint, int, byte*, nuint, long>)library.Export(prefix + "read_space");
            WriteSpace = (delegate* unmanaged<nint, uint, int, byte*, nuint, long>)library.Export(prefix + "write_space");
            SetRomPatches = (delegate* unmanaged<nint, ushort*, ushort*, nuint, void>)library.Export(prefix + "set_rom_patches");
        }
    }

    // A Rust machine behind its handle: the handle's life, the shared status codes and the common calls - see EmuSen_NativeCores.md §4.2.
    public abstract unsafe class NativeMachine : IDisposable
    {
        private nint _handle;
        private readonly NativeExports _exports;

        private readonly Func<long, string> _describe;

        // engine names the library in messages ("MoonRT"); describe is the console's static table over the shared one.
        protected NativeMachine(NativeExports exports, string engine, Func<long, string> describe)
        {
            _exports = exports;
            Engine = engine;
            _describe = describe;
        }

        public string Engine { get; }

        // Set once by the subclass's constructor, from its own create export.
        protected void Attach(nint handle) => _handle = handle;

        public nint Handle => _handle != 0 ? _handle : throw new ObjectDisposedException(GetType().Name);

        // emusen-native's shared codes first, then the console's own band below -8, else "status n".
        public static string Describe(long status, string stateName, Func<long, string?> own) => Shared(status, stateName) ?? own(status) ?? $"status {status}";

        // emusen-native's shared codes, -1 to -8, the same in every core.
        private static string? Shared(long status, string stateName) => status switch
        {
            -1 => "no machine",
            -2 => "the state is truncated",
            -3 => $"not a {stateName} save state",
            -4 => "an unknown state version",
            -5 => "a string length the C# reader refuses",
            -7 => "the buffer is too small",
            _ => null,
        };

        protected long Check(long result) => result >= 0 ? result : throw new InvalidDataException($"{Engine} could not write the state: {_describe(result)}.");

        // A failed load leaves the machine as it was.
        public void Load(ReadOnlySpan<byte> state)
        {
            int status;
            fixed (byte* data = state) status = _exports.LoadState(Handle, data, (nuint)state.Length);
            if (status != 0) throw new InvalidDataException($"{Engine} refused the state: {_describe(status)}.");
        }

        public byte[] Save()
        {
            long size = Check(_exports.SaveStateSize(Live));
            var state = new byte[size];
            fixed (byte* data = state) Check(_exports.SaveState(Live, data, (nuint)state.Length));
            return state;
        }

        // One line per field, "offset length type path", in the order the state writes them.
        public string Layout() => Encoding.UTF8.GetString(Query(_exports.LifecycleOnly ? throw new NotSupportedException($"{Engine} lays its state out by kind.") : _exports.StateLayout));

        // The length-query idiom: asked with no buffer for the length, then again to fill one.
        protected byte[] Query(delegate* unmanaged<nint, byte*, nuint, long> export)
        {
            long size = Check(export(Handle, null, 0));
            var bytes = new byte[size];
            fixed (byte* data = bytes) Check(export(Handle, data, (nuint)bytes.Length));
            return bytes;
        }

        // The handle, for a call only a fully resolved table has; MarsRT's present exports differ in shape, so its table is lifecycle only.
        private nint Live => _exports.LifecycleOnly ? throw new NotSupportedException($"{Engine} has no such export in its present interface.") : Handle;

        public long TotalFrames => _exports.TotalFrames(Live);

        public void SetOptions(bool skipRendering) => _exports.SetOptions(Live, skipRendering ? 1u : 0u);

        public void SetMutes(uint mask) => _exports.SetMutes(Live, mask);

        // The most samples the queue holds before the oldest pair goes, as C#'s SampleQueue.
        public void SetAudioLimit(int samples) => _exports.SetAudioLimit(Live, (ulong)Math.Max(0, samples));

        public void CopyFrame(byte[] into)
        {
            fixed (byte* data = into) _exports.Frame(Live, data, (nuint)into.Length);
        }

        public int BufferedSamples => (int)_exports.AudioBuffered(Live);

        public short[] DrainAudio(int maxFrames)
        {
            int wanted = (int)Math.Min((long)maxFrames * 2, BufferedSamples);
            wanted -= wanted & 1;
            if (wanted <= 0) return Array.Empty<short>();
            var samples = new short[wanted];
            fixed (short* data = samples) _exports.DrainAudio(Live, data, (nuint)samples.Length, maxFrames);
            return samples;
        }

        public int SpaceSize(int space) => (int)_exports.SpaceSize(Live, (uint)space);

        // Read as the C# core's ReadSpace reads, a byte at a time from address on; a bus space's reads have their side effects.
        public void ReadSpace(int space, int address, Span<byte> into)
        {
            fixed (byte* data = into) _exports.ReadSpace(Live, (uint)space, address, data, (nuint)into.Length);
        }

        public void WriteSpace(int space, int address, ReadOnlySpan<byte> from)
        {
            fixed (byte* data = from) _exports.WriteSpace(Live, (uint)space, address, data, (nuint)from.Length);
        }

        // Addresses, then 256 entries each: 0x100 | patched for an original byte a patch replaces, 0 where none does.
        public void SetRomPatches(ushort[] addresses, ushort[] tables)
        {
            fixed (ushort* a = addresses)
            fixed (ushort* t = tables)
            {
                _exports.SetRomPatches(Live, a, t, (nuint)addresses.Length);
            }
        }

        public void Dispose()
        {
            if (_handle != 0) _exports.Free(_handle);
            _handle = 0;
            GC.SuppressFinalize(this);
        }

        ~NativeMachine()
        {
            if (_handle != 0) _exports.Free(_handle);
        }
    }
}
