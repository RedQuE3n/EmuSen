using System.Linq;
using EmuSen.Crystal.Client;
using EmuSen.Serenity.Shaders;

namespace EmuSen.Mistress.Library
{
    // A hosted core's reserved setting console_picture, kept on while the console's CRT filter is; the core's keys stay the host's - see EmuSen_LibretroHost.md §10i.
    public static class ConsolePicture
    {
        public const string Key = "console_picture";

        public static bool Wanted(ScreenFilter? filter) => ReferenceEquals(filter, CrtFilter.Filter);

        // Sets the console's value where the running engine declares the key and it differs; true when it was set.
        public static bool Sync(CrystalSession session, ScreenFilter? filter)
        {
            if (!RuntimeCatalog.SettingsOf(session).Any(r => r.Key == Key)) return false;
            string wanted = Wanted(filter) ? "true" : "false";
            if (session.Setting(Key) == wanted) return false;
            session.SetSetting(Key, wanted, game: false);
            return true;
        }
    }
}
