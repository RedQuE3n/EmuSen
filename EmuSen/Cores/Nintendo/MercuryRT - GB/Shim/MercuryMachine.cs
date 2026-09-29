using System;
using System.IO;
using EmuSen.Cores.Native;

namespace EmuSen.Cores.Nintendo.MercuryRT
{
    // MercuryRT's machine behind its handle, its state read and written in the C# Mercury's own format - see Mercury_Native.md §3.2.
    public sealed unsafe class MercuryMachine : NativeMachine
    {
        private static readonly NativeExports Exports = new(MercuryNative.Library, "mercury_machine_");
        private static readonly delegate* unmanaged<byte*, nuint, uint, int*, nint> New = (delegate* unmanaged<byte*, nuint, uint, int*, nint>)MercuryNative.Export("mercury_machine_new");
        private static readonly delegate* unmanaged<nint, int> CgbHardwareOf = (delegate* unmanaged<nint, int>)MercuryNative.Export("mercury_machine_cgb_hardware");
        private static readonly delegate* unmanaged<nint, uint*, int> RunFrameOf = (delegate* unmanaged<nint, uint*, int>)MercuryNative.Export("mercury_machine_run_frame");
        private static readonly delegate* unmanaged<nint, int> StepOf = (delegate* unmanaged<nint, int>)MercuryNative.Export("mercury_machine_step");
        private static readonly delegate* unmanaged<nint, uint, void> SetButtonsOf = (delegate* unmanaged<nint, uint, void>)MercuryNative.Export("mercury_machine_set_buttons");
        private static readonly delegate* unmanaged<nint, double, double, void> SetSampleRateOf = (delegate* unmanaged<nint, double, double, void>)MercuryNative.Export("mercury_machine_set_sample_rate");
        private static readonly delegate* unmanaged<nint, byte*, nuint, long> SerialOf = (delegate* unmanaged<nint, byte*, nuint, long>)MercuryNative.Export("mercury_machine_serial");

        public const int FrameBytes = 160 * 144 * 4;

        // MercuryCore.Spaces' names, numbered as the C ABI numbers them.
        public static readonly string[] SpaceNames = { "ROM", "VRAM", "CARTRAM", "WRAM", "OAM", "HRAM", "CPUBUS" };

        public static bool Available => New != null;

        public static bool Complete => Available && Exports.TotalFrames != null && Exports.SetRomPatches != null && Exports.SetAudioLimit != null;

        // A Game Boy Color, whatever its cartridge; a state from the other console changes it - see Mercury_Model.md §5.
        public bool CgbHardware => CgbHardwareOf(Handle) == 1;

        // The ROM image on the console the model chooses; the board is built from the header as C# builds it, and the battery save's path stays the host's.
        public MercuryMachine(ReadOnlySpan<byte> rom, EmuSen.Cores.Nintendo.Mercury.GbModel model = EmuSen.Cores.Nintendo.Mercury.GbModel.Auto) : base(Exports, "MercuryRT", Describe)
        {
            if (!Available) throw new InvalidOperationException($"MercuryRT is not in use: {MercuryNative.Report}");
            int status;
            nint handle;
            fixed (byte* image = rom)
            {
                handle = New(image, (nuint)rom.Length, (uint)model, &status);
            }
            if (handle == 0) throw status == -10 ? new NotSupportedException($"Cartridge type ${rom[0x147]:X2} is not implemented - see Mercury_Memory.md §4.") : new InvalidDataException($"MercuryRT refused the image: {Describe(status)}.");
            Attach(handle);
            SetSampleRate(44100);
        }

        // C#'s Apu.SetSampleRate, the division and Math.Pow both evaluated here - see Mercury_Native.md §3.3 and §9.1.
        public void SetSampleRate(int sampleRate)
        {
            double cyclesPerSample = 4194304.0 / sampleRate;
            SetSampleRateOf(Handle, cyclesPerSample, Math.Pow(EmuSen.Cores.Nintendo.Mercury.Audio.Apu.HighPassSeed, cyclesPerSample));
        }

        // A frame, or the C# core's exception for an opcode no SM83 has.
        public void RunFrame()
        {
            uint detail = 0;
            int status = RunFrameOf(Handle, &detail);
            if (status == -20) throw IllegalOpcode(detail);
            if (status != 0) throw new InvalidOperationException($"MercuryRT could not run: {Describe(status)}.");
        }

        // One instruction with the frame loop's acknowledge and stall; the T-cycles, or -20 for an illegal opcode.
        public int Step() => StepOf(Handle);

        public static NotSupportedException IllegalOpcode(uint detail) =>
            new($"Opcode ${detail >> 16:X2} at ${detail & 0xFFFF:X4} is not a real SM83 instruction - see Mercury_Cpu.md §6.1.");

        // Bit order right, left, up, down, A, B, select, start.
        public void SetButtons(uint mask) => SetButtonsOf(Handle, mask);

        public byte[] SerialLog()
        {
            long n = SerialOf(Handle, null, 0);
            var log = new byte[n];
            fixed (byte* data = log) SerialOf(Handle, data, (nuint)log.Length);
            return log;
        }

        public static string Describe(long status) => Describe(status, "Mercury", status => status switch
        {
            -9 => "an image shorter than the 336-byte header",
            -10 => "a cartridge type no board implements",
            -11 => "a model that is not Auto, Game Boy or Game Boy Color",
            _ => null,
        });
    }
}
