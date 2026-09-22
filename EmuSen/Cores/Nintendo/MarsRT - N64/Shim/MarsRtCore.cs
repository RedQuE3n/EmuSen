using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;

namespace EmuSen.Cores.Nintendo.MarsRT
{
    // MarsRT, the N64 core in Rust, behind the interfaces MarsCore implements; the boundary is crossed once a frame - see Mars_Native.md §5.2.
    public sealed unsafe class MarsRtCore : ICore, ISnapshotCore, IStateFormat, IFrameSerial, IRepeatedRows, IDisposable
    {
        private static readonly delegate* unmanaged<byte*, nuint, uint, byte*, nuint, byte*, nuint, nint> LoadRomExport = (delegate* unmanaged<byte*, nuint, uint, byte*, nuint, byte*, nuint, nint>)MarsNative.Export("mars_machine_load_rom");
        private static readonly delegate* unmanaged<byte*, nuint, uint, nint> BootExport = (delegate* unmanaged<byte*, nuint, uint, nint>)MarsNative.Export("mars_machine_boot");
        private static readonly delegate* unmanaged<nint, void> Free = (delegate* unmanaged<nint, void>)MarsNative.Export("mars_machine_free");
        private static readonly delegate* unmanaged<nint, void> RunFrameExport = (delegate* unmanaged<nint, void>)MarsNative.Export("mars_machine_run_frame");
        private static readonly delegate* unmanaged<nint, ulong, void> RunStepsExport = (delegate* unmanaged<nint, ulong, void>)MarsNative.Export("mars_machine_run_steps");
        private static readonly delegate* unmanaged<nint, uint, void> SetOptions = (delegate* unmanaged<nint, uint, void>)MarsNative.Export("mars_machine_set_options");
        private static readonly delegate* unmanaged<nint, uint, uint, uint, void> PressExport = (delegate* unmanaged<nint, uint, uint, uint, void>)MarsNative.Export("mars_machine_press");
        private static readonly delegate* unmanaged<nint, uint, uint, int, void> SetStick = (delegate* unmanaged<nint, uint, uint, int, void>)MarsNative.Export("mars_machine_set_stick");
        private static readonly delegate* unmanaged<nint, ulong> AudioBuffered = (delegate* unmanaged<nint, ulong>)MarsNative.Export("mars_machine_audio_buffered");
        private static readonly delegate* unmanaged<nint, short*, ulong, ulong> DrainAudio = (delegate* unmanaged<nint, short*, ulong, ulong>)MarsNative.Export("mars_machine_drain_audio");
        private static readonly delegate* unmanaged<nint, int> SampleRate = (delegate* unmanaged<nint, int>)MarsNative.Export("mars_machine_audio_sample_rate");
        private static readonly delegate* unmanaged<nint, long*, uint> FrameInfo = (delegate* unmanaged<nint, long*, uint>)MarsNative.Export("mars_machine_frame_info");
        private static readonly delegate* unmanaged<nint, byte*, nuint, ulong> FrameBytes = (delegate* unmanaged<nint, byte*, nuint, ulong>)MarsNative.Export("mars_machine_frame_bytes");
        private static readonly delegate* unmanaged<nint, long*, void> Counters = (delegate* unmanaged<nint, long*, void>)MarsNative.Export("mars_machine_counters");
        private static readonly delegate* unmanaged<nint, byte*, nuint, ulong> IsViewerText = (delegate* unmanaged<nint, byte*, nuint, ulong>)MarsNative.Export("mars_machine_is_viewer_text");
        private static readonly delegate* unmanaged<nint, byte*, nuint, int> RestoreState = (delegate* unmanaged<nint, byte*, nuint, int>)MarsNative.Export("mars_machine_restore_state");
        private static readonly delegate* unmanaged<nint, uint, long> SizeOf = (delegate* unmanaged<nint, uint, long>)MarsNative.Export("mars_machine_save_state_size");
        private static readonly delegate* unmanaged<nint, byte*, nuint, uint, long> SaveStateExport = (delegate* unmanaged<nint, byte*, nuint, uint, long>)MarsNative.Export("mars_machine_save_state");
        private static readonly delegate* unmanaged<nint, byte*, nuint, int*, long> SaveData = (delegate* unmanaged<nint, byte*, nuint, int*, long>)MarsNative.Export("mars_machine_save_data");
        private static readonly delegate* unmanaged<nint, byte*, nuint, int*, long> PakData = (delegate* unmanaged<nint, byte*, nuint, int*, long>)MarsNative.Export("mars_machine_pak_data");
        private static readonly delegate* unmanaged<nint, uint, void> MarkSaved = (delegate* unmanaged<nint, uint, void>)MarsNative.Export("mars_machine_mark_saved");
        private static readonly delegate* unmanaged<nint, byte*, nuint, long> SaveCpu = (delegate* unmanaged<nint, byte*, nuint, long>)MarsNative.Export("mars_machine_save_cpu");
        private static readonly delegate* unmanaged<nint, uint*, ulong, ulong> TakeRegions = (delegate* unmanaged<nint, uint*, ulong, ulong>)MarsNative.Export("mars_machine_take_rdp_regions");

        private const ushort ButtonA = 0x8000, ButtonB = 0x4000, ButtonZ = 0x2000, ButtonStart = 0x1000;
        private const ushort DpadUp = 0x0800, DpadDown = 0x0400, DpadLeft = 0x0200, DpadRight = 0x0100;
        private const ushort ButtonL = 0x0020, ButtonR = 0x0010;
        private const ushort CUp = 0x0008, CDown = 0x0004, CLeft = 0x0002, CRight = 0x0001;

        private nint _handle;
        private string? _romPath, _savePath, _pakPath;
        private readonly bool? _batteryRamDisabled;
        private byte[] _frame = Blank(MarsCore.DefaultScreenHeight);
        private int _screenWidth = MarsCore.ScreenWidthPixels, _screenHeight = MarsCore.DefaultScreenHeight, _rowRepeat = 1;
        private long _frameSerial, _takenSerial;
        private bool _skipRendering, _idleSkip = true, _rspWhole = true, _framesRdp, _repeatRows = true;

        public static bool Available => LoadRomExport != null;

        // A stock console unless asked, as MarsCore; null defers to --nobattery.
        public MarsRtCore(bool expansionPak = false, bool? batteryRamDisabled = null)
        {
            if (!Available) throw new InvalidOperationException($"MarsRT is not in use: {MarsNative.Report}");
            ExpansionPak = expansionPak;
            _batteryRamDisabled = batteryRamDisabled;
        }

        public bool ExpansionPak { get; set; }

        public string CoreName => "N64";
        public int ScreenWidth => _screenWidth;
        public int ScreenHeight => _screenHeight;
        public double FrameRateHz => MarsCore.ProcessorClockHz / (double)(_handle == 0 ? MarsCore.CycleCap : Counter(2));
        public bool IsRomLoaded => _handle != 0;
        public long TotalFrames => _handle == 0 ? 0 : Counter(1);
        public long Cycles => Counter(0);
        public long Instructions => Counter(3);
        public long IdleTurnsPassed => Counter(4);
        public int StateVersion => 1;
        public long FrameSerial => _frameSerial;
        public bool RepeatRows
        {
            get => _repeatRows;
            set { _repeatRows = value; ApplyOptions(); }
        }

        public int RowRepeat => _rowRepeat;
        public IReadOnlyList<PadButton> SupportedButtons => MarsCore.PadButtons;
        public IReadOnlyList<PadAxis> SupportedAxes => MarsCore.PadAxes;
        public int AudioSampleRate => _handle == 0 ? 44100 : SampleRate(_handle);

        public bool SkipRendering
        {
            get => _skipRendering;
            set { _skipRendering = value; ApplyOptions(); }
        }

        // The idle loop passed in one piece, as C#'s Cpu.SkipIdle; exact either way.
        public bool IdleSkip
        {
            get => _idleSkip;
            set { _idleSkip = value; ApplyOptions(); }
        }

        // A running signal processor run to its next event inside the idle loop, as C#'s RunBlocks; exact either way.
        public bool RspWhole
        {
            get => _rspWhole;
            set { _rspWhole = value; ApplyOptions(); }
        }

        // A measurement aid while the display processor is a stub: its words framed, a full sync answered, the RDRAM it would draw recorded.
        public bool FramesRdp
        {
            get => _framesRdp;
            set { _framesRdp = value; ApplyOptions(); }
        }

        private void ApplyOptions()
        {
            if (_handle != 0) SetOptions(_handle, (_skipRendering ? 1u : 0) | (_idleSkip ? 0 : 2u) | (_rspWhole ? 0 : 4u) | (_framesRdp ? 8u : 0) | (_repeatRows ? 16u : 0));
        }

        // The RDRAM ranges the framer saw primitives drawn into since the last call, merged.
        public IReadOnlyList<(uint Start, uint End)> TakeRdpRegions()
        {
            const int Most = 4096;
            uint* pairs = stackalloc uint[2 * Most];
            int n = (int)TakeRegions(Handle, pairs, Most);
            var regions = new List<(uint, uint)>(n);
            for (int i = 0; i < n; i++) regions.Add((pairs[2 * i], pairs[2 * i + 1]));
            return regions;
        }

        public void LoadRom(string path)
        {
            byte[] image = File.ReadAllBytes(path);
            bool enabled = !(_batteryRamDisabled ?? CoreOptions.BatteryRamDisabled);
            _savePath = enabled ? SaveLibrary.SramPathFor(path) : null;
            _pakPath = enabled ? Path.ChangeExtension(_savePath!, MarsCore.PakExtension) : null;
            byte[]? saved = _savePath is null ? null : AtomicFile.TryRead(_savePath);
            byte[]? pak = _pakPath is null ? null : AtomicFile.TryRead(_pakPath);

            nint handle;
            fixed (byte* rom = image)
            fixed (byte* save = saved)
            fixed (byte* card = pak)
                handle = LoadRomExport(rom, (nuint)image.Length, ExpansionPak ? 1u : 0, saved is null ? null : save, (nuint)(saved?.Length ?? 0), pak is null ? null : card, (nuint)(pak?.Length ?? 0));
            if (handle == 0) throw new InvalidDataException($"MarsRT refused {path} as a Nintendo 64 image.");
            Replace(handle);
            _romPath = path;
        }

        // As the corpus boots a machine: the boot code handed off and nothing loaded beside it.
        public void Boot(byte[] image, bool expansionPak = true)
        {
            nint handle;
            fixed (byte* rom = image) handle = BootExport(rom, (nuint)image.Length, expansionPak ? 1u : 0);
            if (handle == 0) throw new InvalidDataException("MarsRT refused the image.");
            Replace(handle);
        }

        private void Replace(nint handle)
        {
            if (_handle != 0) Free(_handle);
            _handle = handle;
            ApplyOptions();
            _frame = Blank(MarsCore.DefaultScreenHeight);
            _screenWidth = MarsCore.ScreenWidthPixels;
            _screenHeight = MarsCore.DefaultScreenHeight;
            _rowRepeat = 1;
            _frameSerial++;
            _takenSerial = 0;
        }

        public void SetButton(int port, PadButton button, bool pressed)
        {
            if (_handle == 0 || port < 0 || port >= 4) return;
            ushort mask = button switch
            {
                PadButton.A => ButtonA,
                PadButton.B => ButtonB,
                PadButton.Start => ButtonStart,
                PadButton.Up => DpadUp,
                PadButton.Down => DpadDown,
                PadButton.Left => DpadLeft,
                PadButton.Right => DpadRight,
                PadButton.L => ButtonL,
                PadButton.R => ButtonR,
                PadButton.L2 => ButtonZ,
                _ => 0,
            };
            if (mask != 0) PressExport(_handle, (uint)port, mask, pressed ? 1u : 0);
        }

        public void SetAxis(int port, PadAxis axis, double value)
        {
            if (_handle == 0 || port < 0 || port >= 4) return;
            value = Math.Clamp(value, -1.0, 1.0);
            switch (axis)
            {
                case PadAxis.LeftX: SetStick(_handle, (uint)port, 0, (sbyte)Math.Round(value * MarsCore.StickReach)); break;
                case PadAxis.LeftY: SetStick(_handle, (uint)port, 1, (sbyte)Math.Round(-value * MarsCore.StickReach)); break;
                case PadAxis.RightX:
                    PressExport(_handle, (uint)port, CLeft, value <= -MarsCore.CButtonThreshold ? 1u : 0);
                    PressExport(_handle, (uint)port, CRight, value >= MarsCore.CButtonThreshold ? 1u : 0);
                    break;
                case PadAxis.RightY:
                    PressExport(_handle, (uint)port, CUp, value <= -MarsCore.CButtonThreshold ? 1u : 0);
                    PressExport(_handle, (uint)port, CDown, value >= MarsCore.CButtonThreshold ? 1u : 0);
                    break;
            }
        }

        public void RunFrame()
        {
            RunFrameExport(Handle);
            if (TotalFrames % MarsCore.SaveEveryNFrames == 0) SaveSram();
            if (!_skipRendering) TakePicture();
        }

        // The interpreter alone, the given number of instructions, as MarsCorpusTests steps the C# core.
        public void RunSteps(ulong steps) => RunStepsExport(Handle, steps);

        public string IsViewerTranscript
        {
            get
            {
                long length = (long)IsViewerText(Handle, null, 0);
                var text = new byte[length];
                fixed (byte* data = text) IsViewerText(Handle, data, (nuint)text.Length);
                return Encoding.UTF8.GetString(text);
            }
        }

        // What the VI's scan composed, as MarsCore's Present composes it after every scan, walked or not.
        private void TakePicture()
        {
            long* info = stackalloc long[4];
            FrameInfo(_handle, info);
            if (info[3] == _takenSerial) return;
            _takenSerial = info[3];
            int width = (int)info[0], height = (int)info[1];
            long length = (long)FrameBytes(_handle, null, 0);
            if (width <= 0 || height <= 0 || length != (long)width * height * 4) return;
            if (_frame.Length != length) _frame = new byte[length];
            fixed (byte* data = _frame) FrameBytes(_handle, data, (nuint)_frame.Length);
            _screenWidth = width;
            _screenHeight = height;
            _rowRepeat = (int)Math.Max(1, info[2]);
            _frameSerial++;
        }

        public byte[] GetFrameBufferRgba() => _frame;

        public short[] DequeueAudioSamples(int maxFrames)
        {
            if (_handle == 0) return Array.Empty<short>();
            long wanted = Math.Min((long)maxFrames * 2, (long)AudioBuffered(_handle));
            wanted -= wanted & 1;
            if (wanted <= 0) return Array.Empty<short>();
            var samples = new short[wanted];
            fixed (short* data = samples) DrainAudio(_handle, data, (ulong)(wanted / 2));
            return samples;
        }

        public void SaveSram()
        {
            if (_handle == 0) return;
            int* info = stackalloc int[2];
            long length = SaveData(_handle, null, 0, info);
            if (_savePath != null && info[1] != 0 && length > 0)
            {
                var contents = new byte[length];
                fixed (byte* data = contents) SaveData(_handle, data, (nuint)contents.Length, info);
                AtomicFile.Write(_savePath, contents);
                MarkSaved(_handle, 1);
            }

            int dirty = 0;
            long pakLength = PakData(_handle, null, 0, &dirty);
            if (_pakPath != null && dirty != 0 && pakLength > 0)
            {
                var pak = new byte[pakLength];
                fixed (byte* data = pak) PakData(_handle, data, (nuint)pak.Length, &dirty);
                AtomicFile.Write(_pakPath, pak);
                MarkSaved(_handle, 2);
            }
        }

        public void SaveState(string path)
        {
            byte[] state = Save(snapshot: false);
            File.WriteAllBytes(path, state);
        }

        public void LoadState(string path) => LoadState(File.ReadAllBytes(path));

        public void SaveState(Stream stream) => stream.Write(Save(snapshot: false));

        public void SaveSnapshot(Stream stream) => stream.Write(Save(snapshot: true));

        public void LoadState(Stream stream)
        {
            using var copy = new MemoryStream();
            stream.CopyTo(copy);
            LoadState(copy.ToArray());
        }

        // The C# Mars's format byte for byte; the machine then derives what MarsCore.LoadState derives.
        public void LoadState(ReadOnlySpan<byte> state)
        {
            int status;
            fixed (byte* data = state) status = RestoreState(Handle, data, (nuint)state.Length);
            if (status != 0) throw new InvalidDataException($"MarsRT refused the state: {MarsMachine.Describe(status)}.");
            if (!_skipRendering) TakePicture();
        }

        public byte[] Save(bool snapshot)
        {
            long size = SizeOf(Handle, snapshot ? 1u : 0);
            if (size < 0) throw new InvalidDataException($"MarsRT could not write the state: {MarsMachine.Describe(size)}.");
            var state = new byte[size];
            long written;
            fixed (byte* data = state) written = SaveStateExport(Handle, data, (nuint)state.Length, snapshot ? 1u : 0);
            if (written < 0) throw new InvalidDataException($"MarsRT could not write the state: {MarsMachine.Describe(written)}.");
            return state;
        }

        // The CPU's fields alone, as StateSerializer writes a Cpu, for a comparison made every few instructions.
        public byte[] SaveProcessor()
        {
            var bytes = new byte[SaveCpu(Handle, null, 0)];
            fixed (byte* data = bytes) SaveCpu(Handle, data, (nuint)bytes.Length);
            return bytes;
        }

        private long Counter(int index)
        {
            long* values = stackalloc long[6];
            Counters(Handle, values);
            return values[index];
        }

        private static byte[] Blank(int height)
        {
            var frame = new byte[MarsCore.ScreenWidthPixels * height * 4];
            for (int alpha = 3; alpha < frame.Length; alpha += 4) frame[alpha] = 0xFF;
            return frame;
        }

        private nint Handle => _handle != 0 ? _handle : throw new InvalidOperationException("MarsRT has no ROM loaded.");

        public void Dispose()
        {
            if (_handle != 0) Free(_handle);
            _handle = 0;
            GC.SuppressFinalize(this);
        }

        ~MarsRtCore()
        {
            if (_handle != 0) Free(_handle);
        }
    }
}
