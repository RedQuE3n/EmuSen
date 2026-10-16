using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.SaveMedia;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Galaxia.Library;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen
{
    // The card manager for any console's save media, over Crystal's one implementation; it needs no running game - see `man memcard`.
    public class MemcardCommand : IDianaShCommand
    {
        public string Name => "memcard";
        public bool IsReadOnly => false;
        public string Usage =>
            "  memcard ls [<card>]           every card under Saves, or the saves on one card\n" +
            "  memcard cp <card>:<save> <card>\n" +
            "                                copy a save onto another card of its medium\n" +
            "  memcard mv <card>:<save> <card>\n" +
            "                                copy a save, read it back, then delete the first\n" +
            "  memcard rm <card>:<save>      delete a save as the console's own manager does\n" +
            "  memcard import <card> <from>...\n" +
            "                                every save of <from> onto <card>, all or none\n" +
            "  memcard new <card>            a new card, formatted where its medium allows\n" +
            "  memcard dup <card> <new>      a whole card copied to a new name\n" +
            "  memcard rename <card> <new>   a card renamed";

        private const string Use = "Usage: memcard ls|cp|mv|rm|import|new|dup|rename ... (see 'man memcard')";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 2) return DianaShResult.Fail(Use);
            try
            {
                return args[1] switch
                {
                    "ls" when args.Length == 2 => ListAll(),
                    "ls" when args.Length == 3 => Card(args[2], out string path) is { } bad ? bad : ListOne(path),
                    "cp" or "mv" when args.Length == 4 => Transfer(args[1], args[2], args[3]),
                    "rm" when args.Length == 3 => Remove(args[2]),
                    "import" when args.Length >= 4 => Import(args[2], args.Skip(3).ToArray()),
                    "new" when args.Length == 3 => Card(args[2], out string path) is { } bad ? bad : Done(SaveMediaManager.New(path)),
                    "dup" or "rename" when args.Length == 4 => Pair(args[1], args[2], args[3]),
                    _ => DianaShResult.Fail(Use),
                };
            }
            catch (IOException e)
            {
                return DianaShResult.Fail($"memcard: {e.Message}");
            }
        }

        private static DianaShResult Done(MediaOutcome outcome) => outcome.Done ? DianaShResult.Ok(outcome.Words) : DianaShResult.Fail($"memcard: {outcome.Words}");

        // A card path in the sandbox, of a medium a system pack names; the failure to print otherwise.
        private static DianaShResult? Card(string given, out string path)
        {
            path = "";
            if (!DianaShSandbox.TryResolve(given, out string resolved)) return DianaShResult.Fail($"memcard: '{given}' is outside the project sandbox ({DianaShSandbox.RootDirectory})");
            if (SaveMediaManager.MediumOf(resolved) is null)
                return DianaShResult.Fail($"memcard: {Path.GetFileName(given)} is no card: the media known are {string.Join(", ", SaveMediumFormats.All.Select(m => $"{m.Extension} ({m.Title})"))}");
            path = resolved;
            return null;
        }

        // <card>:<save>, split at the last colon, as a save's id holds none and a path rarely does.
        private static DianaShResult? CardAndSave(string given, out string path, out string save)
        {
            path = save = "";
            int colon = given.LastIndexOf(':');
            if (colon <= 0 || colon == given.Length - 1) return DianaShResult.Fail($"memcard: '{given}' names no save; write <card>:<save>, the save by its name or its number in 'memcard ls'");
            save = given[(colon + 1)..];
            return Card(given[..colon], out path);
        }

        private static string Shown(string path) => Path.GetRelativePath(DianaShSandbox.RootOf(path), path).Replace('\\', '/');

        private static DianaShResult ListAll()
        {
            var lines = new List<string>();
            string saves = DataStore.Saves;
            if (Directory.Exists(saves))
                foreach (string console in Directory.EnumerateDirectories(saves).Order(StringComparer.OrdinalIgnoreCase))
                {
                    var cards = new[] { SaveLibrary.CardsFolder, SaveLibrary.ConsoleFolder }.Select(f => Path.Combine(console, f)).Where(Directory.Exists)
                        .SelectMany(f => Directory.EnumerateFiles(f)).Concat(Directory.EnumerateFiles(console))
                        .Where(f => SaveMediaManager.MediumOf(f) is not null).Order(StringComparer.OrdinalIgnoreCase).ToList();
                    if (cards.Count == 0) continue;
                    lines.Add(Path.GetFileName(console));
                    foreach (string card in cards) lines.Add("  " + Summary(card));
                }
            return lines.Count == 0 ? DianaShResult.Ok("No cards: nothing under Saves is a file of a medium a system pack names.") : DianaShResult.Ok(string.Join('\n', lines));
        }

        // One line for a card: its path, its medium, its use, and who holds it.
        private static string Summary(string path)
        {
            MediumCard card = SaveMediaManager.Open(path)!;
            string use = card.Listing is not { } l ? $"{card.Length} bytes, no format module" : !l.Formatted ? l.Problem ?? "not formatted" : $"{l.Saves.Count} save(s), {l.FreeUnits} of {card.Medium.Format!.Capacity} {card.Medium.Format.Unit}s free";
            string held = SaveMediaManager.HeldBy(path) is { } holder ? $"  [in use by {holder}]" : "";
            return $"{Shown(path)}  {card.Medium.Title}  {use}{held}";
        }

        private static DianaShResult ListOne(string path)
        {
            MediumCard card = SaveMediaManager.Open(path)!;
            var lines = new List<string> { Summary(path) };
            if (card.Listing is { Formatted: true } listing)
                for (int n = 0; n < listing.Saves.Count; n++)
                {
                    SaveEntry s = listing.Saves[n];
                    lines.Add($"  {n + 1,2}  {s.Units,2} {card.Medium.Format!.Unit}(s)  {s.Id}  {s.Title}{(s.Damaged ? "  [damaged]" : "")}");
                }
            return DianaShResult.Ok(string.Join('\n', lines));
        }

        private static DianaShResult Transfer(string verb, string from, string to)
        {
            if (CardAndSave(from, out string source, out string save) is { } bad) return bad;
            if (Card(to, out string target) is { } badTarget) return badTarget;
            return Done(verb == "cp" ? SaveMediaManager.CopySave(source, save, target) : SaveMediaManager.MoveSave(source, save, target));
        }

        private static DianaShResult Remove(string given) =>
            CardAndSave(given, out string path, out string save) is { } bad ? bad : Done(SaveMediaManager.DeleteSave(path, save));

        private static DianaShResult Import(string to, string[] from)
        {
            if (Card(to, out string target) is { } bad) return bad;
            var sources = new List<string>();
            foreach (string given in from)
            {
                if (Card(given, out string source) is { } badSource) return badSource;
                sources.Add(source);
            }
            return Done(SaveMediaManager.Import(target, sources));
        }

        private static DianaShResult Pair(string verb, string from, string to)
        {
            if (Card(from, out string source) is { } bad) return bad;
            if (Card(to, out string target) is { } badTarget) return badTarget;
            return Done(verb == "dup" ? SaveMediaManager.Duplicate(source, target) : SaveMediaManager.Rename(source, target));
        }
    }
}
