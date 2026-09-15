using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.Cores
{
    // A core that owns its cheat registry, so a frontend can hand it the one its windows edit - see EmuSen_Cheats.md §6.
    public interface ICheatRegistryHost
    {
        CheatRegistry Cheats { get; set; }

        void ApplyCheats();
    }
}
