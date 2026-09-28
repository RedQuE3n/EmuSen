using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Moon.Debug;
using EmuSen.Cores.Nintendo.Moon.Memory;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;

namespace EmuSen.Cores.Nintendo.MoonRT
{
    // MoonRT behind the NES's ICore: the machine in Rust, the registries, saves and cheats' rules in C# - see Moon_Native.md §2.
    public sealed class MoonRtCore : ICore, IFrameProfiler, ICheatRegistryHost, IStateFormat, IFrameBufferPool, IDisposable
    {
        private const int SaveEveryNFrames = 300;
        private const int StateVersion = 3;

        private MoonMachine? _machine;
        private Cartridge? _header;
        private string? _savePath;
        private readonly byte[] _frame = new byte[MoonMachine.FrameBytes];
        private readonly FrameBufferLending _lending = new();
        private readonly uint[] _buttons = new uint[2];
        private bool _skipRendering;
        private int _patchVersion = -1;
        private double _lastFrameMs;

        // The C# machine the debugger reads, refreshed from MoonRT's state; its registries are this core's - see Moon_Native.md §2.4.
        public MoonCore Mirror { get; } = new();

        public static bool Available => MoonMachine.Complete;

        public string CoreName => "NES";
        public int ScreenWidth => Moon.Video.Ppu.ScreenWidth;
        public int ScreenHeight => Moon.Video.Ppu.ScreenHeight;
        public double FrameRateHz => Mirror.FrameRateHz;
        public bool IsRomLoaded => _machine != null;
        public long TotalFrames => _machine?.TotalFrames ?? 0;
        public int AudioSampleRate => 44100;
        public IReadOnlyList<PadButton> SupportedButtons => MoonCore.PadButtons;
        int IStateFormat.StateVersion => StateVersion;

        // One phase: the frame is one call into Rust, which has no seam to time its parts through - see Moon_Native.md §2.1.
        public IReadOnlyList<(string Name, double Milliseconds)> LastFramePhases => new[] { ("frame", _lastFrameMs) };

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

        public MoonMachine Machine => _machine ?? throw new InvalidOperationException("No ROM is loaded.");

        // MoonCore.LoadRom: the header parsed by the C# Cartridge so its exceptions are C#'s own, the battery save read as C# reads it.
        public void LoadRom(string path)
        {
            byte[] image = File.ReadAllBytes(path);
            Cartridge header = Cartridge.FromImage(image);
            string? savePath = null;
            byte[]? saved = null;
            if (header.HasBattery && !CoreOptions.BatteryRamDisabled && !string.IsNullOrEmpty(path))
            {
                savePath = Path.ChangeExtension(path, SaveLibrary.SramExtension);
                saved = AtomicFile.TryRead(savePath);
            }

            var machine = new MoonMachine(image);
            if (saved is not null) machine.WriteSpace(2, 0, saved.AsSpan(0, Math.Min(saved.Length, header.PrgRam.Length)));
            machine.SetOptions(_skipRendering);

            _machine?.Dispose();
            _machine = machine;
            _header = header;
            _savePath = savePath;
            _patchVersion = -1;
            machine.SetButtons(0, _buttons[0]);
            machine.SetButtons(1, _buttons[1]);
            Mirror.LoadRom(path);
        }

        // The RESET button - see Moon_Core.md §6.
        public void Reset()
        {
            if (_machine is null) throw new InvalidOperationException("Reset() called before LoadRom().");
            _machine.Reset();
        }

        public void SetButton(int port, PadButton button, bool pressed)
        {
            int bit = button switch
            {
                PadButton.A => 0,
                PadButton.B => 1,
                PadButton.Select => 2,
                PadButton.Start => 3,
                PadButton.Up => 4,
                PadButton.Down => 5,
                PadButton.Left => 6,
                PadButton.Right => 7,
                _ => -1,
            };
            if (bit < 0) return;
            int pad = port == 0 ? 0 : 1;
            _buttons[pad] = pressed ? _buttons[pad] | (1u << bit) : _buttons[pad] & ~(1u << bit);
            _machine?.SetButtons(pad, _buttons[pad]);
        }

        // MoonCore.RunFrame and EndFrame's host half, in C#'s order; breakpoints and coverage are stage 5's (Moon_Native.md §4).
        public void RunFrame()
        {
            if (_machine is null) throw new InvalidOperationException("RunFrame() called before LoadRom().");
            RefreshRomPatches();
            long start = Stopwatch.GetTimestamp();
            _machine.RunFrame();
            _lastFrameMs = Stopwatch.GetElapsedTime(start).TotalMilliseconds;
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

        // Cartridge.SaveSram: beside the ROM, and not at all under --nobattery.
        public void SaveSram()
        {
            if (_machine is null || _header is null || !_header.HasBattery || _savePath is null) return;
            var ram = new byte[_header.PrgRam.Length];
            _machine.ReadSpace(2, 0, ram);
            AtomicFile.Write(_savePath, ram);
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

        // MoonCore.LoadState's two refusals with its messages; a truncated state is refused whole, where C# stops part-way.
        public void LoadState(Stream stream)
        {
            if (_machine is null) throw new InvalidOperationException("LoadState() called before LoadRom().");
            using var copy = new MemoryStream();
            stream.CopyTo(copy);
            byte[] state = copy.ToArray();
            if (state.Length < 8) throw new EndOfStreamException("Unable to read beyond the end of the stream.");
            if (BitConverter.ToUInt32(state, 0) != 0x4E4F4F4D) throw new InvalidDataException("Not a Moon save state.");
            int version = BitConverter.ToInt32(state, 4);
            if (version != StateVersion) throw new InvalidDataException($"Save state version {version} is not {StateVersion}.");
            _machine.Load(state);
        }

        public byte ReadSpace(string spaceName, int address)
        {
            int space = Array.IndexOf(MoonMachine.SpaceNames, spaceName);
            if (_machine is null || space < 0) return 0;
            Span<byte> one = stackalloc byte[1];
            _machine.ReadSpace(space, address, one);
            return one[0];
        }

        public void WriteSpace(string spaceName, int address, byte value)
        {
            int space = Array.IndexOf(MoonMachine.SpaceNames, spaceName);
            if (_machine is null || space < 0) return;
            _machine.WriteSpace(space, address, stackalloc byte[] { value });
        }

        public int SpaceSize(string spaceName)
        {
            int space = Array.IndexOf(MoonMachine.SpaceNames, spaceName);
            return _machine is null || space < 0 ? 0 : _machine.SpaceSize(space);
        }

        private long ReadForFrameLog(string spaceName, int address, int width)
        {
            long value = 0;
            for (int i = 0; i < width; i++) value |= (long)ReadSpace(spaceName, address + i) << (8 * i);
            return value;
        }

        // The debugger's view: the mirror loaded from MoonRT's state, its reads and writes sent to MoonRT - see Moon_Native.md §2.4.
        public MoonDebugTarget CreateDebugTarget() => new(Mirror, () => (_lastFrameMs, 0.0), new MoonDebugHost(
            ReadSpace, WriteSpace, SyncMirror, ApplyCheats, () => TotalFrames, (_, _) => SyncMutes()));

        public void SyncMirror()
        {
            if (_machine is null || Mirror.Bus is null) return;
            Mirror.LoadState(new MemoryStream(_machine.Save()));
        }

        private void SyncMutes()
        {
            if (_machine is null || Mirror.Apu is null) return;
            uint mask = 0;
            for (int i = 0; i < 5; i++) if (Mirror.Apu.IsChannelMuted(i)) mask |= 1u << i;
            _machine.SetMutes(mask);
        }

        // CheatRegistry.TryPatchRom flattened to a table per patched CPU address, rebuilt when the registry changes - see Moon_Native.md §2.4.
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
                    if (cheat.Kind == CheatKind.RomPatch && cheat.Compare is byte compare) { probes.Add(compare); probes.Add(unchecked((byte)(compare + 1))); }

                for (int address = 0x4020; address <= 0xFFFF; address++)
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
