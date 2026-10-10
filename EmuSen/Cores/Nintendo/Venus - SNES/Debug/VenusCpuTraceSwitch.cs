using EmuSen.DianaSh;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.Cores.Nintendo.Venus.Debug
{
    // Adapts EmuSen.Debug.DebugSettings' own CpuVerboseLogging/ CpuTraceCountdown flags to DianaSh's.
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
