using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia.Native;

namespace EmuSen.WiseMan.Bindings.SaveMedia
{
    // SaveMediaManager's calls into the card manager, the frontend API's cards_op since 2.0, over a runtime with the build's cores; a lone surrogate crosses as U+FFFD - see EmuSen_Backend.md §15.2, EmuSen_RustPlatform.md §47.24.
    public static class SaveMediaNative
    {
        // cards_op's words for a card whose medium no format module reads, where the old answer had no listing.
        private const string NoModule = "no format module reads this medium";

        private static string Args(Action<Utf8JsonWriter> write)
        {
            var stream = new MemoryStream();
            using (var w = new Utf8JsonWriter(stream))
            {
                w.WriteStartObject();
                write(w);
                w.WriteEndObject();
            }
            return System.Text.Encoding.UTF8.GetString(stream.ToArray());
        }

        // A status as the C# threw it: a damaged directory, or a file the system refused.
        private static JsonDocument Call(uint op, Action<Utf8JsonWriter> args)
        {
            try { return JsonDocument.Parse(FrontendRuntime.Cores.CardsOp(op, Args(args))); }
            catch (CrystalException e) when (e.Status == CrystalSessionAbi.Fault && e.Message.StartsWith("ArgumentOutOfRangeException", StringComparison.Ordinal)) { throw new ArgumentOutOfRangeException(); }
            catch (CrystalException e) when (e.Status == CrystalAbi.Io) { throw new IOException(e.Message); }
        }

        private static MediaOutcome Outcome(uint op, Action<Utf8JsonWriter> args)
        {
            using JsonDocument d = Call(op, args);
            return new MediaOutcome(d.RootElement.GetProperty("done").GetBoolean(), d.RootElement.GetProperty("words").GetString()!);
        }

        private static void Text(Utf8JsonWriter w, string name, string value) => w.WriteString(name, PlatformText.Mend(value));

        public static MediumCard? Open(string path)
        {
            JsonDocument d;
            try { d = Call(CrystalAbi.CardsOpen, w => Text(w, "path", path)); }
            catch (CrystalException e) when (e.Status == CrystalSessionAbi.Absent) { return null; }
            using (d)
            {
                JsonElement c = d.RootElement;
                string? problem = c.GetProperty("problem").ValueKind == JsonValueKind.Null ? null : c.GetProperty("problem").GetString();
                MediumListing? listing = problem == NoModule ? null : new MediumListing(c.GetProperty("formatted").GetBoolean(),
                    c.GetProperty("saves").EnumerateArray().Select(s => new SaveEntry(s.GetProperty("id").GetString()!, s.GetProperty("title").GetString()!, s.GetProperty("units").GetInt32(),
                        s.GetProperty("game").ValueKind == JsonValueKind.Null ? null : s.GetProperty("game").GetString(), s.GetProperty("damaged").GetBoolean())).ToArray(),
                    c.GetProperty("free").GetInt32(), problem);
                return new MediumCard(path, SaveMediaManager.MediumOf(path)!, c.GetProperty("exists").GetBoolean(), c.GetProperty("length").GetInt64(), listing);
            }
        }

        public static MediaOutcome CopySave(string from, string save, string to) =>
            Outcome(CrystalAbi.CardsCopy, w => { Text(w, "from", from); Text(w, "save", save); Text(w, "to", to); });

        public static MediaOutcome MoveSave(string from, string save, string to) =>
            Outcome(CrystalAbi.CardsMove, w => { Text(w, "from", from); Text(w, "save", save); Text(w, "to", to); });

        public static MediaOutcome DeleteSave(string card, string save) =>
            Outcome(CrystalAbi.CardsDelete, w => { Text(w, "card", card); Text(w, "save", save); });

        public static MediaOutcome Import(string to, IReadOnlyList<string> from) =>
            Outcome(CrystalAbi.CardsImport, w =>
            {
                Text(w, "to", to);
                w.WriteStartArray("from");
                foreach (string f in from) w.WriteStringValue(PlatformText.Mend(f));
                w.WriteEndArray();
            });

        public static MediaOutcome New(string path) => Outcome(CrystalAbi.CardsNew, w => Text(w, "path", path));

        public static MediaOutcome Duplicate(string from, string to) => Outcome(CrystalAbi.CardsDuplicate, w => { Text(w, "from", from); Text(w, "to", to); });

        public static MediaOutcome Rename(string from, string to) => Outcome(CrystalAbi.CardsRename, w => { Text(w, "from", from); Text(w, "to", to); });
    }
}
