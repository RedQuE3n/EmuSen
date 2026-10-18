using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.MoonRT;

namespace EmuSen.WiseMan.Fixtures
{
    // MoonRT as the moved tests hold it: the generic engine over its v1 library, or its shim while EMUSEN_MOONRT_ENGINE=shim, until the switch - see EmuSen_Backend.md §21.
    public static unsafe class MoonEngines
    {
        public const string Variable = "EMUSEN_MOONRT_ENGINE";

        public static bool Shim => Environment.GetEnvironmentVariable(Variable) == "shim";

        // The library the build puts beside the tests, opened by its path rather than through discovery.
        public static string LibraryPath => Path.Combine(AppContext.BaseDirectory,
            OperatingSystem.IsWindows() ? "moonrt.dll" : OperatingSystem.IsMacOS() ? "libmoonrt.dylib" : "libmoonrt.so");

        public static CoreLibrary Library => CoreLibrary.Open(LibraryPath);

        public static bool Available => Shim ? MoonRtCore.Available : Library.Available;

        public static string Report => Shim ? MoonNative.Report : Library.Report;

        // The engine a test runs a game on, made with options.
        public static CoreEngine Create(CoreOptions? options = null) =>
            Shim ? new MoonRtCore { Options = options ?? CoreOptions.Default } : new CoreEngine(Library) { Options = options ?? CoreOptions.Default };

        // A bare v1 machine for an image, the battery as file 0 when given.
        public static CoreMachine Machine(byte[] image, byte[]? battery = null) =>
            new(Library, image, "", battery is null ? Array.Empty<(uint, byte[])>() : new List<(uint, byte[])> { (0, battery) });

        // moonrt_core_step: one instruction, its cycles or a negative status.
        public static int Step(CoreMachine machine)
        {
            nint export = Library.Export("moonrt_core_step");
            if (export == 0) throw new NotSupportedException($"{Path.GetFileName(LibraryPath)} has no moonrt_core_step.");
            lock (machine.Gate) return ((delegate* unmanaged<nint, int>)export)(machine.Handle);
        }

        // A state of the machine, whole.
        public static byte[] Save(CoreMachine machine)
        {
            var state = new byte[machine.StateSize()];
            machine.Save(state);
            return state;
        }

        // A field of the engine's state by its layout listing, as the shim's Layout read it.
        public static long Field(CoreEngine core, string path, int index = 0)
        {
            CoreMachine m = core.Machine;
            return StateLayout.FromText("MoonRT", m.Layout()).Int(Save(m), path, index);
        }

        // A refused image in the corpus traces' words: the exception the shim threw for it, the generic engine's refusal read by its status, so the traces hold on both.
        public static string? RefusalWord(Exception e) => e switch
        {
            NotSupportedException or InvalidDataException => e.GetType().Name,
            CoreRefusedException { Status: -10 } => nameof(NotSupportedException),
            CoreRefusedException { Status: -9 or -11 } => nameof(InvalidDataException),
            _ => null,
        };
    }
}
