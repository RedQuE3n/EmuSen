using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.WiseMan.Fixtures
{
    // Bytes of 65816 or SPC700 code at an address, with labels whose branches and addresses are filled in at the end.
    public sealed class CodeBytes(int origin)
    {
        private readonly List<byte> _bytes = new();
        private readonly Dictionary<string, int> _labels = new();
        private readonly List<(int At, string Label, bool Relative)> _fixups = new();

        public int Origin { get; } = origin;
        public int Here => Origin + _bytes.Count;

        public CodeBytes B(params byte[] bytes)
        {
            _bytes.AddRange(bytes);
            return this;
        }

        public CodeBytes Label(string name)
        {
            _labels.Add(name, Here);
            return this;
        }

        // An opcode whose last operand byte is a relative branch to the label.
        public CodeBytes Branch(byte op, string label, params byte[] before)
        {
            _bytes.Add(op);
            _bytes.AddRange(before);
            _fixups.Add((_bytes.Count, label, true));
            _bytes.Add(0);
            return this;
        }

        // An opcode followed by the label's 16-bit address.
        public CodeBytes Address(byte op, string label)
        {
            _bytes.Add(op);
            _fixups.Add((_bytes.Count, label, false));
            _bytes.Add(0);
            _bytes.Add(0);
            return this;
        }

        public int this[string label] => _labels[label];

        public byte[] Bytes
        {
            get
            {
                byte[] bytes = _bytes.ToArray();
                foreach (var (at, label, relative) in _fixups)
                {
                    int target = _labels[label];
                    if (relative)
                    {
                        int delta = target - (Origin + at + 1);
                        if (delta is < -128 or > 127) throw new InvalidOperationException($"branch to {label} is {delta} bytes away");
                        bytes[at] = (byte)(sbyte)delta;
                    }
                    else (bytes[at], bytes[at + 1]) = ((byte)target, (byte)(target >> 8));
                }
                return bytes;
            }
        }
    }

    // VenusRT's synthetic programs for its goldens: the display, its effects, the sound processor, each coprocessor board and random code - see VenusRT_Native.md §68.2.
    public static class VenusPrograms
    {
        // LoROM: file offset n is CPU $00:8000+n; the tables sit at $00:C000 and the vectors' handlers in the code.
        private const int Code = 0x8000, Tables = 0xC000;
        private const int NativeNmi = 0x7FEA, EmulationNmi = 0x7FFA, NativeIrq = 0x7FEE;

        // The WRAM the programs keep: a scroll sum, a frame counter, and results.
        private const byte Scroll = 0x10, Frame = 0x12, Results = 0x20;

        // Native mode, 16-bit index, 8-bit accumulator, the stack at $1FFF, the screen blanked and interrupts off.
        private static CodeBytes Start(int origin = Code) => new CodeBytes(origin)
            .B(0x78, 0x18, 0xFB, 0xC2, 0x10, 0xE2, 0x20, 0xA2, 0xFF, 0x1F, 0x9A)
            .B(0xA9, 0x8F, 0x8D, 0x00, 0x21, 0x9C, 0x00, 0x42);

        private static CodeBytes Store(this CodeBytes c, int register, byte value) => c.B(0xA9, value, 0x8D, (byte)register, (byte)(register >> 8));

        // DMA channel 0 from bank 0 to a B-bus register.
        private static CodeBytes Dma(this CodeBytes c, byte mode, byte bbus, int source, int length) => c
            .Store(0x4300, mode).Store(0x4301, bbus)
            .B(0xA2, (byte)source, (byte)(source >> 8), 0x8E, 0x02, 0x43, 0x9C, 0x04, 0x43)
            .B(0xA2, (byte)length, (byte)(length >> 8), 0x8E, 0x05, 0x43)
            .Store(0x420B, 0x01);

        // Mode 1 with BG1's sixteen 4bpp tiles at VRAM $0000 and its 32x32 map at $0400, a 16-colour palette, BG1 on the main screen.
        private static CodeBytes Screen(this CodeBytes c) => c
            .B(0x9C, 0x21, 0x21).Dma(0x00, 0x22, Tables, 32)
            .Store(0x2115, 0x80).B(0x9C, 0x16, 0x21, 0x9C, 0x17, 0x21).Dma(0x01, 0x18, Tables + 0x20, 512)
            .Store(0x2116, 0x00).Store(0x2117, 0x04).Dma(0x01, 0x18, Tables + 0x220, 2048)
            .Store(0x2105, 0x01).Store(0x2107, 0x04).Store(0x210B, 0x00).Store(0x212C, 0x01);

        // The tables at $00:C000: the palette, the tiles, the map; each byte a function of its place.
        private static void PutTables(byte[] rom, int at)
        {
            for (int i = 0; i < 16; i++)
            {
                int colour = (i * 2) | (31 - i * 2) << 5 | (i * 7 % 32) << 10;
                (rom[at + 2 * i], rom[at + 2 * i + 1]) = ((byte)colour, (byte)(colour >> 8));
            }
            for (int i = 0; i < 512; i++) rom[at + 0x20 + i] = (byte)(i * 37 + (i >> 5) * 11);
            for (int i = 0; i < 1024; i++)
            {
                int entry = (i * 5 + (i >> 5)) & 0x0F | ((i & 0x40) != 0 ? 0x4000 : 0);
                (rom[at + 0x220 + 2 * i], rom[at + 0x221 + 2 * i]) = ((byte)entry, (byte)(entry >> 8));
            }
        }

        // The NMI handler's common part: the pad added to the horizontal scroll, the frame counter the vertical one.
        private static CodeBytes ScrollByPad(this CodeBytes c) => c
            .B(0xC2, 0x20, 0xAD, 0x18, 0x42, 0x18, 0x6D, Scroll, 0x00, 0x8D, Scroll, 0x00, 0xE2, 0x20)
            .B(0x8D, 0x0D, 0x21, 0xEB, 0x8D, 0x0D, 0x21)
            .B(0xEE, Frame, 0x00, 0xAD, Frame, 0x00, 0x8D, 0x0E, 0x21, 0x9C, 0x0E, 0x21);

        // The screen on, NMI and auto-joypad on, and the main loop waiting for interrupts, with the given work before each wait.
        private static CodeBytes Run(this CodeBytes c, Action<CodeBytes>? work = null)
        {
            c.Store(0x2100, 0x0F).Store(0x4200, 0x81).Label("main");
            work?.Invoke(c);
            return c.B(0xCB).Branch(0x80, "main");
        }

        // A LoROM image of 32 KiB from SyntheticRom's builder for the board, with the code at $8000, the tables, and the NMI vector set.
        private static byte[] LoRom(byte[] board, CodeBytes code, int nmi, int? irq = null)
        {
            byte[] rom = (byte[])board.Clone();
            byte[] bytes = code.Bytes;
            Array.Copy(bytes, 0, rom, 0, bytes.Length);
            PutTables(rom, Tables - Code);
            (rom[NativeNmi], rom[NativeNmi + 1]) = ((byte)nmi, (byte)(nmi >> 8));
            (rom[EmulationNmi], rom[EmulationNmi + 1]) = ((byte)nmi, (byte)(nmi >> 8));
            if (irq is int i) (rom[NativeIrq], rom[NativeIrq + 1]) = ((byte)i, (byte)(i >> 8));
            return rom;
        }

        // The display: the pad scrolls BG1, the frame counter scrolls it down.
        public static byte[] Display() => DisplayOn(SyntheticRom.Build());

        private static byte[] DisplayOn(byte[] board)
        {
            var c = Start().Screen().Run();
            c.Label("nmi").ScrollByPad().B(0x40);
            return LoRom(board, c, c["nmi"]);
        }

        // Colour math on the backdrop from an HDMA table of fixed colours, and the multiply and divide units read before they finish.
        public static byte[] Effects()
        {
            var c = Start().Screen()
                .Store(0x2130, 0x00).Store(0x2131, 0x20)
                .Store(0x4310, 0x00).Store(0x4311, 0x32).B(0xA2, 0x00, 0xD0, 0x8E, 0x12, 0x43, 0x9C, 0x14, 0x43)
                .Run(w => w
                    .B(0xAD, Frame, 0x00, 0x8D, 0x02, 0x42, 0xAD, 0x18, 0x42, 0x8D, 0x03, 0x42, 0xEA, 0xEA, 0xEA)
                    .B(0xAE, 0x16, 0x42, 0x8E, Results, 0x00)
                    .B(0xA2, 0x34, 0x12, 0x8E, 0x04, 0x42, 0xAD, Frame, 0x00, 0x09, 0x01, 0x8D, 0x06, 0x42, 0xEA, 0xEA, 0xEA, 0xEA)
                    .B(0xAE, 0x14, 0x42, 0x8E, Results + 2, 0x00, 0xAE, 0x16, 0x42, 0x8E, Results + 4, 0x00));
            c.Label("nmi").ScrollByPad().Store(0x420C, 0x02).B(0x40);
            byte[] rom = LoRom(SyntheticRom.Build(), c, c["nmi"]);
            for (int i = 0; i < 28; i++) (rom[0x5000 + 2 * i], rom[0x5001 + 2 * i]) = (8, (byte)((0x20 << (i % 3)) | (i & 31)));
            rom[0x5000 + 56] = 0;
            return rom;
        }

        // The SPC700's program at $0200: voice 0 plays a looped square from the directory at $0300, its pitch from port 0, keyed again while port 1 is not zero.
        private static byte[] SoundProgram()
        {
            var s = new CodeBytes(0x0200);
            void Dsp(byte reg, byte value) => s.B(0x8F, reg, 0xF2, 0x8F, value, 0xF3);
            Dsp(0x6C, 0x20);
            Dsp(0x0C, 0x7F); Dsp(0x1C, 0x7F); Dsp(0x2C, 0x00); Dsp(0x3C, 0x00);
            Dsp(0x4D, 0x00); Dsp(0x3D, 0x00); Dsp(0x2D, 0x00); Dsp(0x5D, 0x03);
            Dsp(0x00, 0x40); Dsp(0x01, 0x40); Dsp(0x02, 0x00); Dsp(0x03, 0x10);
            Dsp(0x04, 0x00); Dsp(0x05, 0x8F); Dsp(0x06, 0xE0); Dsp(0x5C, 0x00); Dsp(0x4C, 0x01);
            s.Label("loop").B(0xE4, 0xF4, 0x28, 0x1F, 0x08, 0x08, 0x8F, 0x03, 0xF2, 0xC4, 0xF3, 0xE4, 0xF5)
                .Branch(0xF0, "loop").B(0x8F, 0x4C, 0xF2, 0x8F, 0x01, 0xF3).Branch(0x2F, "loop");
            byte[] image = new byte[0x0209];
            byte[] code = s.Bytes;
            Array.Copy(code, image, code.Length);
            (image[0x100], image[0x101], image[0x102], image[0x103]) = (0x00, 0x04, 0x00, 0x04);
            image[0x200] = 0xC3;
            for (int i = 1; i <= 4; i++) (image[0x200 + i], image[0x204 + i]) = (0x77, 0x99);
            return image;
        }

        // The S-CPU uploads the program by the boot program's protocol (fullsnes, "SNES APU Main CPU Communication Port"), starts it, and sends the pad each frame.
        public static byte[] Sound()
        {
            byte[] program = SoundProgram();
            int length = program.Length;
            var c = Start()
                .Label("aa").B(0xAD, 0x40, 0x21, 0xC9, 0xAA).Branch(0xD0, "aa")
                .Label("bb").B(0xAD, 0x41, 0x21, 0xC9, 0xBB).Branch(0xD0, "bb")
                .B(0xA2, 0x00, 0x02, 0x8E, 0x42, 0x21).Store(0x2141, 0x01).Store(0x2140, 0xCC)
                .Label("cc").B(0xCD, 0x40, 0x21).Branch(0xD0, "cc")
                .B(0xA2, 0x00, 0x00)
                .Label("byte").B(0xBD, 0x00, 0xE0, 0x8D, 0x41, 0x21, 0x8A, 0x8D, 0x40, 0x21)
                .Label("echo").B(0xCD, 0x40, 0x21).Branch(0xD0, "echo")
                .B(0xE8, 0xE0, (byte)length, (byte)(length >> 8)).Branch(0xD0, "byte")
                .B(0xA0, 0x00, 0x02, 0x8C, 0x42, 0x21, 0x9C, 0x41, 0x21, 0x8A, 0x1A, 0x8D, 0x40, 0x21)
                .Label("go").B(0xCD, 0x40, 0x21).Branch(0xD0, "go")
                .Screen().Run();
            c.Label("nmi").ScrollByPad().B(0xAD, 0x18, 0x42, 0x8D, 0x40, 0x21, 0xAD, 0x19, 0x42, 0x8D, 0x41, 0x21, 0x40);
            byte[] rom = LoRom(SyntheticRom.Build(), c, c["nmi"]);
            Array.Copy(program, 0, rom, 0x6000, length);
            return rom;
        }

        // fullsnes: an SA-1 cartridge's chipset is 32h-35h, on map mode 23h.
        private static byte[] Sa1Board() => SyntheticRom.BuildSa1((0x7FD6, new byte[] { 0x35 }));

        // SA-1: the S-CPU releases the SA-1 at $A000, which counts in I-RAM and writes BW-RAM; the count scrolls BG1.
        public static byte[] Sa1()
        {
            var c = Start().Screen().Store(0x2226, 0x80).Store(0x2229, 0xFF)
                .B(0xA2, 0x00, 0xA0, 0x8E, 0x03, 0x22).Store(0x2200, 0x00).Run();
            c.Label("nmi").ScrollByPad().B(0xAD, 0x00, 0x30, 0x8D, 0x0D, 0x21, 0xAD, 0x01, 0x30, 0x8D, 0x0D, 0x21, 0x40);
            byte[] rom = LoRom(Sa1Board(), c, c["nmi"]);
            var sa1 = new CodeBytes(0xA000).B(0x18, 0xFB, 0xE2, 0x20).Store(0x2227, 0x80).Store(0x222A, 0xFF).B(0xC2, 0x20)
                .Label("count").B(0xEE, 0x00, 0x30, 0xAD, 0x00, 0x30, 0x8F, 0x00, 0x00, 0x40).Branch(0x80, "count");
            byte[] code = sa1.Bytes;
            Array.Copy(code, 0, rom, 0x2000, code.Length);
            return rom;
        }

        // The GSU plots a line in the frame counter's colour from ROM at $9000 while the S-CPU waits in WRAM, polls GO, then waits a frame.
        public static byte[] SuperFx()
        {
            var wram = new CodeBytes(0x0200).Label("again")
                .B(0xAD, Frame, 0x00, 0x8D, 0x00, 0x30, 0x9C, 0x01, 0x30).Store(0x301E, 0x00).Store(0x301F, 0x90)
                .Label("go").B(0xAD, 0x30, 0x30, 0x29, 0x20).Branch(0xD0, "go")
                .Label("vb").B(0xAD, 0x12, 0x42).Branch(0x10, "vb")
                .Label("ve").B(0xAD, 0x12, 0x42).Branch(0x30, "ve")
                .B(0xEE, Frame, 0x00).Branch(0x80, "again");
            byte[] loop = wram.Bytes;
            var c = Start().Screen().Store(0x2100, 0x0F).Store(0x4200, 0x01)
                .Store(0x3034, 0x00).Store(0x3038, 0x00).Store(0x303A, 0x18).Store(0x3037, 0x80).Store(0x3039, 0x00)
                .B(0xC2, 0x30, 0xA2, 0x00, 0xB0, 0xA0, 0x00, 0x02, 0xA9, (byte)(loop.Length - 1), 0x00, 0x54, 0x00, 0x00, 0xE2, 0x20)
                .B(0x5C, 0x00, 0x02, 0x00);
            byte[] rom = LoRom(SyntheticRom.BuildSuperFx(), c, 0x8000);
            Array.Copy(loop, 0, rom, 0x3000, loop.Length);
            var gsu = new CodeBytes(0x9000).B(0x4E, 0xFC, 0x00, 0x02).Address(0xFD, "plot").B(0xA1, 0x00, 0xA2, 0x10)
                .Label("plot").B(0x4C, 0x3C, 0x01, 0x00, 0x01);
            byte[] code = gsu.Bytes;
            Array.Copy(code, 0, rom, 0x1000, code.Length);
            return rom;
        }

        // A DSP-1 (Pilotwings' title) with no firmware, so the open replacement answers: each frame a multiply of the counter by the pad, read back to WRAM.
        public static byte[] NecDsp()
        {
            var c = Start().Screen().Run();
            c.Label("nmi").ScrollByPad()
                .B(0xA9, 0x00).Address(0x20, "send").B(0xAD, Frame, 0x00).Address(0x20, "send").B(0xA9, 0x00).Address(0x20, "send")
                .B(0xAD, 0x18, 0x42).Address(0x20, "send").B(0xA9, 0x40).Address(0x20, "send")
                .Address(0x20, "receive").B(0x8D, Results, 0x00).Address(0x20, "receive").B(0x8D, Results + 1, 0x00).B(0x40)
                .Label("send").B(0x48).Label("rqm").B(0xAF, 0x00, 0xC0, 0x30).Branch(0x10, "rqm").B(0x68, 0x8F, 0x00, 0x80, 0x30, 0x60)
                .Label("receive").Label("ready").B(0xAF, 0x00, 0xC0, 0x30).Branch(0x10, "ready").B(0xAF, 0x00, 0x80, 0x30, 0x60);
            return LoRom(SyntheticRom.BuildNecDsp("PILOTWINGS"), c, c["nmi"]);
        }

        // The OBC1 given an object each frame through its registers at $7FF0-$7FF6.
        public static byte[] Obc1()
        {
            var c = Start().Screen().Run();
            c.Label("nmi").ScrollByPad()
                .B(0xAD, Frame, 0x00, 0x29, 0x7F, 0x8D, 0xF6, 0x7F, 0x8D, 0xF0, 0x7F, 0x49, 0xFF, 0x8D, 0xF1, 0x7F)
                .B(0xAD, 0x18, 0x42, 0x8D, 0xF2, 0x7F, 0x8D, 0xF3, 0x7F, 0x8D, 0xF4, 0x7F, 0x40);
            return LoRom(SyntheticRom.BuildObc1(), c, c["nmi"]);
        }

        // HiROM, 64 KiB: the display's code at file $8000, which is $00:8000, under a header at $FFC0 that scores as HiROM.
        public static byte[] HiRom()
        {
            byte[] lo = Display();
            byte[] rom = new byte[0x10000];
            Array.Copy(lo, 0, rom, 0x8000, 0x8000);
            Array.Clear(rom, 0xFFC0, 0x20);
            byte[] title = System.Text.Encoding.ASCII.GetBytes("VENUSRT GOLDEN HIROM ");
            Array.Copy(title, 0, rom, 0xFFC0, 21);
            (rom[0xFFD5], rom[0xFFD7], rom[0xFFD8]) = (0x21, 0x09, 0x00);
            (rom[0xFFDC], rom[0xFFDD], rom[0xFFDE], rom[0xFFDF]) = (0xCB, 0xED, 0x34, 0x12);
            return rom;
        }

        // The boards a random program runs on, by name, each a builder of an image with the board's header.
        public static readonly (string Name, Func<byte[]> Board)[] Boards =
        {
            ("lorom", () => SyntheticRom.Build()),
            ("hirom", HiRom),
            ("sa1", Sa1Board),
            ("gsu", () => SyntheticRom.BuildSuperFx()),
            ("dsp1", () => SyntheticRom.BuildNecDsp("PILOTWINGS")),
            ("obc1", SyntheticRom.BuildObc1),
        };

        // Random bytes as code from $8005 to the header, the board's header and vectors kept; seeds 4-7 make STP and WAI NOPs so the program runs on.
        public static byte[] Random(string board, int seed)
        {
            byte[] rom = Boards.Single(b => b.Name == board).Board();
            int start = rom.Length == 0x10000 ? 0x8005 : 0x0005, end = rom.Length == 0x10000 ? 0xFFB0 : 0x7FB0;
            var noise = new System.Random(seed * 131 + Array.FindIndex(Boards, b => b.Name == board) * 7);
            noise.NextBytes(rom.AsSpan(start, end - start));
            if (seed >= 4)
                for (int i = start; i < end; i++)
                    if (rom[i] is 0xDB or 0xCB) rom[i] = 0xEA;
            return rom;
        }
    }
}
