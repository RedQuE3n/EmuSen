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

    // Bytes a call passes where the host already holds them, a mapped file's among them; the host keeps them in place for the call, and past it where it lends them.
    public readonly unsafe struct CoreBytes(byte* data, long length)
    {
        public byte* Data { get; } = data;
        public nuint Length { get; } = (nuint)length;
    }

    // One machine of a v1 library: its handle and one method per export, every refusal thrown with the core's words, one call at a time - see EmuSen_CoreAPI.md §6.16, §19.
    public sealed unsafe class CoreMachine : IDisposable
    {
        private nint _handle;
        private readonly CoreInterface _api;
        private readonly object _gate = new();

        // What keeps each lent disc in place, by its drive, released when the loan ends: at free, or when that drive takes another disc - see EmuSen_CoreAPI.md §6.24.
        private readonly Dictionary<uint, IDisposable> _loans = new();

        // What keeps the disc lent to a drive in place, or null where nothing was lent to it - see EmuSen_CoreAPI.md §6.25.
        public IDisposable? LoanOf(uint drive)
        {
            lock (_gate) return _loans.GetValueOrDefault(drive);
        }

        // Held by every call on the handle, here and by a caller of Handle, so no two threads are ever in the machine at once - see Diamond_Players.md §12.7.
        public object Gate => _gate;

        public CoreLibrary Library { get; }
        public CoreMachineInfo Info { get; private set; }

        // The image, the settings text and the files, and the pixel formats offered: RGBA8888 only, the one ICore knows.
        public CoreMachine(CoreLibrary library, byte[] image, string settings, IReadOnlyList<(uint Which, byte[] Data)> files)
        {
            if (!library.Available) throw new InvalidOperationException($"{library.Info?.DisplayName ?? library.Path} is not in use: {library.Report}");
            Library = library;
            _api = library.Api;
            fixed (byte* img = image) _handle = Create(new CoreBytes(img, image.Length), Array.Empty<(uint, CoreBytes)>(), settings, files, lent: false, sharedRemovable: false, sharedConsole: false);
            Info = ReadInfo();
        }

        // The same from bytes the host holds in place, the media files among them; with a loan they are lent, and the loan is released when the machine is freed or its drive takes another disc.
        // sharedRemovable and sharedConsole ask the core to leave those kinds of battery file out of what a state load restores - see EmuSen_CoreAPI.md §6.27, §6.28.
        // device is the host's Vulkan device lent to a core that claims DEVICE_PICTURE, the machine's from then on - see EmuSen_CoreAPI.md §6.29.
        public CoreMachine(CoreLibrary library, CoreBytes image, IReadOnlyList<(uint Which, CoreBytes Data)> media, string settings, IReadOnlyList<(uint Which, byte[] Data)> files, IDisposable? loan = null, bool sharedRemovable = false, bool sharedConsole = false, EmuSen.Serenity.Slang.SlangLend? device = null)
        {
            Lend = device is not null && library.Has(CoreInterface.CapDevicePicture) ? device : null;
            if (Lend is null) device?.Dispose();
            if (!library.Available) throw new InvalidOperationException($"{library.Info?.DisplayName ?? library.Path} is not in use: {library.Report}");
            Library = library;
            _api = library.Api;
            try { _handle = Create(image, media, settings, files, lent: loan is not null, sharedRemovable, sharedConsole, Lend is { } l ? LendOf(l) : null); }
            catch
            {
                // A refused create lends nothing.
                loan?.Dispose();
                Lend?.Dispose();
                throw;
            }
            Info = ReadInfo();
            if (loan is not null) _loans[Info.Drives.Count > 0 ? Info.Drives[0].Drive : 0] = loan;
        }

        // The file entries of a call: the pinned arrays, then the bytes the host holds in place.
        private static CoreInterface.File[] Entries(IReadOnlyList<(uint Which, byte[] Data)> files, IReadOnlyList<(uint Which, CoreBytes Data)> media, GCHandle[] pins)
        {
            var entries = new CoreInterface.File[files.Count + media.Count];
            for (int i = 0; i < files.Count; i++)
            {
                pins[i] = GCHandle.Alloc(files[i].Data, GCHandleType.Pinned);
                entries[i] = new CoreInterface.File { Size = (uint)sizeof(CoreInterface.File), Which = files[i].Which, Data = (byte*)pins[i].AddrOfPinnedObject(), Len = (nuint)files[i].Data.Length };
            }
            for (int i = 0; i < media.Count; i++)
                entries[files.Count + i] = new CoreInterface.File { Size = (uint)sizeof(CoreInterface.File), Which = media[i].Which, Data = media[i].Data.Data, Len = media[i].Data.Length };
            return entries;
        }

        private nint Create(CoreBytes image, IReadOnlyList<(uint Which, CoreBytes Data)> media, string settings, IReadOnlyList<(uint Which, byte[] Data)> files, bool lent, bool sharedRemovable, bool sharedConsole, CoreInterface.DeviceLend? device = null)
        {
            CoreInterface.DeviceLend lend = device ?? default;
            lend.Size = (uint)sizeof(CoreInterface.DeviceLend);
            byte[] text = Encoding.UTF8.GetBytes(settings);
            var pins = new GCHandle[files.Count];
            var error = new byte[1024];
            try
            {
                var entries = Entries(files, media, pins);
                int status;
                nint handle;
                fixed (byte* s = text)
                fixed (CoreInterface.File* f = entries)
                fixed (byte* e = error)
                {
                    var p = new CoreInterface.CreateParams
                    {
                        Size = (uint)sizeof(CoreInterface.CreateParams),
                        HostAbiVersion = CoreInterface.Version,
                        Image = image.Data,
                        ImageLen = image.Length,
                        Settings = s,
                        SettingsLen = (nuint)text.Length,
                        Files = f,
                        FileCount = (nuint)entries.Length,
                        FileSize = (nuint)sizeof(CoreInterface.File),
                        PixelFormats = 1,
                        Error = e,
                        ErrorLen = (nuint)error.Length,
                        Flags = (lent ? CoreInterface.CreateLent : 0) | (sharedRemovable ? CoreInterface.CreateSharedRemovable : 0) | (sharedConsole ? CoreInterface.CreateSharedConsole : 0),
                        Device = device is not null && Library.Has(CoreInterface.CapDevicePicture) ? &lend : null,
                    };
                    handle = _api.Create(&p, &status);
                }
                if (handle == 0)
                {
                    int end = Array.IndexOf(error, (byte)0);
                    string words = end > 0 ? Encoding.UTF8.GetString(error, 0, end) : Library.Words(status);
                    throw new CoreRefusedException(status, $"{Library.Info.DisplayName} refused the game: {words}.");
                }
                return handle;
            }
            finally
            {
                foreach (var pin in pins) if (pin.IsAllocated) pin.Free();
            }
        }

        // Where a refused state's exception carries the core's status, so that a host can tell EMUSEN_WRONG_MEDIA from the rest - see EmuSen_CoreAPI.md §6.23.
        public const string StatusKey = "emusen.status";

        public static int? StatusOf(Exception e) => e.Data[StatusKey] as int?;

        public nint Handle => _handle != 0 ? _handle : throw new ObjectDisposedException(nameof(CoreMachine));

        public bool Has(ulong capability) => Library.Has(capability);

        private string Name => Library.Info.DisplayName;

        // The words for a refusal: the machine's last_error, else status_text.
        public string Words(long status)
        {
            string detail;
            lock (_gate) detail = TextOf(_api.LastError);
            return detail.Length > 0 ? detail : Library.Words(status);
        }

        private void Ok(long status, string what)
        {
            if (status < 0) throw new CoreRefusedException((int)status, $"{Name} could not {what}: {Words(status)}.");
        }

        private string TextOf(delegate* unmanaged<nint, byte*, nuint, long> call)
        {
            lock (_gate)
            {
                nint h = Handle;
                long n = call(h, null, 0);
                if (n <= 0) return "";
                var bytes = new byte[n];
                fixed (byte* b = bytes) call(h, b, (nuint)n);
                return CoreLibrary.Utf8(bytes) ?? "";
            }
        }

        // Re-read after create and after a MACHINE_INFO event.
        public CoreMachineInfo ReadInfo()
        {
            lock (_gate) return Info = CoreDescriptorReader.MachineInfo(TextOf(_api.MachineInfo), AudioSampleRate);
        }

        public void Advance()
        {
            lock (_gate)
            {
                ulong detail;
                int status = _api.Advance(Handle, &detail);
                if (status != 0) throw new CoreRefusedException(status, $"{Name} stopped: {Words(status)} (detail {detail:X}).") { Detail = detail };
            }
        }

        public void Present()
        {
            lock (_gate) Ok(_api.Present(Handle), "present the picture");
        }

        public void Reset()
        {
            lock (_gate) Ok(_api.Reset(Handle), "reset");
        }

        public void SetOptions(bool skipRendering)
        {
            lock (_gate) _api.SetOptions(Handle, skipRendering ? 1u : 0u);
        }

        public long TotalFrames
        {
            get { lock (_gate) return _api.FrameCount(Handle); }
        }

        public CoreInterface.FrameInfo FrameInfo
        {
            get
            {
                var info = new CoreInterface.FrameInfo { Size = (uint)sizeof(CoreInterface.FrameInfo) };
                lock (_gate) Ok(_api.FrameInfoOf(Handle, &info), "describe its picture");
                return info;
            }
        }

        // The device lent to this machine, which takes its pictures; null when nothing is lent.
        public EmuSen.Serenity.Slang.SlangLend? Lend { get; }

        private static CoreInterface.DeviceLend LendOf(EmuSen.Serenity.Slang.SlangLend l) => new()
        {
            Size = (uint)sizeof(CoreInterface.DeviceLend), ApiVersion = l.ApiVersion, GetInstanceProcAddr = (nint)l.GetInstanceProcAddr, Instance = l.Instance,
            PhysicalDevice = l.PhysicalDevice, Device = l.Device, QueueFamily = l.QueueFamily, QueueIndex = l.QueueIndex, HostQueueFamily = l.HostQueueFamily, Done = l.Done,
        };

        // After advance and present: the frame's picture taken on the lent device, true when it is there and frame_copy need not be read - see EmuSen_DevicePicture.md §17.
        public bool TakeDeviceFrame()
        {
            if (Lend is not { } lend || FrameDevice() is not { } f) return false;
            lend.Take(f.Buffer, f.Offset, f.Width, f.Height, f.Stride, f.Ready, f.Value);
            return true;
        }

        // The frame's picture where it lies on the lent device, or null when frame_copy has it - see EmuSen_CoreAPI.md §6.29.
        public CoreInterface.DeviceFrame? FrameDevice()
        {
            if (_api.FrameDevice == null || !Has(CoreInterface.CapDevicePicture)) return null;
            var frame = new CoreInterface.DeviceFrame { Size = (uint)sizeof(CoreInterface.DeviceFrame) };
            int status;
            lock (_gate) status = _api.FrameDevice(Handle, &frame);
            if (status < 0) throw new CoreRefusedException(status, $"{Library.Info.DisplayName} could not say where its picture is: {Library.Words(status)}.");
            return status == 1 ? frame : null;
        }

        // RAW_PICTURE: the encodings of machine info's video.raw whose bits are set kept from the next advance - see EmuSen_CoreAPI.md §6.30.
        public void RawSelect(ulong mask)
        {
            if (_api.RawSelect == null || !Has(CoreInterface.CapRawPicture)) return;
            lock (_gate) Ok(_api.RawSelect(Handle, mask), "keep its picture before the palette");
        }

        // The shape of the frame's picture in encoding raw, or null when the frame has none.
        public CoreInterface.RawFrame? RawInfo(uint raw)
        {
            if (_api.RawInfo == null || !Has(CoreInterface.CapRawPicture)) return null;
            var frame = new CoreInterface.RawFrame { Size = (uint)sizeof(CoreInterface.RawFrame) };
            int status;
            lock (_gate) status = _api.RawInfo(Handle, raw, &frame);
            if (status < 0) throw new CoreRefusedException(status, $"{Library.Info.DisplayName} could not describe its picture before the palette: {Library.Words(status)}.");
            return status == 1 ? frame : null;
        }

        public long CopyRaw(uint raw, byte[] into)
        {
            if (_api.RawCopy == null || !Has(CoreInterface.CapRawPicture)) return 0;
            lock (_gate)
                fixed (byte* data = into) return _api.RawCopy(Handle, raw, data, (nuint)into.Length);
        }

        public long CopyFrame(byte[] into)
        {
            lock (_gate)
                fixed (byte* data = into) return _api.FrameCopy(Handle, data, (nuint)into.Length);
        }

        public int AudioSampleRate
        {
            get { lock (_gate) return _api.AudioRate(Handle); }
        }

        public int BufferedSamples
        {
            get { lock (_gate) return (int)Math.Max(0, _api.AudioBuffered(Handle)); }
        }

        // Whole stereo frames, never across a change of rate; the rate they were made at.
        public (short[] Samples, int Rate) DrainAudio(int maxFrames)
        {
            lock (_gate)
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
        }

        public short[] PeekAudio()
        {
            if (!Has(CoreInterface.CapAudioPeek)) return Array.Empty<short>();
            lock (_gate)
            {
                var samples = new short[BufferedSamples];
                long n;
                fixed (short* data = samples) n = _api.AudioPeek(Handle, data, (nuint)samples.Length);
                return n < samples.Length ? samples[..(int)Math.Max(0, n)] : samples;
            }
        }

        public void SetAudioLimit(int samples)
        {
            lock (_gate) _api.SetAudioLimit(Handle, (ulong)Math.Max(0, samples));
        }

        public void SetMutes(uint mask)
        {
            if (!Has(CoreInterface.CapMutes)) return;
            lock (_gate) _api.SetMutes(Handle, mask);
        }

        public void SetButtons(int port, uint mask, uint changed)
        {
            lock (_gate) _api.SetButtons(Handle, (uint)port, mask, changed);
        }

        public void SetAxis(int port, uint axis, double value)
        {
            if (!Has(CoreInterface.CapAxes)) return;
            lock (_gate) _api.SetAxis(Handle, (uint)port, axis, value);
        }

        public int StateSize(uint kind = 0)
        {
            lock (_gate)
            {
                long n = _api.StateSize(Handle, kind);
                Ok(n, "size its state");
                return (int)n;
            }
        }

        public void Save(byte[] into, uint kind = 0)
        {
            lock (_gate)
            {
                long n;
                fixed (byte* data = into) n = _api.StateSave(Handle, kind, data, (nuint)into.Length);
                Ok(n, "write its state");
            }
        }

        public void Load(ReadOnlySpan<byte> state)
        {
            lock (_gate)
            {
                int status;
                fixed (byte* data = state) status = _api.StateLoad(Handle, data, (nuint)state.Length);
                if (status != 0) throw new InvalidDataException($"{Name} refused the state: {Words(status)}.") { Data = { [StatusKey] = status } };
            }
        }

        public string Layout(uint kind = 0)
        {
            lock (_gate)
            {
                long n = _api.StateLayout(Handle, kind, null, 0);
                Ok(n, "list its state");
                var bytes = new byte[n];
                fixed (byte* b = bytes) _api.StateLayout(Handle, kind, b, (nuint)n);
                return Encoding.UTF8.GetString(bytes);
            }
        }

        // MEDIA: a drive's lid opened or closed, as the player would; machine info is read again for the drive's state.
        public void MediaLid(uint drive, bool open)
        {
            lock (_gate)
            {
                Ok(_api.MediaLid(Handle, drive, open ? 1u : 0u), open ? "open the drive's lid" : "close the drive's lid");
                ReadInfo();
            }
        }

        // MEDIA: the disc in a drive replaced by image with its companions, or emptied by null; restore for a state about to load.
        public void MediaInsert(uint drive, byte[]? image, IReadOnlyList<(uint Which, byte[] Data)> files, bool restore)
        {
            fixed (byte* img = image) Insert(drive, image is null ? null : new CoreBytes(img, image.Length), files, Array.Empty<(uint, CoreBytes)>(), restore, null);
        }

        // The same from bytes the host holds in place; with a loan they are lent, and the drive's earlier loan is released once the core has taken the new disc. A refused disc's loan is released at once.
        public void MediaInsert(uint drive, CoreBytes image, IReadOnlyList<(uint Which, CoreBytes Data)> media, bool restore, IDisposable? loan = null)
        {
            try { Insert(drive, image, Array.Empty<(uint, byte[])>(), media, restore, loan); }
            catch
            {
                loan?.Dispose();
                throw;
            }
        }

        private void Insert(uint drive, CoreBytes? image, IReadOnlyList<(uint Which, byte[] Data)> files, IReadOnlyList<(uint Which, CoreBytes Data)> media, bool restore, IDisposable? loan)
        {
            var pins = new GCHandle[files.Count];
            try
            {
                var entries = Entries(files, media, pins);
                fixed (CoreInterface.File* f = entries)
                {
                    var m = new CoreInterface.Media
                    {
                        Size = (uint)sizeof(CoreInterface.Media),
                        Flags = (restore ? CoreInterface.MediaRestore : 0) | (loan is not null ? CoreInterface.MediaLent : 0),
                        Image = image is { } i ? i.Data : null,
                        ImageLen = image?.Length ?? 0,
                        Files = f,
                        FileCount = (nuint)entries.Length,
                        FileSize = (nuint)sizeof(CoreInterface.File),
                    };
                    lock (_gate)
                    {
                        int status = _api.MediaInsert(Handle, drive, &m);
                        Ok(status, image is null ? "empty the drive" : "take the disc");
                        // The core holds nothing of the drive's earlier disc now, so what kept it in place may go.
                        if (_loans.Remove(drive, out IDisposable? earlier)) earlier.Dispose();
                        if (loan is not null) _loans[drive] = loan;
                        ReadInfo();
                    }
                }
            }
            finally
            {
                foreach (var pin in pins) if (pin.IsAllocated) pin.Free();
            }
        }

        public long SpaceSize(uint space)
        {
            lock (_gate) return Math.Max(0, _api.SpaceSize(Handle, space));
        }

        public bool ReadSpace(uint space, int address, Span<byte> into)
        {
            lock (_gate)
                fixed (byte* data = into) return _api.SpaceRead(Handle, space, (uint)address, data, (nuint)into.Length) >= 0;
        }

        public bool WriteSpace(uint space, int address, ReadOnlySpan<byte> from)
        {
            lock (_gate)
                fixed (byte* data = from) return _api.SpaceWrite(Handle, space, (uint)address, data, (nuint)from.Length) >= 0;
        }

        // A battery file's bytes, empty where there is none, with EMUSEN_BATTERY_* flags.
        public (byte[] Data, uint Flags) Battery(uint which)
        {
            lock (_gate)
            {
                uint flags;
                long length = _api.Battery(Handle, which, null, 0, &flags);
                if (length <= 0) return (Array.Empty<byte>(), flags);
                var data = new byte[length];
                fixed (byte* d = data) _api.Battery(Handle, which, d, (nuint)data.Length, &flags);
                return (data, flags);
            }
        }

        public void BatterySaved(uint which)
        {
            lock (_gate) _api.BatterySaved(Handle, which);
        }

        public void SetRomPatches(uint[] triples)
        {
            if (!Has(CoreInterface.CapRomPatches)) return;
            lock (_gate)
                fixed (uint* words = triples) _api.SetRomPatches(Handle, words, (nuint)(triples.Length / 3));
        }

        public void SetCheatPokes(uint[] quads)
        {
            if (!Has(CoreInterface.CapCheatPokes)) return;
            lock (_gate)
                fixed (uint* words = quads) _api.SetCheatPokes(Handle, words, (nuint)(quads.Length / 4));
        }

        // Run-scope "key=value" lines applied together; the core's words on refusal.
        public void SetSettings(string text)
        {
            if (!Has(CoreInterface.CapSettings)) throw new CoreRefusedException(CoreInterface.StatusNotSupported, $"{Name} takes no settings between frames.");
            byte[] bytes = Encoding.UTF8.GetBytes(text);
            lock (_gate)
            {
                int status;
                fixed (byte* b = bytes) status = _api.SetSettings(Handle, b, (nuint)bytes.Length);
                if (status != 0) throw new CoreRefusedException(status, $"{Name} refused the settings: {Words(status)}.");
            }
        }

        public IReadOnlyDictionary<string, string> SettingNotes() =>
            Has(CoreInterface.CapSettingNotes) && TextOf(_api.SettingNotes) is { Length: > 0 } t ? CoreDescriptorReader.Notes(t) : new Dictionary<string, string>();

        public long[] Phases()
        {
            if (!Has(CoreInterface.CapPhases)) return Array.Empty<long>();
            lock (_gate)
            {
                long n = _api.Phases(Handle, null, 0);
                if (n <= 0) return Array.Empty<long>();
                var values = new long[n];
                fixed (long* v = values) _api.Phases(Handle, v, (nuint)n);
                return values;
            }
        }

        public long[] Registers(uint processor)
        {
            if (!Has(CoreInterface.CapDebugRegisters)) return Array.Empty<long>();
            lock (_gate)
            {
                long n = _api.DebugRegisters(Handle, processor, null, 0);
                if (n <= 0) return Array.Empty<long>();
                var values = new long[n];
                fixed (long* v = values) _api.DebugRegisters(Handle, processor, v, (nuint)n);
                return values;
            }
        }

        public string Disassemble(uint processor, uint space, uint address, uint count)
        {
            if (!Has(CoreInterface.CapDebugDisassemble)) return "[]";
            lock (_gate)
            {
                long n = _api.DebugDisassemble(Handle, processor, space, address, count, null, 0);
                if (n <= 0) return "[]";
                var bytes = new byte[n];
                fixed (byte* b = bytes) _api.DebugDisassemble(Handle, processor, space, address, count, b, (nuint)n);
                return CoreLibrary.Utf8(bytes) ?? "[]";
            }
        }

        // Every event waiting, oldest first.
        public IReadOnlyList<CoreInterface.Event> DrainEvents()
        {
            lock (_gate)
            {
                long n = _api.Events(Handle, null, 0, 0);
                if (n <= 0) return Array.Empty<CoreInterface.Event>();
                var events = new CoreInterface.Event[n];
                for (int i = 0; i < events.Length; i++) events[i].Size = (uint)sizeof(CoreInterface.Event);
                fixed (CoreInterface.Event* e = events) n = _api.Events(Handle, e, (nuint)events.Length, (nuint)sizeof(CoreInterface.Event));
                return n == events.Length ? events : events[..(int)Math.Max(0, n)];
            }
        }

        public IReadOnlyList<string> DrainLog()
        {
            lock (_gate) return DrainLog(_api, Handle);
        }

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
            lock (_gate)
            {
                // Nothing of the core's may still be read on the device once it is freed.
                if (_handle != 0) Lend?.Settle();
                if (_handle != 0) _api.Free(_handle);
                _handle = 0;
                // The machine is freed, so every loan has ended.
                foreach (IDisposable loan in _loans.Values) loan.Dispose();
                _loans.Clear();
                Lend?.Dispose();
            }
            GC.SuppressFinalize(this);
        }

        ~CoreMachine()
        {
            if (_handle != 0) _api.Free(_handle);
        }
    }
}
