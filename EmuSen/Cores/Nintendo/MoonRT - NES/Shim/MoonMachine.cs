using System;
using System.IO;
using EmuSen.Cores.Native;

namespace EmuSen.Cores.Nintendo.MoonRT
{
    // MoonRT's machine behind its handle, its state read and written in the C# Moon's own format - see Moon_Native.md §3.2.
    public sealed unsafe class MoonMachine : NativeMachine
    {
        private static readonly NativeExports Exports = new(MoonNative.Library, "moon_machine_");
        private static readonly delegate* unmanaged<byte*, nuint, int*, nint> New = (delegate* unmanaged<byte*, nuint, int*, nint>)MoonNative.Export("moon_machine_new");
        private static readonly delegate* unmanaged<nint, int> RunFrameOf = (delegate* unmanaged<nint, int>)MoonNative.Export("moon_machine_run_frame");
        private static readonly delegate* unmanaged<nint, int> ResetOf = (delegate* unmanaged<nint, int>)MoonNative.Export("moon_machine_reset");
        private static readonly delegate* unmanaged<nint, int> StepOf = (delegate* unmanaged<nint, int>)MoonNative.Export("moon_machine_step");
        private static readonly delegate* unmanaged<nint, int, uint, void> SetButtonsOf = (delegate* unmanaged<nint, int, uint, void>)MoonNative.Export("moon_machine_set_buttons");

        public const int FrameBytes = 256 * 240 * 4;

        // MoonCore.Spaces' names, numbered as the C ABI numbers them.
        public static readonly string[] SpaceNames = { "RAM", "PRGROM", "PRGRAM", "CHR", "CIRAM", "OAM", "PALETTE", "CPUBUS" };

        public static bool Available => New != null;

        public static bool Complete => Available && Exports.SetRomPatches != null && ResetOf != null && Exports.SetAudioLimit != null;

        // The iNES image; the board is built from the header as C# builds it.
        public MoonMachine(ReadOnlySpan<byte> image) : base(Exports, "MoonRT", Describe)
        {
            if (!Available) throw new InvalidOperationException($"MoonRT is not in use: {MoonNative.Report}");
            int status;
            nint handle;
            fixed (byte* data = image) handle = New(data, (nuint)image.Length, &status);
            if (handle == 0) throw Refusal(status);
            Attach(handle);
        }

        // The C# exception MoonCore.LoadRom or RunFrame would have thrown for this status - see Moon_Native.md §6.2, D4.
        public static Exception Refusal(int status) => status switch
        {
            -9 => new InvalidDataException("Not an iNES image: missing the \"NES\\x1A\" magic."),
            -10 => new NotSupportedException("The iNES mapper is not implemented - see Moon_Memory.md §4 for what is."),
            -11 => new InvalidDataException("The header claims more PRG than the file holds."),
            -31 => new IndexOutOfRangeException("Index was outside the bounds of the array."),
            -32 => new DivideByZeroException("Attempted to divide by zero."),
            -33 => new ArgumentOutOfRangeException("masterDelta", "A clock cannot run backwards."),
            _ => new InvalidOperationException($"MoonRT refused: {Describe(status)}."),
        };

        public void RunFrame()
        {
            int status = RunFrameOf(Handle);
            if (status != 0) throw Refusal(status);
        }

        public void Reset()
        {
            int status = ResetOf(Handle);
            if (status != 0) throw Refusal(status);
        }

        // One instruction with the frame loop's DMA charge, NMI edge and stolen cycles; the cycles, or a negative status.
        public int Step() => StepOf(Handle);

        // Bit n for NesButton n: A, B, Select, Start, Up, Down, Left, Right.
        public void SetButtons(int port, uint mask) => SetButtonsOf(Handle, port, mask);

        public static string Describe(long status) => Describe(status, "Moon", status => status switch
        {
            -9 => "not an iNES image",
            -10 => "a mapper no board implements",
            -11 => "a header claiming more PRG than the file holds",
            -31 => "an index outside an array",
            -32 => "a division by zero",
            -33 => "a clock running backwards",
            _ => null,
        });
    }
}
