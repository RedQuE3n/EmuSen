using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Rom;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using VideoInterface = EmuSen.Cores.Nintendo.Mars.Vi.Vi;

namespace EmuSen.Cores.Nintendo.Mars
{
    // The Nintendo 64's ICore; what the machine cannot provide yet is stubbed on purpose - see Mars_Core.md.
    public sealed class MarsCore : global::EmuSen.Cores.ICore
    {
        // The VR4300's pipeline clock, which is what MemoryBus.Cycles counts - see Mars_Memory.md §3.
        public const long ProcessorClockHz = 93_750_000;

        // A little over the longest field the VI's registers can describe, so it only ends a frame the VI never would - see Mars_Core.md §3.
        public const long CycleCap = 4_100_000;

        public const int ScreenWidthPixels = VideoInterface.RasterWidth;

        // An NTSC picture, which is what the VI describes before a game programs it - see Mars_Core.md §2.
        public const int DefaultScreenHeight = 480;

        // Where each PadButton lands in the joybus's sixteen button bits - see Mars_Serial.md §3.1.
        private const ushort ButtonA = 0x8000, ButtonB = 0x4000, ButtonZ = 0x2000, ButtonStart = 0x1000;
        private const ushort DpadUp = 0x0800, DpadDown = 0x0400, DpadLeft = 0x0200, DpadRight = 0x0100;
        private const ushort ButtonL = 0x0020, ButtonR = 0x0010;
        private const ushort CUp = 0x0008, CDown = 0x0004, CLeft = 0x0002, CRight = 0x0001;

        // A stick all the way over reads the signed byte's full reach, as Project64's input plugin scales it - see Mars_Core.md §5.
        public const int StickReach = 127;

        // The right stick stands in for the four C buttons, pressed past half its travel - see Mars_Core.md §5.
        public const double CButtonThreshold = 0.5;

        // How often a changed save is written without being asked, as the other cores do - see Mars_Save.md §7.
        public const int SaveEveryNFrames = 300;

        // The Controller Pak's own file beside the cartridge's - see Mars_Save.md §7.
        public const string PakExtension = ".mpk";

        private byte[] _frame = Blank(DefaultScreenHeight);
        private string? _savePath;
        private string? _pakPath;
        private int _screenHeight = DefaultScreenHeight;
        private long _lastFrameCycles = CycleCap;

        // A stock console by default, because the plan defers the Pak as a default - see Mars_Core.md §7.
        public MarsCore(bool expansionPak = false, bool? batteryRamDisabled = null)
        {
            ExpansionPak = expansionPak;
            _batteryRamDisabled = batteryRamDisabled;
        }

        // Null defers to --nobattery; a test passes its own so no other test's switch can reach it - see Mars_Save.md §7.
        private readonly bool? _batteryRamDisabled;

        public bool ExpansionPak { get; }

        public RomImage? Rom { get; private set; }
        public MemoryBus? Bus { get; private set; }
        public Cpu.Core.Cpu? Cpu { get; private set; }

        public string CoreName => "N64";

        public int ScreenWidth => ScreenWidthPixels;

        // Set with the buffer it describes, so the two cannot disagree between frames - see Mars_Core.md §2.
        public int ScreenHeight => _screenHeight;

        // The rate of the frames RunFrame actually produced, whichever boundary ended them - see Mars_Core.md §3.
        public double FrameRateHz => ProcessorClockHz / (double)_lastFrameCycles;

        public bool IsRomLoaded => Bus != null;
        public long TotalFrames { get; private set; }

        public bool SkipRendering { get; set; }

        // The rate the game set the DAC to, which a frontend's audio sink follows when it changes - see Mars_Core.md §4.
        public int AudioSampleRate => Bus?.Ai.SampleRate ?? AiInterface.DefaultSampleRate;

        // The pad's buttons on the generic template, with L2 as Z; the C buttons arrive on the right stick - see Mars_Core.md §5.
        public static IReadOnlyList<PadButton> PadButtons { get; } = new[]
        {
            PadButton.Up, PadButton.Down, PadButton.Left, PadButton.Right,
            PadButton.A, PadButton.B, PadButton.Start, PadButton.L, PadButton.R, PadButton.L2,
        };

        public IReadOnlyList<PadButton> SupportedButtons => PadButtons;

        public static IReadOnlyList<PadAxis> PadAxes { get; } = new[] { PadAxis.LeftX, PadAxis.LeftY, PadAxis.RightX, PadAxis.RightY };

        public IReadOnlyList<PadAxis> SupportedAxes => PadAxes;

        public void LoadRom(string path)
        {
            var rom = RomImage.Load(path);
            var bus = new MemoryBus(ExpansionPak);
            var cpu = new Cpu.Core.Cpu(bus);
            Boot.HandOff(bus, cpu, rom);
            LoadSaves(bus, rom, path);

            Rom = rom;
            Bus = bus;
            Cpu = cpu;

            TotalFrames = 0;
            _lastFrameCycles = CycleCap;
            _screenHeight = DefaultScreenHeight;
            _frame = Blank(DefaultScreenHeight);
        }

        // A binding for a button the pad lacks is dropped rather than moved onto another one - see Mars_Core.md §5.
        public void SetButton(int port, PadButton button, bool pressed)
        {
            Controller[]? ports = Bus?.Si.Controllers;
            if (ports is null || port < 0 || port >= ports.Length) return;

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

            if (mask == 0) return;
            Press(ports[port], mask, pressed);
        }

        // The left stick is the stick, turned over because the N64's up is positive; the right stick is the C buttons - see Mars_Core.md §5.
        public void SetAxis(int port, PadAxis axis, double value)
        {
            Controller[]? ports = Bus?.Si.Controllers;
            if (ports is null || port < 0 || port >= ports.Length) return;

            Controller controller = ports[port];
            value = Math.Clamp(value, -1.0, 1.0);

            switch (axis)
            {
                case PadAxis.LeftX: controller.StickX = (sbyte)Math.Round(value * StickReach); break;
                case PadAxis.LeftY: controller.StickY = (sbyte)Math.Round(-value * StickReach); break;
                case PadAxis.RightX:
                    Press(controller, CLeft, value <= -CButtonThreshold);
                    Press(controller, CRight, value >= CButtonThreshold);
                    break;
                case PadAxis.RightY:
                    Press(controller, CUp, value <= -CButtonThreshold);
                    Press(controller, CDown, value >= CButtonThreshold);
                    break;
            }
        }

        private static void Press(Controller controller, ushort mask, bool pressed) =>
            controller.Buttons = pressed ? (ushort)(controller.Buttons | mask) : (ushort)(controller.Buttons & ~mask);

        public void RunFrame()
        {
            if (Bus is null || Cpu is null)
            {
                throw new InvalidOperationException("RunFrame() called before LoadRom().");
            }

            long start = Bus.Cycles;
            long fields = Bus.Vi.Fields;

            // The VI's field is the frame; the cap only ends one a VI nobody has programmed never will - see Mars_Core.md §3.
            while (Bus.Vi.Fields == fields && Bus.Cycles - start < CycleCap) Cpu.Step();

            _lastFrameCycles = Bus.Cycles - start;
            TotalFrames++;

            if (TotalFrames % SaveEveryNFrames == 0) SaveSram();

            if (!SkipRendering) Present(Bus.Vi);
        }

        public byte[] GetFrameBufferRgba() => _frame;

        public short[] DequeueAudioSamples(int maxFrames) => Bus?.Ai.Drain(maxFrames) ?? Array.Empty<short>();

        // Whatever the game changed since the last write, and nothing it did not - see Mars_Save.md §7.
        public void SaveSram()
        {
            if (Bus is null) return;

            SaveChip chip = Bus.Save;
            if (_savePath != null && chip.Dirty && chip.Contents is { } contents)
            {
                AtomicFile.Write(_savePath, contents);
                chip.Saved();
            }

            ControllerPak? pak = Bus.Si.Controllers[0].Pak;
            if (_pakPath != null && pak is { Dirty: true })
            {
                AtomicFile.Write(_pakPath, pak.Data);
                pak.Dirty = false;
            }
        }

        // The image's word, the table, the last save's length, and otherwise the game's first move - see Mars_Save.md §1.
        private void LoadSaves(MemoryBus bus, RomImage rom, string romPath)
        {
            // Latched here, as a cartridge does, so --nobattery holds for the whole run - see Mars_Save.md §7.
            bool enabled = !(_batteryRamDisabled ?? CoreOptions.BatteryRamDisabled);
            _savePath = enabled ? SaveLibrary.SramPathFor(romPath) : null;
            _pakPath = enabled ? Path.ChangeExtension(_savePath!, PakExtension) : null;

            byte[]? saved = Read(_savePath);
            N64SaveType type = SaveTypes.Declared(rom);
            if (type == N64SaveType.Unknown && saved != null) type = SaveChip.FromSaveLength(saved.Length);

            bus.Save = new SaveChip(type, saved);
            bus.Si.Controllers[0].Pak = new ControllerPak(Read(_pakPath));
        }

        private static byte[]? Read(string? path) => path is null ? null : AtomicFile.TryRead(path);

        // An explicit save refuses before any file exists, so no empty state is left behind - see Mars_Core.md §6.
        public void SaveState(string path) => throw NoSaveStates(nameof(SaveState));

        public void LoadState(string path) => throw NoSaveStates(nameof(LoadState));

        // The rewind buffer calls these every few frames, so they record nothing rather than throw - see Mars_Core.md §6.
        public void SaveState(Stream stream) => RequireRom(nameof(SaveState));

        public void LoadState(Stream stream) => RequireRom(nameof(LoadState));

        // The raster's fourth byte is coverage, not opacity, and a progressive field is every other line - see Mars_Core.md §2.
        private void Present(VideoInterface vi)
        {
            vi.Scan();

            ReadOnlySpan<byte> raster = vi.Frame;
            int rows = vi.FrameHeight;
            int repeat = vi.Serrate ? 1 : 2;
            int rowBytes = ScreenWidthPixels * 4;

            byte[] frame = rows * repeat == _screenHeight ? _frame : new byte[rowBytes * rows * repeat];

            for (int row = 0; row < rows; row++)
            {
                ReadOnlySpan<byte> source = raster.Slice(row * rowBytes, rowBytes);

                for (int copy = 0; copy < repeat; copy++)
                {
                    Span<byte> line = frame.AsSpan((row * repeat + copy) * rowBytes, rowBytes);
                    source.CopyTo(line);
                    for (int alpha = 3; alpha < rowBytes; alpha += 4) line[alpha] = 0xFF;
                }
            }

            _frame = frame;
            _screenHeight = rows * repeat;
        }

        private static byte[] Blank(int height)
        {
            var frame = new byte[ScreenWidthPixels * height * 4];
            for (int alpha = 3; alpha < frame.Length; alpha += 4) frame[alpha] = 0xFF;
            return frame;
        }

        private void RequireRom(string member)
        {
            if (Bus is null) throw new InvalidOperationException($"{member}() called before LoadRom().");
        }

        private NotSupportedException NoSaveStates(string member)
        {
            RequireRom(member);
            return new NotSupportedException("Mars has no save states yet - see Mars_Core.md §6.");
        }
    }
}
