using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Services;
using EmuSen.Crystal.Systems;
using EmuSen.Galaxia.Native;

namespace EmuSen.Crystal.SaveMedia
{
    // SaveMediaManager's calls into emusen-savemedia; the medium a file is, by its extension, crosses as data - see EmuSen_Backend.md §15.2.
    public static class SaveMediaNative
    {
        // Where the services' switch, the packs' code and Galaxia's lock all chose the library, so a card is read by the modules and held by the lock the rest of the program uses.
        public static bool Active => ServiceLibrary.Active && SystemCode.Active && PlatformLibrary.GalaxiaActive;

        // Whether a call goes to the library: chosen, every text crosses, and every medium named is one the library's table knows.
        public static bool Takes(IEnumerable<string> files, string? save) => Active && Crosses(files, save);

        public static bool Crosses(IEnumerable<string> files, string? save) =>
            ServiceLibrary.Takes(save) && files.All(f => ServiceLibrary.Takes(f) && (SaveMediaManager.MediumOf(f) is not { } m || SystemTable.Media.Any(r => r.Id == m.Id)));

        private static byte[] Request(string op, IEnumerable<string> files, Action<Utf8JsonWriter> args) => ServiceLibrary.Request(w =>
        {
            w.WriteString("op", op);
            args(w);
            w.WriteStartObject("media");
            foreach (string file in files.Distinct(StringComparer.Ordinal)) ServiceLibrary.WriteText(w, file, SaveMediaManager.MediumOf(file)?.Id);
            w.WriteEndObject();
        });

        // A thrown status as the C# threw it: a damaged directory, or a rename the system refused.
        private static JsonDocument Call(byte[] request)
        {
            try { return ServiceLibrary.CallJson("emusen_savemedia_op", request, 2048); }
            catch (ServiceFault fault) when (fault.Status == ServiceFault.BadArgument) { throw new ArgumentOutOfRangeException(); }
            catch (ServiceFault fault) when (fault.Status == ServiceFault.NotFound) { throw new DirectoryNotFoundException(fault.Message); }
            catch (ServiceFault fault) when (fault.Status == ServiceFault.Access) { throw new UnauthorizedAccessException(fault.Message); }
            catch (ServiceFault fault) when (fault.Status == ServiceFault.Io) { throw new IOException(fault.Message); }
        }

        private static MediaOutcome Outcome(byte[] request)
        {
            using JsonDocument d = Call(request);
            return new MediaOutcome(d.RootElement.GetProperty("done").GetBoolean(), d.RootElement.GetProperty("words").GetString()!);
        }

        public static MediumCard? Open(string path)
        {
            using JsonDocument d = Call(Request("open", [path], w => w.WriteString("file", path)));
            JsonElement card = d.RootElement.GetProperty("card");
            if (card.ValueKind == JsonValueKind.Null) return null;
            JsonElement l = card.GetProperty("listing");
            MediumListing? listing = l.ValueKind == JsonValueKind.Null ? null : new MediumListing(l.GetProperty("formatted").GetBoolean(),
                l.GetProperty("saves").EnumerateArray().Select(s => new SaveEntry(s.GetProperty("id").GetString()!, s.GetProperty("title").GetString()!, s.GetProperty("units").GetInt32(),
                    ServiceLibrary.Text(s, "game"), s.GetProperty("damaged").GetBoolean())).ToArray(),
                l.GetProperty("free").GetInt32(), ServiceLibrary.Text(l, "problem"));
            return new MediumCard(path, SaveMediaManager.MediumOf(path)!, card.GetProperty("exists").GetBoolean(), card.GetProperty("length").GetInt64(), listing);
        }

        public static MediaOutcome CopySave(string from, string save, string to) =>
            Outcome(Request("copy", [from, to], w => { w.WriteString("from", from); w.WriteString("save", save); w.WriteString("to", to); }));

        public static MediaOutcome MoveSave(string from, string save, string to) =>
            Outcome(Request("move", [from, to], w => { w.WriteString("from", from); w.WriteString("save", save); w.WriteString("to", to); }));

        public static MediaOutcome DeleteSave(string card, string save) =>
            Outcome(Request("delete", [card], w => { w.WriteString("card", card); w.WriteString("save", save); }));

        public static MediaOutcome Import(string to, IReadOnlyList<string> from) =>
            Outcome(Request("import", [to, .. from], w => { w.WriteString("to", to); ServiceLibrary.WriteList(w, "from", from); }));

        public static MediaOutcome New(string path) => Outcome(Request("new", [path], w => w.WriteString("file", path)));

        public static MediaOutcome Duplicate(string from, string to) => Outcome(Request("duplicate", [from, to], w => { w.WriteString("from", from); w.WriteString("to", to); }));

        public static MediaOutcome Rename(string from, string to) => Outcome(Request("rename", [from, to], w => { w.WriteString("from", from); w.WriteString("to", to); }));
    }
}
