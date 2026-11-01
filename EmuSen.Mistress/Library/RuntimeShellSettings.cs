using EmuSen.Crystal.Client;
using EmuSen.DianaSh.DianaSh.Bin.Native;

namespace EmuSen.Mistress.Library
{
    // The settings service the shell's command modules reach, over the runtime's own calls and checks: a console's or a game's setting, and a core's approval - see EmuSen_DianaSh.md §9.9.
    internal sealed class RuntimeShellSettings(CrystalRuntime runtime) : IShellSettings
    {
        public const uint Console = 0, Game = 1;

        public string? Get(uint scope, string target, string key) => Call(() => scope == Game ? runtime.GameSetting(target, key) : runtime.Setting(target, key));

        public void Set(uint scope, string target, string key, string? value) => Call(() =>
        {
            if (scope == Game) runtime.SetGameSetting(target, key, value);
            else runtime.SetSetting(target, key, value);
            RuntimeCatalog.Forget();
            return 0;
        });

        public void Approve(string sha256) => Call(() => { runtime.CoreApprove(sha256); return 0; });

        public void Revoke(string sha256) => Call(() => { runtime.CoreRevoke(sha256); return 0; });

        // The binding's refusal as the InvalidOperationException the service turns into its words.
        private static T Call<T>(System.Func<T> call)
        {
            try { return call(); }
            catch (CrystalException e) { throw new System.InvalidOperationException(e.Message, e); }
        }
    }
}
