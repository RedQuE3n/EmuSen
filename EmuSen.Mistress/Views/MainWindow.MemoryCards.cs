using System;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.Crystal.Client.Contract;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Views
{
    // The import of a console's per-game cards onto its shared card, offered once before a game starts under the Shared default - see EmuSen_SaveMedia.md §9.
    public partial class MainWindow
    {
        // The question, which a test answers in place of a person.
        internal static Func<MainWindow, string, string, Task<bool>> ConfirmCardImport =
            (owner, title, message) => Dialogs.ConfirmAsync(owner, title, message, "Copy Saves", "Not Now");

        // Everything asked before a game starts: its core's approval, its firmware, then the cards' import.
        private async Task PrepareGameAsync(string romPath)
        {
            await AskCoreApprovalAsync(romPath);
            await PromptForMissingFirmwareAsync(romPath);
            await OfferCardImportAsync(romPath);
        }

        // Asked once per medium and recorded either way, in Mistress's own settings; the games' own cards are copied, never moved, and stay where Per game finds them.
        internal async Task OfferCardImportAsync(string romPath)
        {
            if (RuntimeCatalog.ConsoleForRom(romPath) is not { } console) return;
            CardMedium? medium = CardMedium.Of(Runtime, console).FirstOrDefault(m => m.ImportDue && _graphics.Value(console, OfferedKey(m)) is null);
            if (medium is null) return;
            _graphics.SetValue(console, OfferedKey(medium), "true");
            _graphics.Save();
            string noun = CardMedium.InText(medium.Title), shared = medium.Slots[0].Name;
            string message = $"Your {console} games each kept their own {noun}. Copy their saves onto the shared one, {shared}, so that every game sees them? Each game's own stays as it is: Memory Cards can copy from them later, and Per game in Graphics Settings uses them again.";
            if (!await ConfirmCardImport(this, $"Shared {medium.Title}", message))
            {
                StatusText.Text = $"The {noun}s are shared; each game's own is kept, and Memory Cards can copy from it.";
                return;
            }
            var games = medium.Cards.Where(c => c.GamesOwn).Select(c => c.Path).ToArray();
            CardOutcome outcome = CardOutcome.Of(Runtime, CrystalAbi.CardsImport, new { to = medium.Slots[0].Path, from = games });
            StatusText.Text = outcome.Words.Split('\n')[0];
            if (!outcome.Done) ShowMemoryCards();
        }

        // The key recording that a medium's import was offered, beside its mode's, in graphics.json, which is Mistress's.
        internal static string OfferedKey(CardMedium medium) => medium.ModeKey + "Offered";
    }
}
