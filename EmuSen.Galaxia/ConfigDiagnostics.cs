using System;

namespace EmuSen.Galaxia
{
    // Where "this config file didn't load, and why" goes - see
    // EmuSen_Config_Reference.md §6.2. Loading stays best-effort and still
    // falls back to defaults; this only stops it happening in silence.
    public static class ConfigDiagnostics
    {
        // Set once by the host. Null discards, which is what tests and any
        // caller that has nowhere to print want.
        public static Action<string>? Sink { get; set; }

        public static string? LastMessage { get; private set; }

        public static void Reset() => LastMessage = null;

        internal static void Report(string message)
        {
            LastMessage = message;
            Sink?.Invoke(message);
        }
    }
}
