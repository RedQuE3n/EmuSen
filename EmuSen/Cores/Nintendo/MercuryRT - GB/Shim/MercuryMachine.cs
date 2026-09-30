using System;
using System.IO;
using EmuSen.Cores.Native;

namespace EmuSen.Cores.Nintendo.MercuryRT
{
    // MercuryRT's machine behind the common interface, its state read and written in the C# Mercury's own format - see Mercury_Native.md §3.2, §8.7.
    public sealed unsafe class MercuryMachine : NativeMachine
    {
        private static readonly delegate* unmanaged<nint, int> CgbHardwareOf = (delegate* unmanaged<nint, int>)MercuryNative.Export("mercuryrt_cgb_hardware");
        private static readonly delegate* unmanaged<nint, int> StepOf = (delegate* unmanaged<nint, int>)MercuryNative.Export("mercuryrt_step");
        private static readonly delegate* unmanaged<nint, double, double, int> SetSampleRateOf = (delegate* unmanaged<nint, double, double, int>)MercuryNative.Export("mercuryrt_set_sample_rate");
        private static readonly delegate* unmanaged<nint, byte*, nuint, long> SerialOf = (delegate* unmanaged<nint, byte*, nuint, long>)MercuryNative.Export("mercuryrt_serial");
        private static readonly delegate* unmanaged<nint, uint, uint, int> RomPatchOf = (delegate* unmanaged<nint, uint, uint, int>)MercuryNative.Export("mercuryrt_rom_patch");
        private static readonly delegate* unmanaged<nint, long> RamLengthOf = (delegate* unmanaged<nint, long>)MercuryNative.Export("mercuryrt_ram_length");

        public const int FrameBytes = 160 * 144 * 4;

        // MercuryCore.Spaces' names, numbered as the C ABI numbers them.
        public static readonly string[] SpaceNames = { "ROM", "VRAM", "CARTRAM", "WRAM", "OAM", "HRAM", "CPUBUS" };

        public static bool Available => MercuryNative.Available;

        public static bool Complete => Available && StepOf != null && CgbHardwareOf != null && SetSampleRateOf != null && SerialOf != null;

        // A Game Boy Color, whatever its cartridge; a state from the other console changes it - see Mercury_Model.md §5.
        public bool CgbHardware => CgbHardwareOf(Handle) == 1;

        // The ROM image on the console the model chooses, sent as the create-time setting Model, and the battery save as file 0.
        public MercuryMachine(ReadOnlySpan<byte> rom, EmuSen.Cores.Nintendo.Mercury.GbModel model = EmuSen.Cores.Nintendo.Mercury.GbModel.Auto, byte[]? battery = null)
            : base(MercuryNative.Api, "MercuryRT", "Mercury", Refuser(rom.Length > 0x147 ? rom[0x147] : (byte)0), OwnWords, rom,
                $"Model={(int)model}", battery is null ? Array.Empty<(uint, byte[])>() : new[] { (0u, battery) })
        {
            SetSampleRate(44100);
        }

        // MercuryCore.LoadRom's exceptions for the core's refusals, with the cartridge type the header named.
        private static Func<int, Exception?> Refuser(byte cartridgeType) => status => status switch
        {
            -10 => new NotSupportedException($"Cartridge type ${cartridgeType:X2} is not implemented - see Mercury_Memory.md §4."),
            -9 or -11 => new InvalidDataException($"MercuryRT refused the image: {Describe(status)}."),
            _ => null,
        };

        private static string? OwnWords(long status) => status switch
        {
            -9 => "an image shorter than the 336-byte header",
            -10 => "a cartridge type no board implements",
            -11 => "a model that is not Auto, Game Boy or Game Boy Color",
            -20 => "an opcode no SM83 has",
            _ => null,
        };

        public static string Describe(long status) => OwnWords(status) ?? Shared(status, "Mercury") ?? $"status {status}";

        // An illegal opcode is built from the frame's detail word, (opcode << 16) | pc.
        protected override Exception? FrameException(int status, ulong detail) => status == -20 ? IllegalOpcode((uint)detail) : null;

        // C#'s Apu.SetSampleRate, the division and Math.Pow both evaluated here - see Mercury_Native.md §3.3 and §9.1.
        public void SetSampleRate(int sampleRate)
        {
            double cyclesPerSample = 4194304.0 / sampleRate;
            SetSampleRateOf(Handle, cyclesPerSample, Math.Pow(EmuSen.Cores.Nintendo.Mercury.Audio.Apu.HighPassSeed, cyclesPerSample));
        }

        public void RunFrame() => Advance();

        // One instruction with the frame loop's acknowledge and stall; the T-cycles, or -20 for an illegal opcode.
        public int Step() => StepOf(Handle);

        public static NotSupportedException IllegalOpcode(uint detail) =>
            new($"Opcode ${detail >> 16:X2} at ${detail & 0xFFFF:X4} is not a real SM83 instruction - see Mercury_Cpu.md §6.1.");

        // Bit order right, left, up, down, A, B, select, start; the joypad is in no state, so the whole mask is sent.
        public void SetButtons(uint mask) => SetButtons(0, mask, 0xFF);

        public byte[] SerialLog()
        {
            long n = SerialOf(Handle, null, 0);
            var log = new byte[n];
            fixed (byte* data = log) SerialOf(Handle, data, (nuint)log.Length);
            return log;
        }

        // The byte a CPU read of address returns for an original byte under the current patches, or -1 for none.
        public int RomPatch(int address, byte original) => RomPatchOf(Handle, (uint)address, original);

        // The cartridge RAM's length, battery or not.
        public long RamLength => RamLengthOf(Handle);
    }
}
