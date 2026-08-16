using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
namespace EmuSen.DianaOS.DianaOS.Lib
{
    // Injected, because counting instructions only means something to a core with a CPU loop - see §3.3b.
    public interface ICpuTraceSwitch
    {
        void Arm(int instructionCount);
        void Disarm();
    }
}
