using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Debug;
using EmuSen.Cores.Nintendo.Mercury.Memory;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;

namespace EmuSen.Cores.Nintendo.MercuryRT
{
    // MercuryRT behind the Game Boy's ICore: the machine in Rust, the registries, saves and cheats' rules in C# - see Mercury_Native.md §8.3.
    public sealed class MercuryRtCore : ICore, ICheatRegistryHost, IStateFormat, IFrameBufferPool, IDisposable
    {
        private const int SaveEveryNFrames = 300;

        private static readonly PadButton[] MaskOrder = { PadButton.Right, PadButton.Left, PadButton.Up, PadButton.Down, PadButton.A, PadButton.B, PadButton.Select, PadButton.Start };

        private MercuryMachine? _machine;
        private Cartridge? _header;
        private byte[] _rom = Array.Empty<byte>();
        private readonly byte[] _frame = new byte[MercuryMachine.FrameBytes];
        private readonly FrameBufferLending _lending = new();
        private uint _buttons;
        private bool _skipRendering;
        private int _patchVersion = -1;

        // The battery save's file, the host's and in no state - see Mercury_Native.md §9.3.
        private string? _savePath;

        // The C# machine the debugger reads, refreshed from MercuryRT's state; its registries are this core's - see Mercury_Native.md §8.3.
        public MercuryCore Mirror { get; } = new();

        public static bool Available => MercuryMachine.Complete;

        public string CoreName => _header?.Cgb is null or CgbSupport.None ? "GB" : "GBC";
        public int ScreenWidth => MercuryCore.ScreenWidthPixels;
        public int ScreenHeight => MercuryCore.ScreenHeightPixels;
        public double FrameRateHz => MercuryCore.CpuClockHz / (double)MercuryCore.CyclesPerFrame;
        public bool IsRomLoaded => _machine != null;
        public long TotalFrames => _machine?.TotalFrames ?? 0;
        public int AudioSampleRate => 44100;
        public IReadOnlyList<PadButton> SupportedButtons => MercuryCore.PadButtons;
        int IStateFormat.StateVersion => 6;

        public WatchRegistry Watches => Mirror.Watches;
        public FrameLogRegistry FrameLog => Mirror.FrameLog;
        public BreakpointRegistry Breakpoints => Mirror.Breakpoints;
        public CoverageRegistry Coverage => Mirror.Coverage;
        public LabelRegistry Labels => Mirror.Labels;

        public CheatRegistry Cheats
        {
            get => Mirror.Cheats;
            set
            {
                Mirror.Cheats = value;
                _patchVersion = -1;
            }
        }

        public bool SkipRendering
        {
            get => _skipRendering;
            set
            {
                _skipRendering = value;
                _machine?.SetOptions(value);
            }
        }

        public MercuryMachine Machine => _machine ?? throw new InvalidOperationException("No ROM is loaded.");

        // MercuryCore.LoadRom: the header parsed by the C# Cartridge so its exceptions are C#'s own, the battery save read as C# reads it.
        public void LoadRom(string path)
        {
            byte[] image = File.ReadAllBytes(path);
            Cartridge header = Cartridge.FromImage(image);
            string? savePath = null;
            byte[]? saved = null;
            if (header.HasBattery && !CoreOptions.BatteryRamDisabled && !string.IsNullOrEmpty(path))
            {
                savePath = Path.ChangeExtension(path, ".srm");
                if (header.Ram.Length > 0) saved = AtomicFile.TryRead(savePath);
            }

            var machine = new MercuryMachine(image);
            if (saved is not null) machine.WriteSpace(2, 0, saved.AsSpan(0, Math.Min(saved.Length, header.Ram.Length)));
            machine.SetOptions(_skipRendering);

            _machine?.Dispose();
            _machine = machine;
            _header = header;
            _savePath = savePath;
            _rom = image;
            _patchVersion = -1;
            machine.SetButtons(_buttons);
            Mirror.LoadRom(path);
        }

        public void SetButton(int port, PadButton button, bool pressed)
        {
            if (port != 0) return;
            int bit = Array.IndexOf(MaskOrder, button);
            if (bit < 0) return;
            _buttons = pressed ? _buttons | (1u << bit) : _buttons & ~(1u << bit);
            _machine?.SetButtons(_buttons);
        }

        // MercuryCore.RunFrame and EndFrame; breakpoints and coverage are not yet honoured here (Mercury_Native.md §8.3).
        public void RunFrame()
        {
            if (_machine is null) throw new InvalidOperationException("RunFrame() called before LoadRom().");
            RefreshRomPatches();
            _machine.RunFrame();
            FrameLog.RecordFrame(TotalFrames, ReadForFrameLog);
            ApplyCheats();
            if (TotalFrames % SaveEveryNFrames == 0) SaveSram();
        }

        public void ApplyCheats() => Cheats.ApplyAll(ReadSpace, WriteSpace);

        // A copy in an array no one else holds, as MarsRT's - see EmuSen_Multicore.md §16.
        public byte[] GetFrameBufferRgba()
        {
            _machine?.CopyFrame(_frame);
            byte[] buffer = _lending.Lend(_frame.Length);
            _frame.AsSpan().CopyTo(buffer);
            return buffer;
        }

        public void ReturnFrameBuffer(byte[] buffer) => _lending.Return(buffer);

        public short[] DequeueAudioSamples(int maxFrames) => _machine?.DrainAudio(maxFrames) ?? Array.Empty<short>();

        // Cartridge.SaveSram: the path this session chose at load, which no state can change.
        public void SaveSram()
        {
            if (_machine is null || _header is null || !_header.HasBattery || _header.Ram.Length == 0) return;
            if (_savePath is not { } path) return;
            var ram = new byte[_header.Ram.Length];
            _machine.ReadSpace(2, 0, ram);
            AtomicFile.Write(path, ram);
        }

        public void SaveState(string path)
        {
            using var stream = File.Create(path);
            SaveState(stream);
        }

        public void LoadState(string path)
        {
            using var stream = File.OpenRead(path);
            LoadState(stream);
        }

        public void SaveState(Stream stream)
        {
            if (_machine is null) throw new InvalidOperationException("SaveState() called before LoadRom().");
            stream.Write(_machine.Save());
        }

        // MercuryCore.LoadState's two refusals with its messages; a truncated state is refused whole, where C# stops part-way.
        public void LoadState(Stream stream)
        {
            if (_machine is null) throw new InvalidOperationException("LoadState() called before LoadRom().");
            using var copy = new MemoryStream();
            stream.CopyTo(copy);
            byte[] state = copy.ToArray();
            if (state.Length < 4) throw new EndOfStreamException("Unable to read beyond the end of the stream.");
            if (BitConverter.ToUInt32(state, 0) != 0x4352454D) throw new InvalidDataException("Not a Mercury save state.");
            if (state.Length < 8) throw new EndOfStreamException("Unable to read beyond the end of the stream.");
            int version = BitConverter.ToInt32(state, 4);
            if (version is < 5 or > 6) throw new InvalidDataException($"Save state version {version} is not one this build reads (5 to 6).");
            _machine.Load(state);
        }

        public byte ReadSpace(string spaceName, int address)
        {
            int space = Array.IndexOf(MercuryMachine.SpaceNames, spaceName);
            if (_machine is null || space < 0) return 0;
            Span<byte> one = stackalloc byte[1];
            _machine.ReadSpace(space, address, one);
            return one[0];
        }

        public void WriteSpace(string spaceName, int address, byte value)
        {
            int space = Array.IndexOf(MercuryMachine.SpaceNames, spaceName);
            if (_machine is null || space < 0) return;
            _machine.WriteSpace(space, address, stackalloc byte[] { value });
        }

        public int SpaceSize(string spaceName)
        {
            int space = Array.IndexOf(MercuryMachine.SpaceNames, spaceName);
            return _machine is null || space < 0 ? 0 : _machine.SpaceSize(space);
        }

        private long ReadForFrameLog(string spaceName, int address, int width)
        {
            long value = 0;
            for (int i = 0; i < width; i++) value |= (long)ReadSpace(spaceName, address + i) << (8 * i);
            return value;
        }

        // The debugger's view: the mirror loaded from MercuryRT's state, its reads and writes sent to MercuryRT - see Mercury_Native.md §8.3.
        public MercuryDebugTarget CreateDebugTarget() => new(Mirror, new MercuryDebugHost(
            ReadSpace, WriteSpace, SyncMirror, ApplyCheats, () => TotalFrames, (_, _) => SyncMutes()));

        public void SyncMirror()
        {
            if (_machine is null || Mirror.Bus is null) return;
            Mirror.LoadState(new MemoryStream(_machine.Save()));
        }

        private void SyncMutes()
        {
            if (_machine is null || Mirror.Bus is null) return;
            uint mask = 0;
            for (int i = 0; i < 4; i++) if (Mirror.Bus.Apu.IsChannelMuted(i)) mask |= 1u << i;
            _machine.SetMutes(mask);
        }

        // CheatRegistry.TryPatchRom flattened to a table per patched address, rebuilt when the registry changes - see Mercury_Native.md §8.3.
        private void RefreshRomPatches()
        {
            CheatRegistry cheats = Cheats;
            int version = cheats.Version;
            if (version == _patchVersion || _machine is null) return;
            _patchVersion = version;

            var addresses = new List<ushort>();
            var tables = new List<ushort>();
            if (cheats.EnabledRomPatches > 0)
            {
                var probes = new HashSet<byte> { 0 };
                foreach (var cheat in cheats.GetCheats())
                    if (cheat.Kind == DianaOS.DianaOS.Var.CheatKind.RomPatch && cheat.Compare is byte compare) { probes.Add(compare); probes.Add(unchecked((byte)(compare + 1))); }

                for (int address = 0; address < 0x8000; address++)
                {
                    bool touched = false;
                    foreach (byte probe in probes)
                        if (cheats.TryPatchRom((uint)address, probe, out _)) { touched = true; break; }
                    if (!touched) continue;

                    addresses.Add((ushort)address);
                    for (int value = 0; value < 256; value++)
                        tables.Add(cheats.TryPatchRom((uint)address, (byte)value, out byte patched) ? (ushort)(0x100 | patched) : (ushort)0);
                }
            }
            _machine.SetRomPatches(addresses.ToArray(), tables.ToArray());
        }

        public void Dispose()
        {
            _machine?.Dispose();
            _machine = null;
        }
    }
}
