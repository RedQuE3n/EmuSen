using System;
using System.Collections.Generic;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;

namespace EmuSen.Cores.Native
{
    // A core refused a call: its status, and its own words for it from last_error or status_text - see EmuSen_CoreAPI.md §6.18.
    public sealed class CoreRefusedException(int status, string message) : InvalidOperationException(message)
    {
        public int Status { get; } = status;

        // The detail word a failed advance or debug frame returned beside its status, zero for any other call.
        public ulong Detail { get; init; }
    }

    // One machine of a v1 library: its handle and one method per export, every refusal thrown with the core's words - see EmuSen_CoreAPI.md §19.
    public sealed unsafe class CoreMachine : IDisposable
    {
        private nint _handle;
        private readonly CoreInterface _api;

        public CoreLibrary Library { get; }
        public CoreMachineInfo Info { get; private set; }

        // The image, the settings text and the files, and the pixel formats offered: RGBA8888 only, the one ICore knows.
        public CoreMachine(CoreLibrary library, byte[] image, string settings, IReadOnlyList<(uint Which, byte[] Data)> files)
        {
            if (!library.Available) throw new InvalidOperationException($"{library.Info?.DisplayName ?? library.Path} is not in use: {library.Report}");
            Library = library;
            _api = library.Api;
            byte[] text = Encoding.UTF8.GetBytes(settings);
            var pins = new GCHandle[files.Count];
            var entries = new CoreInterface.File[files.Count];
            var error = new byte[1024];
            try
            {
                for (int i = 0; i < files.Count; i++)
                {
                    pins[i] = GCHandle.Alloc(files[i].Data, GCHandleType.Pinned);
                    entries[i] = new CoreInterface.File { Size = (uint)sizeof(CoreInterface.File), Which = files[i].Which, Data = (byte*)pins[i].AddrOfPinnedObject(), Len = (nuint)files[i].Data.Length };
                }
                int status;
                nint handle;
                fixed (byte* img = image)
                fixed (byte* s = text)
                fixed (CoreInterface.File* f = entries)
                fixed (byte* e = error)
                {
                    var p = new CoreInterface.CreateParams
                    {
                        Size = (uint)sizeof(CoreInterface.CreateParams),
                        HostAbiVersion = CoreInterface.Version,
                        Image = img,
                        ImageLen = (nuint)image.Length,
                        Settings = s,
                        SettingsLen = (nuint)text.Length,
                        Files = f,
                        FileCount = (nuint)entries.Length,
                        FileSize = (nuint)sizeof(CoreInterface.File),
                        PixelFormats = 1,
                        Error = e,
                        ErrorLen = (nuint)error.Length,
                    };
                    handle = _api.Create(&p, &status);
                }
                if (handle == 0)
                {
                    int end = Array.IndexOf(error, (byte)0);
                    string words = end > 0 ? Encoding.UTF8.GetString(error, 0, end) : library.Words(status);
                    throw new CoreRefusedException(status, $"{library.Info.DisplayName} refused the game: {words}.");
                }
                _handle = handle;
            }
            finally
            {
                foreach (var pin in pins) if (pin.IsAllocated) pin.Free();
            }
            Info = ReadInfo();
        }

        public nint Handle => _handle != 0 ? _handle : throw new ObjectDisposedException(nameof(CoreMachine));

        public bool Has(ulong capability) => Library.Has(capability);

        private string Name => Library.Info.DisplayName;

        // The words for a refusal: the machine's last_error, else status_text.
        public string Words(long status)
        {
            string detail = TextOf(_api.LastError);
            return detail.Length > 0 ? detail : Library.Words(status);
        }

        private void Ok(long status, string what)
        {
            if (status < 0) throw new CoreRefusedException((int)status, $"{Name} could not {what}: {Words(status)}.");
        }

        private string TextOf(delegate* unmanaged<nint, byte*, nuint, long> call)
        {
            nint h = Handle;
            long n = call(h, null, 0);
            if (n <= 0) return "";
            var bytes = new byte[n];
            fixed (byte* b = bytes) call(h, b, (nuint)n);
            return CoreLibrary.Utf8(bytes) ?? "";
        }

        // Re-read after create and after a MACHINE_INFO event.
        public CoreMachineInfo ReadInfo() => Info = CoreDescriptorReader.MachineInfo(TextOf(_api.MachineInfo), AudioSampleRate);

        public void Advance()
        {
            ulong detail;
            int status = _api.Advance(Handle, &detail);
            if (status != 0) throw new CoreRefusedException(status, $"{Name} stopped: {Words(status)} (detail {detail:X}).") { Detail = detail };
        }

        public void Present() => Ok(_api.Present(Handle), "present the picture");

        public void Reset() => Ok(_api.Reset(Handle), "reset");

        public void SetOptions(bool skipRendering) => _api.SetOptions(Handle, skipRendering ? 1u : 0u);

        public long TotalFrames => _api.FrameCount(Handle);

        public CoreInterface.FrameInfo FrameInfo
        {
            get
            {
                var info = new CoreInterface.FrameInfo { Size = (uint)sizeof(CoreInterface.FrameInfo) };
                Ok(_api.FrameInfoOf(Handle, &info), "describe its picture");
                return info;
            }
        }

        public long CopyFrame(byte[] into)
        {
            fixed (byte* data = into) return _api.FrameCopy(Handle, data, (nuint)into.Length);
        }

        public int AudioSampleRate => _api.AudioRate(Handle);

        public int BufferedSamples => (int)Math.Max(0, _api.AudioBuffered(Handle));

        // Whole stereo frames, never across a change of rate; the rate they were made at.
        public (short[] Samples, int Rate) DrainAudio(int maxFrames)
        {
            int rate;
            int wanted = (int)Math.Min((long)maxFrames * 2, BufferedSamples);
            wanted -= wanted & 1;
            if (wanted <= 0) return (Array.Empty<short>(), AudioSampleRate);
            var samples = new short[wanted];
            long n;
            fixed (short* data = samples) n = _api.AudioDrain(Handle, data, (nuint)samples.Length, maxFrames, &rate);
            if (n < wanted) Array.Resize(ref samples, (int)Math.Max(0, n));
            return (samples, rate);
        }

        public short[] PeekAudio()
        {
            if (!Has(CoreInterface.CapAudioPeek)) return Array.Empty<short>();
            var samples = new short[BufferedSamples];
            long n;
            fixed (short* data = samples) n = _api.AudioPeek(Handle, data, (nuint)samples.Length);
            return n < samples.Length ? samples[..(int)Math.Max(0, n)] : samples;
        }

        public void SetAudioLimit(int samples) => _api.SetAudioLimit(Handle, (ulong)Math.Max(0, samples));

        public void SetMutes(uint mask)
        {
            if (Has(CoreInterface.CapMutes)) _api.SetMutes(Handle, mask);
        }

        public void SetButtons(int port, uint mask, uint changed) => _api.SetButtons(Handle, (uint)port, mask, changed);

        public void SetAxis(int port, uint axis, double value)
        {
            if (Has(CoreInterface.CapAxes)) _api.SetAxis(Handle, (uint)port, axis, value);
        }

        public int StateSize(uint kind = 0)
        {
            long n = _api.StateSize(Handle, kind);
            Ok(n, "size its state");
            return (int)n;
        }

        public void Save(byte[] into, uint kind = 0)
        {
            long n;
            fixed (byte* data = into) n = _api.StateSave(Handle, kind, data, (nuint)into.Length);
            Ok(n, "write its state");
        }

        public void Load(ReadOnlySpan<byte> state)
        {
            int status;
            fixed (byte* data = state) status = _api.StateLoad(Handle, data, (nuint)state.Length);
            if (status != 0) throw new InvalidDataException($"{Name} refused the state: {Words(status)}.");
        }

        public string Layout(uint kind = 0)
        {
            long n = _api.StateLayout(Handle, kind, null, 0);
            Ok(n, "list its state");
            var bytes = new byte[n];
            fixed (byte* b = bytes) _api.StateLayout(Handle, kind, b, (nuint)n);
            return Encoding.UTF8.GetString(bytes);
        }

        public long SpaceSize(uint space) => Math.Max(0, _api.SpaceSize(Handle, space));

        public bool ReadSpace(uint space, int address, Span<byte> into)
        {
            fixed (byte* data = into) return _api.SpaceRead(Handle, space, (uint)address, data, (nuint)into.Length) >= 0;
        }

        public bool WriteSpace(uint space, int address, ReadOnlySpan<byte> from)
        {
            fixed (byte* data = from) return _api.SpaceWrite(Handle, space, (uint)address, data, (nuint)from.Length) >= 0;
        }

        // A battery file's bytes, empty where there is none, with EMUSEN_BATTERY_* flags.
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

        public void SetRomPatches(uint[] triples)
        {
            if (!Has(CoreInterface.CapRomPatches)) return;
            fixed (uint* words = triples) _api.SetRomPatches(Handle, words, (nuint)(triples.Length / 3));
        }

        public void SetCheatPokes(uint[] quads)
        {
            if (!Has(CoreInterface.CapCheatPokes)) return;
            fixed (uint* words = quads) _api.SetCheatPokes(Handle, words, (nuint)(quads.Length / 4));
        }

        // Run-scope "key=value" lines applied together; the core's words on refusal.
        public void SetSettings(string text)
        {
            if (!Has(CoreInterface.CapSettings)) throw new CoreRefusedException(CoreInterface.StatusNotSupported, $"{Name} takes no settings between frames.");
            byte[] bytes = Encoding.UTF8.GetBytes(text);
            int status;
            fixed (byte* b = bytes) status = _api.SetSettings(Handle, b, (nuint)bytes.Length);
            if (status != 0) throw new CoreRefusedException(status, $"{Name} refused the settings: {Words(status)}.");
        }

        public IReadOnlyDictionary<string, string> SettingNotes() =>
            Has(CoreInterface.CapSettingNotes) && TextOf(_api.SettingNotes) is { Length: > 0 } t ? CoreDescriptorReader.Notes(t) : new Dictionary<string, string>();

        public long[] Phases()
        {
            if (!Has(CoreInterface.CapPhases)) return Array.Empty<long>();
            long n = _api.Phases(Handle, null, 0);
            if (n <= 0) return Array.Empty<long>();
            var values = new long[n];
            fixed (long* v = values) _api.Phases(Handle, v, (nuint)n);
            return values;
        }

        public long[] Registers(uint processor)
        {
            if (!Has(CoreInterface.CapDebugRegisters)) return Array.Empty<long>();
            long n = _api.DebugRegisters(Handle, processor, null, 0);
            if (n <= 0) return Array.Empty<long>();
            var values = new long[n];
            fixed (long* v = values) _api.DebugRegisters(Handle, processor, v, (nuint)n);
            return values;
        }

        public string Disassemble(uint processor, uint space, uint address, uint count)
        {
            if (!Has(CoreInterface.CapDebugDisassemble)) return "[]";
            long n = _api.DebugDisassemble(Handle, processor, space, address, count, null, 0);
            if (n <= 0) return "[]";
            var bytes = new byte[n];
            fixed (byte* b = bytes) _api.DebugDisassemble(Handle, processor, space, address, count, b, (nuint)n);
            return CoreLibrary.Utf8(bytes) ?? "[]";
        }

        // Every event waiting, oldest first.
        public IReadOnlyList<CoreInterface.Event> DrainEvents()
        {
            long n = _api.Events(Handle, null, 0, 0);
            if (n <= 0) return Array.Empty<CoreInterface.Event>();
            var events = new CoreInterface.Event[n];
            for (int i = 0; i < events.Length; i++) events[i].Size = (uint)sizeof(CoreInterface.Event);
            fixed (CoreInterface.Event* e = events) n = _api.Events(Handle, e, (nuint)events.Length, (nuint)sizeof(CoreInterface.Event));
            return n == events.Length ? events : events[..(int)Math.Max(0, n)];
        }

        public IReadOnlyList<string> DrainLog() => DrainLog(_api, Handle);

        // A queue's records as lines; machine 0 is the library's own queue.
        public static IReadOnlyList<string> DrainLog(CoreInterface api, nint machine)
        {
            var lines = new List<string>();
            for (int round = 0; round < 8; round++)
            {
                long n = api.LogDrain(machine, null, 0);
                if (n <= 0) break;
                var bytes = new byte[n];
                fixed (byte* b = bytes) n = api.LogDrain(machine, b, (nuint)n);
                if (n <= 0) break;
                foreach (string line in Encoding.UTF8.GetString(bytes, 0, (int)n).Split('\n', StringSplitOptions.RemoveEmptyEntries)) lines.Add(line);
            }
            return lines;
        }

        public void Dispose()
        {
            if (_handle != 0) _api.Free(_handle);
            _handle = 0;
            GC.SuppressFinalize(this);
        }

        ~CoreMachine()
        {
            if (_handle != 0) _api.Free(_handle);
        }
    }
}
