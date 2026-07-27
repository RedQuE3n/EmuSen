using EmuSen.DianaOS;

namespace EmuSen.Cores.Nintendo.Venus.Debug
{
    // Adapts EmuSen.Debug.DebugSettings' own CpuVerboseLogging/
    // CpuTraceCountdown flags to DianaOS's ICpuTraceSwitch contract, so
    // TraceCommand can use them without EmuSen.DianaOS itself referencing
    // this project (see TraceCommand's own header comment). The flags
    // themselves stay in DebugSettings unchanged - this is purely the
    // plug that lets a host wire them in.
    public sealed class VenusCpuTraceSwitch : ICpuTraceSwitch
    {
        public void Arm(int instructionCount)
        {
            EmuSen.Debug.DebugSettings.CpuTraceCountdown = instructionCount;
            EmuSen.Debug.DebugSettings.CpuVerboseLogging = true;
        }

        public void Disarm()
        {
            EmuSen.Debug.DebugSettings.CpuVerboseLogging = false;
            EmuSen.Debug.DebugSettings.CpuTraceCountdown = 0;
        }
    }
}
