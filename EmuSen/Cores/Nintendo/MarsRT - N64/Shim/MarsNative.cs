using EmuSen.Cores.Native;

namespace EmuSen.Cores.Nintendo.MarsRT
{
    // MarsRT's Rust library on the core ABI v1, loaded once; when it is absent, refused or turned off, this build runs no N64 game - see Mars_Native.md §1, §6.7.
    public static class MarsNative
    {
        public const string Variable = "EMUSEN_MARS_NATIVE";

        // The capabilities and extensions MarsRtCore calls; a library without them serves neither the engine nor the components.
        public static readonly PortLibrary Library = new("marsrt", Variable,
            CoreInterface.CapPresent | CoreInterface.CapPhases | CoreInterface.CapAudioPeek | CoreInterface.CapAxes | CoreInterface.CapSettings | CoreInterface.CapSnapshot
                | CoreInterface.CapFrameSerial | CoreInterface.CapRowRepeat | CoreInterface.CapBatteryDirty | CoreInterface.CapRomPatches | CoreInterface.CapDebug,
            "marsrt_core_counters", "marsrt_core_thread_counters", "marsrt_core_block_counters", "marsrt_core_gpu_report", "marsrt_core_is_viewer_text",
            "marsrt_core_run_steps", "marsrt_core_save_cpu", "marsrt_core_cop0", "marsrt_core_cpu_registers", "marsrt_core_rsp_registers", "marsrt_core_vi_registers",
            "marsrt_core_physical", "marsrt_core_bus_write32", "marsrt_core_bus_read32", "marsrt_core_set_cop0", "marsrt_core_mi_raise", "marsrt_core_rsp_step",
            "marsrt_core_debug_counters", "marsrt_core_state_kind", "marsrt_core_load_fields", "marsrt_core_set_present");

        public static bool Available => Library.Available;

        // Why the library is or is not in use, for a log line or a test's message.
        public static string Report => Library.Report;

        // An export by name, or zero when the library is not in use.
        public static nint Export(string name) => Library.Export(name);
    }
}
