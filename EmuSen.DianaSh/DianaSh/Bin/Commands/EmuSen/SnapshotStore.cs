using System.Collections.Generic;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen
{
    // Neither command owns saved snapshots more than the other - see §3.10.
    public class SnapshotStore
    {
        public readonly Dictionary<string, (string SpaceName, byte[] Data)> Snapshots = new();
    }
}
