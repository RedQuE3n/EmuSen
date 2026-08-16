using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
namespace EmuSen.DianaOS.DianaOS.Etc
{
    // One kill switch that leaves every individual flag's value alone - see §2.
    public static class DianaOSLogging
    {
        public static bool MasterEnabled { get; set; }
    }
}
