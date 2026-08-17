using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Cores.Nintendo.Venus.Debug
{
    // Adapts EmuSen.Debug.DebugSettings' own CpuVerboseLogging/ CpuTraceCountdown flags to DianaOS's.
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
