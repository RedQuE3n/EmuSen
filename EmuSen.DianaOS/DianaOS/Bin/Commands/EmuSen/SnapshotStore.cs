using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Neither command owns saved snapshots more than the other - see §3.10.
    public class SnapshotStore
    {
        public readonly Dictionary<string, (string SpaceName, byte[] Data)> Snapshots = new();
    }
}
