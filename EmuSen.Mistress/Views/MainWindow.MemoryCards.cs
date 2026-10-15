using System;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.Cores;
using EmuSen.Crystal.SaveMedia;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    // The import of a console's per-game cards onto its shared card, offered once before a game starts under the Shared default - see EmuSen_SaveMedia.md §9.
    public partial class MainWindow
    {
        // The question, which a test answers in place of a person.
        internal static Func<MainWindow, string, string, Task<bool>> ConfirmCardImport =
            (owner, title, message) => Dialogs.ConfirmAsync(owner, title, message, "Copy Saves", "Not Now");

        // Everything asked before a game starts: its firmware, then the cards' import.
        private async Task PrepareGameAsync(string romPath)
        {
            await PromptForMissingFirmwareAsync(romPath);
            await OfferCardImportAsync(romPath);
        }

        // Asked once per medium and recorded either way; the games' own cards are copied, never moved, and stay where Per game finds them.
        internal async Task OfferCardImportAsync(string romPath)
        {
            if (CoreCatalog.ConsoleForRom(romPath) is not { } console) return;
            var media = MemoryCards.MediaFor(console);
            if (MemoryCards.ImportToOffer(media, key => _graphics.Value(console, key)) is not { } medium) return;
            _graphics.SetValue(console, MemoryCards.OfferedKey(medium), "true");
            _graphics.Save();
            string noun = medium.Title.ToLowerInvariant(), shared = medium.DefaultName(0);
            string message = $"Your {console} games each kept their own {noun}. Copy their saves onto the shared one, {shared}, so that every game sees them? Each game's own stays as it is: Memory Cards can copy from them later, and Per game in Graphics Settings uses them again.";
            if (!await ConfirmCardImport(this, $"Shared {medium.Title}", message))
            {
                StatusText.Text = $"The {noun}s are shared; each game's own is kept, and Memory Cards can copy from it.";
                return;
            }
            var games = MemoryCards.CardsOf(medium).Where(c => c.GamesOwn).Select(c => c.Path).ToArray();
            MediaOutcome outcome = SaveMediaManager.Import(medium.PathOf(shared), games);
            StatusText.Text = outcome.Words.Split('\n')[0];
            if (!outcome.Done) ShowMemoryCards();
        }
    }
}
