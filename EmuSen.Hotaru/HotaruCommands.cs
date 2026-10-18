using EmuSen.Crystal.Client;
using EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen;
using EmuSen.DianaSh.DianaSh.Lib;

namespace EmuSen.Hotaru
{
    // What a game window lends the commands it adds: the runtime and the session it runs, read at each use since a swap replaces it.
    public interface IHotaruHost
    {
        CrystalRuntime Runtime { get; }
        CrystalSession Session { get; }
    }

    // The commands a game window adds to its shells, each acting through the window's session - see EmuSen_Frontend_Driver.md §1 and EmuSen_Backend.md §14.11.
    public static class HotaruCommands
    {
        public static IDianaShCommand[] For(IHotaruHost host) => new IDianaShCommand[]
        {
            new CoretopCommand(() => EmuSen.Hotaru.Views.DebugWindows.ShowCoretopWindow(host)),
            new StateCommand(path => host.Session.SaveState(path), path => host.Session.LoadState(path), () => host.Session.DefaultStatePath()),
            new CoreCommand(HotaruCores.Of(host.Runtime).Registry, console => HotaruSession.StartsEmpty(host.Runtime, console)),
            new HotaruCommand(null, HotaruSession.GameAlreadyRunning),
        };
    }
}
