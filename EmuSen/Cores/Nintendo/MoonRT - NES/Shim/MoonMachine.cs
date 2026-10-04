using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores.Native;

namespace EmuSen.Cores.Nintendo.MoonRT
{
    // MoonRT's machine behind the common interface, its state read and written in the C# Moon's own format - see Moon_Native.md §3.2, §8.3.
    public sealed unsafe class MoonMachine : NativeMachine
    {
        private static readonly delegate* unmanaged<nint, int> StepOf = (delegate* unmanaged<nint, int>)MoonNative.Export("moonrt_step");
        private static readonly delegate* unmanaged<nint, uint, uint, int> RomPatchOf = (delegate* unmanaged<nint, uint, uint, int>)MoonNative.Export("moonrt_rom_patch");

        public const int FrameBytes = 256 * 240 * 4;

        // MoonCore.Spaces' names, numbered as the C ABI numbers them.
        public static readonly string[] SpaceNames = { "RAM", "PRGROM", "PRGRAM", "CHR", "CIRAM", "OAM", "PALETTE", "CPUBUS" };

        public static bool Available => MoonNative.Available;

        public static bool Complete => Available && StepOf != null;

        // The iNES image, and the battery save as file 0; the board is built from the header as C# builds it.
        public MoonMachine(ReadOnlySpan<byte> image, byte[]? battery = null)
            : base(MoonNative.Api, "MoonRT", "Moon", Own, OwnWords, image, "", battery is null ? Array.Empty<(uint, byte[])>() : new[] { (0u, battery) })
        {
        }

        // The C# exception MoonCore.LoadRom or RunFrame would have thrown for this status, from the shim's table - see Moon_Native.md §6.2, D4.
        public static Exception Refusal(int status) => Own(status) ?? status switch
        {
            NativeInterface.FaultBase - 1 => new IndexOutOfRangeException("Index was outside the bounds of the array."),
            NativeInterface.FaultBase - 2 => new DivideByZeroException("Attempted to divide by zero."),
            _ => new InvalidOperationException($"MoonRT refused: {Describe(status)}."),
        };

        private static Exception? Own(int status) => MoonRtCore.Own(status);

        private static string? OwnWords(long status) => MoonRtCore.StatusWords(status);

        public static string Describe(long status) => OwnWords(status) ?? Shared(status, "Moon") ?? $"status {status}";

        public void RunFrame() => Advance();

        // One instruction with the frame loop's DMA charge, NMI edge and stolen cycles; the cycles, or a negative status.
        public int Step() => StepOf(Handle);

        // Bit n for NesButton n: A, B, Select, Start, Up, Down, Left, Right; the pad is in no state, so the whole mask is sent.
        public void SetButtons(int port, uint mask) => SetButtons(port, mask, 0xFF);

        // The byte a CPU read of address returns for an original byte under the current patches, or -1 for none.
        public int RomPatch(int address, byte original) => RomPatchOf(Handle, (uint)address, original);
    }
}
