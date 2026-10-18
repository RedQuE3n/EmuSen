using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Common.Firmware;
using EmuSen.Cores;
using EmuSen.Crystal.SaveMedia;
using EmuSen.Crystal.Session;
using EmuSen.Galaxia.Models;

namespace EmuSen.Crystal.Managed
{
    // Firmware asked before a load and the card manager over a console's cards, both needing no session - see EmuSen_Backend.md §3.5 and §14.
    internal sealed partial class ManagedRuntime
    {
        public static byte[] FirmwareNeededJson(string path, string? engine) => CrystalJson.Write(w =>
        {
            w.WriteStartArray();
            foreach (FirmwareRequest r in Common.EmulatorSession.MissingFirmwareFor(path, engine ?? EngineFor(path))) Request(w, r);
            w.WriteEndArray();
        });

        private static void Request(Utf8JsonWriter w, FirmwareRequest r)
        {
            w.WriteStartObject();
            w.WriteString("core", r.CoreName);
            w.WriteString("chip", r.ChipName);
            w.WriteString("file", r.FileName);
            w.WriteNumber("size", r.Size);
            w.WriteString("purpose", r.Purpose);
            CrystalJson.Strings(w, "alternate_names", r.AlternateNames);
            w.WriteStartArray("parts");
            foreach (IReadOnlyList<string> form in r.Parts)
            {
                w.WriteStartArray();
                foreach (string part in form) w.WriteStringValue(part);
                w.WriteEndArray();
            }
            w.WriteEndArray();
            w.WriteBoolean("required", r.Required);
            if (r.ReplacementEffect is null) w.WriteNull("replacement");
            else
            {
                w.WriteStartObject("replacement");
                w.WriteString("effect", r.ReplacementEffect);
                CrystalJson.StringOrNull(w, "cost", r.ReplacementCost);
                w.WriteEndObject();
            }
            w.WriteEndObject();
        }

        // A request as firmware_needed lists it, read back.
        private static FirmwareRequest RequestOf(JsonElement e)
        {
            int size = e.TryGetProperty("size", out JsonElement s) && s.TryGetInt32(out int n) ? n : throw new CrystalException(SessionAbi.BadArgument, "the request's \"size\" must be given");
            return new FirmwareRequest(CrystalJson.String(e, "core"), CrystalJson.String(e, "chip"), CrystalJson.String(e, "file"), size, CrystalJson.OptionalString(e, "purpose") ?? "required")
            {
                AlternateNames = e.TryGetProperty("alternate_names", out JsonElement a) && a.ValueKind == JsonValueKind.Array ? a.EnumerateArray().Select(x => x.GetString() ?? "").ToArray() : Array.Empty<string>(),
            };
        }

        public static void InstallFirmware(ReadOnlySpan<byte> requestJson, string source)
        {
            FirmwareRequest request = RequestOf(CrystalJson.Parse(requestJson));
            if (!File.Exists(source)) throw new CrystalException(CrystalNumbers.Io, $"{source} does not exist");
            long length = new FileInfo(source).Length;
            if (length != request.Size) throw new CrystalException(SessionAbi.BadArgument, $"{Path.GetFileName(source)} is {length:N0} bytes, and {request.FileName} is {request.Size:N0}");
            if (!FirmwareLibrary.Install(request, source)) throw new CrystalException(CrystalNumbers.Io, $"{request.FileName} could not be written to {FirmwareLibrary.Directory}");
        }

        public static byte[] FirmwareOverviewJson() => CrystalJson.Write(w =>
        {
            w.WriteStartObject();
            w.WriteString("folder", FirmwareOverview.Folder);
            CrystalJson.Strings(w, "folders", FirmwareOverview.Folders);
            w.WriteStartArray("systems");
            foreach (FirmwareSystem system in FirmwareOverview.Build())
            {
                w.WriteStartObject();
                w.WriteString("name", system.Name);
                w.WriteString("engine", system.Engine);
                w.WriteStartArray("items");
                foreach (FirmwareItem i in system.Items)
                {
                    w.WriteStartObject();
                    w.WriteString("label", i.Label);
                    w.WriteString("file", i.FileName);
                    w.WriteNumber("size", i.Size);
                    w.WriteBoolean("present", i.Present);
                    w.WriteBoolean("wrong_size", i.WrongSize);
                    CrystalJson.StringOrNull(w, "effect", i.Effect);
                    CrystalJson.StringOrNull(w, "cost", i.Cost);
                    CrystalJson.StringOrNull(w, "found_in", i.FoundIn);
                    CrystalJson.StringOrNull(w, "wrong_size_in", i.WrongSizeIn);
                    w.WriteEndObject();
                }
                w.WriteEndArray();
                w.WriteEndObject();
            }
            w.WriteEndArray();
            w.WriteEndObject();
        });

        // A session's save media, each slot's file as its medium's mode places it.
        public static byte[] SaveMediaJson(ManagedSession s) => CrystalJson.Write(w =>
        {
            GraphicsConfig config = GraphicsConfig.Load();
            w.WriteStartObject();
            CrystalJson.StringOrNull(w, "notice", s.Engine?.MediaNotice);
            w.WriteStartArray("media");
            foreach (SaveMedium medium in MemoryCards.MediaFor(s.Console))
            {
                CardChoice choice = MemoryCards.Read(key => config.Value(s.Console, key), medium);
                w.WriteStartObject();
                w.WriteString("medium", medium.Id);
                w.WriteString("title", medium.Title);
                w.WriteString("noun", medium.Noun);
                w.WriteString("extension", medium.Extension);
                w.WriteString("kind", medium.IsConsole ? "console" : "removable");
                w.WriteString("mode", choice.Mode);
                w.WriteStartArray("slots");
                for (int slot = 0; slot < medium.Slots.Count; slot++)
                {
                    w.WriteStartObject();
                    w.WriteString("label", medium.SlotLabel(slot));
                    if (choice.Apart)
                    {
                        string name = choice.Mode == MemoryCards.Named ? medium.NameFor(choice, slot) : medium.DefaultName(slot);
                        w.WriteString("name", name);
                        w.WriteString("path", medium.PathOf(name));
                    }
                    else
                    {
                        w.WriteNull("name");
                        w.WriteNull("path");
                    }
                    w.WriteEndObject();
                }
                w.WriteEndArray();
                w.WriteEndObject();
            }
            w.WriteEndArray();
            w.WriteEndObject();
        });

        public static byte[] CardsOp(uint op, ReadOnlySpan<byte> json)
        {
            JsonElement args = CrystalJson.Parse(json);
            return op switch
            {
                CrystalNumbers.CardsList => CardsList(CrystalJson.String(args, "console")),
                CrystalNumbers.CardsOpen => CardOpen(CrystalJson.String(args, "path")),
                CrystalNumbers.CardsCopy => Outcome(SaveMediaManager.CopySave(CrystalJson.String(args, "from"), CrystalJson.String(args, "save"), CrystalJson.String(args, "to"))),
                CrystalNumbers.CardsMove => Outcome(SaveMediaManager.MoveSave(CrystalJson.String(args, "from"), CrystalJson.String(args, "save"), CrystalJson.String(args, "to"))),
                CrystalNumbers.CardsDelete => Outcome(SaveMediaManager.DeleteSave(CrystalJson.String(args, "card"), CrystalJson.String(args, "save"))),
                CrystalNumbers.CardsImport => Outcome(SaveMediaManager.Import(CrystalJson.String(args, "to"),
                    args.TryGetProperty("from", out JsonElement from) && from.ValueKind == JsonValueKind.Array ? from.EnumerateArray().Select(f => f.GetString() ?? "").ToArray()
                    : throw new CrystalException(SessionAbi.BadArgument, "\"from\" must be given as a list of paths"))),
                CrystalNumbers.CardsNew => Outcome(SaveMediaManager.New(CrystalJson.String(args, "path"))),
                CrystalNumbers.CardsDuplicate => Outcome(SaveMediaManager.Duplicate(CrystalJson.String(args, "from"), CrystalJson.String(args, "to"))),
                CrystalNumbers.CardsRename => Outcome(SaveMediaManager.Rename(CrystalJson.String(args, "from"), CrystalJson.String(args, "to"))),
                _ => throw new CrystalException(SessionAbi.NotSupported, $"no cards op {op} in 1.{CrystalNumbers.Minor}"),
            };
        }

        private static byte[] Outcome(MediaOutcome outcome) => CrystalJson.Write(w =>
        {
            w.WriteStartObject();
            w.WriteBoolean("done", outcome.Done);
            w.WriteString("words", outcome.Words);
            w.WriteEndObject();
        });

        private static byte[] CardsList(string console) => CrystalJson.Write(w =>
        {
            w.WriteStartArray();
            foreach (SaveMedium medium in MemoryCards.MediaFor(console))
            {
                w.WriteStartObject();
                w.WriteString("medium", medium.Id);
                w.WriteString("title", medium.Title);
                w.WriteString("noun", medium.Noun);
                w.WriteString("extension", medium.Extension);
                w.WriteStartArray("cards");
                foreach (MemoryCards.MediumFile card in MemoryCards.CardsOf(medium))
                {
                    w.WriteStartObject();
                    w.WriteString("path", card.Path);
                    w.WriteString("name", card.Name);
                    w.WriteString("label", card.Label);
                    w.WriteBoolean("games_own", card.GamesOwn);
                    CrystalJson.StringOrNull(w, "held_by", SaveMediaManager.HeldBy(card.Path));
                    w.WriteEndObject();
                }
                w.WriteEndArray();
                w.WriteEndObject();
            }
            w.WriteEndArray();
        });

        private static byte[] CardOpen(string path)
        {
            MediumCard card = SaveMediaManager.Open(path) ?? throw new CrystalException(SessionAbi.Absent, $"{Path.GetFileName(path)} is of no medium a system pack names");
            return CrystalJson.Write(w =>
            {
                w.WriteStartObject();
                w.WriteString("medium", card.Medium.Id);
                w.WriteBoolean("exists", card.Exists);
                w.WriteNumber("length", card.Length);
                w.WriteBoolean("formatted", card.Listing?.Formatted ?? false);
                w.WriteNumber("free", card.Listing?.FreeUnits ?? 0);
                CrystalJson.StringOrNull(w, "problem", card.Listing is null ? "no format module reads this medium" : card.Listing.Problem);
                w.WriteStartArray("saves");
                foreach (SaveEntry save in card.Listing?.Saves ?? Array.Empty<SaveEntry>())
                {
                    w.WriteStartObject();
                    w.WriteString("id", save.Id);
                    w.WriteString("title", save.Title);
                    w.WriteNumber("units", save.Units);
                    CrystalJson.StringOrNull(w, "game", save.Game);
                    w.WriteBoolean("damaged", save.Damaged);
                    w.WriteEndObject();
                }
                w.WriteEndArray();
                w.WriteEndObject();
            });
        }
    }
}
