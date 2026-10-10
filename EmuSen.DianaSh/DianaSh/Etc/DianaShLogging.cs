using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;
namespace EmuSen.DianaSh.DianaSh.Etc
{
    // One kill switch that leaves every individual flag's value alone - see §2.
    public static class DianaShLogging
    {
        public static bool MasterEnabled { get; set; }
    }
}
