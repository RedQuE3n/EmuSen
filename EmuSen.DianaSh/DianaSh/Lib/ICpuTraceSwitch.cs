using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;
namespace EmuSen.DianaSh.DianaSh.Lib
{
    // Injected, because counting instructions only means something to a core with a CPU loop - see §3.3b.
    public interface ICpuTraceSwitch
    {
        void Arm(int instructionCount);
        void Disarm();
    }
}
