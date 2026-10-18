using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores;
using EmuSen.Cores.Native;

namespace EmuSen.WiseMan.Fixtures
{
    // MoonRT as its tests hold it: the generic engine over its v1 library, a bare v1 machine, and its two test extensions - see EmuSen_Backend.md §21, §23.
    public static unsafe class MoonEngines
    {
        // The library the build puts beside the tests, opened by its path rather than through discovery.
        public static string LibraryPath => Path.Combine(AppContext.BaseDirectory,
            OperatingSystem.IsWindows() ? "moonrt.dll" : OperatingSystem.IsMacOS() ? "libmoonrt.dylib" : "libmoonrt.so");

        public static CoreLibrary Library => CoreLibrary.Open(LibraryPath);

        public static bool Available => Library.Available;

        public static string Report => Library.Report;

        // The engine a test runs a game on, made with options.
        public static CoreEngine Create(CoreOptions? options = null) => new(Library) { Options = options ?? CoreOptions.Default };

        // A bare v1 machine for an image, the battery as file 0 when given.
        public static CoreMachine Machine(byte[] image, byte[]? battery = null) =>
            new(Library, image, "", battery is null ? Array.Empty<(uint, byte[])>() : new List<(uint, byte[])> { (0, battery) });

        // moonrt_core_step: one instruction, its cycles or a negative status.
        public static int Step(CoreMachine machine) => Call<int>("moonrt_core_step", machine, h => ((delegate* unmanaged<nint, int>)Export("moonrt_core_step"))(h));

        // moonrt_core_rom_patch: the byte a CPU read of address returns for original under the current patches, or -1 for none.
        public static int RomPatch(CoreMachine machine, int address, byte original) =>
            Call<int>("moonrt_core_rom_patch", machine, h => ((delegate* unmanaged<nint, uint, uint, int>)Export("moonrt_core_rom_patch"))(h, (uint)address, original));

        private static nint Export(string name) =>
            Library.Export(name) is var export and not 0 ? export : throw new NotSupportedException($"{Path.GetFileName(LibraryPath)} has no {name}.");

        private static T Call<T>(string name, CoreMachine machine, Func<nint, T> call)
        {
            _ = Export(name);
            lock (machine.Gate) return call(machine.Handle);
        }

        // A state of the machine, whole.
        public static byte[] Save(CoreMachine machine)
        {
            var state = new byte[machine.StateSize()];
            machine.Save(state);
            return state;
        }

        // A field of the engine's state by its layout listing.
        public static long Field(CoreEngine core, string path, int index = 0)
        {
            CoreMachine m = core.Machine;
            return StateLayout.FromText("MoonRT", m.Layout()).Int(Save(m), path, index);
        }
    }
}
