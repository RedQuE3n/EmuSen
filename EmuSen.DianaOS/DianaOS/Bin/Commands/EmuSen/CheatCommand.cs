using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;
using EmuSen.Galaxia.Text;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Every cheat mechanism under one command, because a player has only "my cheats" - see `man cheat`.
    public class CheatCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        // Named by role, not by format, since a future core supplies its own - see EmuSen_Cheats.md.
        private readonly ICheatCodeCodec? _autoDetectCodec;
        private readonly ICheatCodeCodec? _explicitCodec;

        // Injected, because DianaOS has no idea which cores exist - see EmuSen_Settings_Reference.md §4.16.
        private readonly Func<IReadOnlyCollection<string>>? _supportedCheatSystems;

        public CheatCommand(ICheatCodeCodec? autoDetectCodec = null, ICheatCodeCodec? explicitCodec = null,
            Func<IReadOnlyCollection<string>>? supportedCheatSystems = null)
        {
            _autoDetectCodec = autoDetectCodec;
            _explicitCodec = explicitCodec;
            _supportedCheatSystems = supportedCheatSystems;
        }

        // The moment 250MB lands is the moment to say most of it is unusable here.
        private string PruneOffer(CheatDatabase db)
        {
            CheatPrunePlan plan = CheatDatabasePruner.Plan(db, _supportedCheatSystems?.Invoke() ?? Array.Empty<string>());
            if (!plan.CanApply) return "";

            return $"\n\n{plan.Removing.Count} of {plan.Removing.Count + plan.Keeping.Count} system(s) have no core in this build " +
                   $"({plan.Files} file(s), {CheatPrunePlan.Human(plan.Bytes)}). `cheat db prune` lists them.";
        }

        public string Name => "cheat";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  cheat add <code> [description]    decode a code, guessing whether it's Pro",
            "                                     Action Replay/Game Wizard or Game Genie from",
            "                                     its formatting (see `gg` if it guesses wrong -",
            "                                     both formats use the same 8 hex-digit",
            "                                     characters, so this is a best-effort guess,",
            "                                     not a guarantee)",
            "  cheat poke <space> <addr> <value> [description]",
            "                                     add a raw RAM-poke cheat directly, bypassing",
            "                                     code decoding (e.g. an address you already",
            "                                     found with `search`)",
            "  cheat gg <code> [description]     decode a real SNES Game Genie code (8",
            "                                     characters) and add it as a ROM patch, enabled",
            "                                     - use this instead of `add` when its guess is",
            "                                     wrong for a code formatted unconventionally",
            "  cheat rompatch <addr> <value> [<compare>|-] [description]",
            "                                     add a Game Genie-style ROM-read intercept",
            "                                     directly, bypassing code decoding: substitute",
            "                                     <value> for reads of cartridge address <addr>.",
            "                                     <compare> gates the patch to only apply while",
            "                                     the real byte there equals it (pass - to skip,",
            "                                     an unconditional patch - what every real SNES",
            "                                     Game Genie code decodes to; unlike NES/Genesis,",
            "                                     SNES Game Genie codes never carry a compare)",
            "  cheat list                         list every cheat with its ID, kind, and state",
            "  cheat master [on|off]              one switch over every cheat at once, without",
            "                                     touching any cheat's own enabled state (no",
            "                                     argument reports which way it is set)",
            "  cheat enable <id>                  turn a cheat back on",
            "  cheat disable <id>                 turn a cheat off without removing it",
            "  cheat remove <id>                  remove a cheat entirely",
            "  cheat clear                        remove every cheat",
            "  cheat save <name>                  write every cheat to",
            "                                     /etc/EmuSen/cheats/<name>.json",
            "  cheat load <name>                  add every cheat from that file to whatever",
            "                                     is already loaded (`cheat clear` first to",
            "                                     replace rather than merge)",
            "  cheat files                        list the saved cheat files",
            "  cheat import <path.cht>            import RetroArch/libretro .cht cheats, added",
            "                                     disabled so nothing turns on by surprise",
            "  cheat export <path.cht>            write RAM-poke cheats out as a .cht file",
            "  cheat db [status]                  where the cheat database is, and what is in it",
            "  cheat db find <game>               search the database for a game",
            "  cheat db load <game>               import that game's cheats, disabled",
            "  cheat db update                    download the libretro cheat database (CC BY-SA 4.0)",
            "  cheat db prune [--apply]           delete the systems no core in this build can use",
        });

        // AppSettings when set, the sandbox's Cheats folder otherwise - see `man cheat`.
        private static string CheatDatabaseDirectory
        {
            get
            {
                string? configured = AppSettings.Load().CheatDatabaseDirectory;
                return string.IsNullOrWhiteSpace(configured) ? DianaOSSandbox.CheatDatabaseDirectory : configured;
            }
        }

        // Both SNES formats share one character set, so the guess is punctuation - see EmuSen_Cheats.md.
        public static bool LooksLikeGameGenieFormat(string code)
        {
            for (int i = 0; i < code.Length; i++)
            {
                if (!Uri.IsHexDigit(code[i])) return i == 4;
            }
            return false; // no separator at all - assume Pro Action Replay/Game Wizard
        }

        // Ask the codecs first; the punctuation guess is the last resort - see EmuSen_Cheats.md.
        public static bool PrefersExplicitCodec(ICheatCodeCodec? explicitCodec, ICheatCodeCodec? autoDetectCodec, string code)
        {
            bool explicitClaims = explicitCodec?.CanDecode(code) == true;
            bool autoClaims = autoDetectCodec?.CanDecode(code) == true;

            if (explicitClaims != autoClaims) return explicitClaims;

            return LooksLikeGameGenieFormat(code);
        }

        private bool PrefersExplicitCodec(string code) =>
            PrefersExplicitCodec(_explicitCodec, _autoDetectCodec, code);

        // A cheat is one toggle however many addresses it drives, and the list must show that.
        private static string FormatCheat(CheatInfo c)
        {
            string state = c.Enabled ? "on " : "off";
            string kind = c.Kind == CheatKind.RamPoke ? "RAM" : "ROM";
            string compareText = c.Compare.HasValue ? $" if==0x{c.Compare.Value:X2}" : "";

            if (c.Writes.Count == 1)
            {
                return $"  #{c.Id}: [{state}] {kind}  {FormatWrite(c.Writes[0], c.Kind)}{compareText}  {c.Description}";
            }

            var lines = new List<string> { $"  #{c.Id}: [{state}] {kind}  {c.Writes.Count} writes{compareText}  {c.Description}" };
            lines.AddRange(c.Writes.Select(w => $"        {FormatWrite(w, c.Kind)}"));
            return string.Join('\n', lines);
        }

        public static string FormatWrite(CheatWrite w, CheatKind kind)
        {
            string where = kind == CheatKind.RamPoke ? $"{w.Space} 0x{w.Address:X}" : $"0x{w.Address:X6}";
            string op = w.Type switch
            {
                CheatWriteType.Increase => "+=",
                CheatWriteType.Decrease => "-=",
                _ => "=",
            };

            if (w.BitPosition is int bit) return $"{where} bit{bit} {op} {w.Value & 1}";

            string value = $"0x{w.Value.ToString("X" + w.EffectiveWidth * 2)}";
            string width = w.EffectiveWidth > 1 ? $" ({w.EffectiveWidth}-byte{(w.BigEndian ? ", big-endian" : "")})" : "";
            string repeat = w.EffectiveRepeatCount > 1
                ? $" x{w.EffectiveRepeatCount} step 0x{w.RepeatAddAddress:X}" + (w.RepeatAddValue != 0 ? $"/+0x{w.RepeatAddValue:X}" : "")
                : "";
            return $"{where} {op} {value}{width}{repeat}";
        }

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: cheat add|poke|gg|rompatch|list|master|enable|disable|remove|clear|save|load|files|import|export ...";
            string sub = parts[1].ToLowerInvariant();
            var cheats = target.Cheats;

            switch (sub)
            {
                case "add":
                {
                    if (parts.Length < 3) return "Usage: cheat add <code> [description]";
                    string code = parts[2];
                    string description = parts.Length > 3 ? string.Join(' ', parts.Skip(3)) : code;

                    if (PrefersExplicitCodec(code))
                    {
                        if (_explicitCodec is null) return "No Game Genie-style cheat codec is registered for this target.";
                        (int ggAddress, byte ggValue) = _explicitCodec.Decode(code);
                        byte? ggCompare = _explicitCodec.DecodeCompare(code);
                        int ggId = cheats.AddRomPatch(ggAddress, ggValue, ggCompare, description);
                        string ggIf = ggCompare.HasValue ? $" if==0x{ggCompare.Value:X2}" : "";
                        return $"Cheat #{ggId} added (detected {_explicitCodec.Name} format): ROM 0x{ggAddress:X6} = 0x{ggValue:X2}{ggIf} ({description})";
                    }

                    if (_autoDetectCodec is null) return "No cheat codec is registered for this target.";
                    (int address, byte value) = _autoDetectCodec.Decode(code);
                    string space = _autoDetectCodec.SpaceName ?? "CpuBus";
                    int id = cheats.AddRamPoke(space, address, value, description);
                    return $"Cheat #{id} added (detected {_autoDetectCodec.Name} format): {space} 0x{address:X6} = 0x{value:X2} ({description})";
                }
                case "poke":
                {
                    if (parts.Length < 5) return "Usage: cheat poke <space> <addr> <value> [description]";
                    FindSpace(target, parts[2]); // validates the space name exists, or throws a helpful error
                    int addr = ParseHex(parts[3]);
                    byte value = (byte)ParseHex(parts[4]);
                    string description = parts.Length > 5 ? string.Join(' ', parts.Skip(5)) : $"{parts[2]} 0x{addr:X}";
                    int id = cheats.AddRamPoke(parts[2], addr, value, description);
                    return $"Cheat #{id} added: {parts[2]} 0x{addr:X} = 0x{value:X2} ({description})";
                }
                case "gg":
                {
                    if (parts.Length < 3) return "Usage: cheat gg <code> [description]";
                    if (_explicitCodec is null) return "No Game Genie-style cheat codec is registered for this target.";
                    (int ggAddress, byte ggValue) = _explicitCodec.Decode(parts[2]);
                    byte? ggCompare = _explicitCodec.DecodeCompare(parts[2]);
                    string ggDescription = parts.Length > 3 ? string.Join(' ', parts.Skip(3)) : parts[2];
                    int ggId = cheats.AddRomPatch(ggAddress, ggValue, ggCompare, ggDescription);
                    string ggIfText = ggCompare.HasValue ? $" if==0x{ggCompare.Value:X2}" : "";
                    return $"Cheat #{ggId} added: ROM 0x{ggAddress:X6} = 0x{ggValue:X2}{ggIfText} ({ggDescription})";
                }
                case "rompatch":
                {
                    if (parts.Length < 4) return "Usage: cheat rompatch <addr> <value> [<compare>|-] [description]";
                    int addr = ParseHex(parts[2]);
                    byte value = (byte)ParseHex(parts[3]);
                    byte? compare = null;
                    int descStart = 4;
                    if (parts.Length > 4)
                    {
                        if (parts[4] != "-") compare = (byte)ParseHex(parts[4]);
                        descStart = 5;
                    }
                    string description = parts.Length > descStart ? string.Join(' ', parts.Skip(descStart)) : $"ROM 0x{addr:X6}";
                    int id = cheats.AddRomPatch(addr, value, compare, description);
                    string compareText = compare.HasValue ? $" if==0x{compare.Value:X2}" : "";
                    return $"Cheat #{id} added: ROM 0x{addr:X6} = 0x{value:X2}{compareText} ({description})";
                }
                case "list":
                {
                    var list = cheats.GetCheats();
                    if (list.Count == 0) return "No cheats added.";

                    IEnumerable<string> lines = list.Select(FormatCheat);
                    // Or an off master switch makes every [on] below it a lie.
                    if (!cheats.MasterEnabled) lines = new[] { "Cheats are OFF at the master switch (`cheat master on`)." }.Concat(lines);
                    return string.Join('\n', lines);
                }
                case "master":
                {
                    if (parts.Length < 3) return $"Cheats are {(cheats.MasterEnabled ? "ON" : "OFF")} at the master switch.";

                    string want = parts[2].ToLowerInvariant();
                    if (want is not ("on" or "off")) return "Usage: cheat master [on|off]";

                    cheats.MasterEnabled = want == "on";
                    return $"Cheats are now {(cheats.MasterEnabled ? "ON" : "OFF")} at the master switch.";
                }
                case "enable":
                {
                    if (parts.Length < 3) return "Usage: cheat enable <id>";
                    bool ok = cheats.SetEnabled(ParseHex(parts[2]), true);
                    return ok ? $"Cheat #{parts[2]} enabled." : $"No cheat #{parts[2]} found.";
                }
                case "disable":
                {
                    if (parts.Length < 3) return "Usage: cheat disable <id>";
                    bool ok = cheats.SetEnabled(ParseHex(parts[2]), false);
                    return ok ? $"Cheat #{parts[2]} disabled." : $"No cheat #{parts[2]} found.";
                }
                case "remove":
                {
                    if (parts.Length < 3) return "Usage: cheat remove <id>";
                    bool removed = cheats.RemoveCheat(ParseHex(parts[2]));
                    return removed ? $"Cheat #{parts[2]} removed." : $"No cheat #{parts[2]} found.";
                }
                case "clear":
                {
                    cheats.Clear();
                    return "All cheats removed.";
                }
                case "save":
                {
                    if (parts.Length < 3) return "Usage: cheat save <name>";
                    if (!CheatFile.IsValidName(parts[2])) return $"cheat save: '{parts[2]}' is not a usable file name.";

                    ConfigFile<CheatFile> file = CheatFile.For(parts[2]);
                    if (!file.Save(cheats.ToCheatFile())) return $"cheat save: couldn't write {file.Path}";
                    return $"Saved {cheats.GetCheats().Count} cheat(s) to {file.Path}";
                }
                case "load":
                {
                    if (parts.Length < 3) return "Usage: cheat load <name>";
                    if (!CheatFile.IsValidName(parts[2])) return $"cheat load: '{parts[2]}' is not a usable file name.";

                    ConfigFile<CheatFile> file = CheatFile.For(parts[2]);
                    CheatFile? loaded = file.Load();
                    if (loaded is null) return $"cheat load: no readable cheat file at {file.Path}";

                    // Added to what is loaded, not replacing it; `cheat clear` first if that was meant.
                    (int added, int skipped) = cheats.LoadFrom(loaded);
                    string skippedText = skipped > 0 ? $" ({skipped} unparseable entr{(skipped == 1 ? "y" : "ies")} skipped)" : "";
                    return $"Loaded {added} cheat(s) from {file.Path}{skippedText}";
                }
                case "import":
                {
                    if (parts.Length < 3) return "Usage: cheat import <path.cht>";
                    // Joined, not parts[2]: a path is the last argument and may contain spaces.
                    string importArg = string.Join(' ', parts.Skip(2));
                    if (!DianaOSSandbox.TryResolve(importArg, out string importPath)) return $"cheat import: '{importArg}' is outside the sandbox.";
                    if (!System.IO.File.Exists(importPath)) return $"cheat import: no file at {importPath}";

                    CheatImportResult imported;
                    try { imported = CheatImport.FromChtFile(cheats, importPath, _autoDetectCodec); }
                    catch (Exception ex) { return $"cheat import: {ex.Message}"; }

                    string skippedText = imported.Skipped > 0 ? $", {imported.Skipped} skipped" : "";
                    return $"Imported {imported.Loaded} cheat(s) from {System.IO.Path.GetFileName(importPath)}{skippedText} - all disabled, `cheat enable <id>` to turn one on.";
                }
                case "export":
                {
                    if (parts.Length < 3) return "Usage: cheat export <path.cht>";
                    string exportArg = string.Join(' ', parts.Skip(2));
                    if (!DianaOSSandbox.TryResolve(exportArg, out string exportPath)) return $"cheat export: '{exportArg}' is outside the sandbox.";

                    var exportable = cheats.GetCheats().Where(c => c.Kind == CheatKind.RamPoke).ToList();
                    int romPatches = cheats.GetCheats().Count - exportable.Count;

                    var chtCheats = exportable
                        .Select(c => new ChtCheat { Description = c.Description, Enabled = c.Enabled, Writes = c.Writes })
                        .ToList();

                    try
                    {
                        System.IO.Directory.CreateDirectory(System.IO.Path.GetDirectoryName(exportPath)!);
                        System.IO.File.WriteAllText(exportPath, ChtFile.Write(chtCheats));
                    }
                    catch (Exception ex) { return $"cheat export: {ex.Message}"; }

                    // RetroArch's model has no ROM-read substitution, so there is nothing honest to write.
                    string skippedText = romPatches > 0 ? $" ({romPatches} ROM patch(es) skipped - .cht has no equivalent)" : "";
                    return $"Exported {chtCheats.Count} cheat(s) to {exportPath}{skippedText}";
                }
                case "db":
                {
                    string dbSub = parts.Length > 2 ? parts[2].ToLowerInvariant() : "status";
                    var db = new CheatDatabase(CheatDatabaseDirectory);

                    switch (dbSub)
                    {
                        case "status":
                        {
                            if (!db.Exists) return $"No cheat database at {db.Directory}\nRun `cheat db update` to download one, or point AppSettings.CheatDatabaseDirectory at an existing RetroArch cheats folder.";
                            var systems = db.Systems();
                            int total = systems.Sum(s => s.Count);
                            if (total == 0) return $"Cheat database at {db.Directory} is empty.";
                            return string.Join('\n', new[] { $"{total} cheat file(s) in {db.Directory}" }
                                .Concat(systems.Select(s => $"  {s.System}  ({s.Count})")));
                        }
                        case "find":
                        {
                            if (parts.Length < 4) return "Usage: cheat db find <game>";
                            var matches = db.Find(string.Join(' ', parts.Skip(3)));
                            if (matches.Count == 0) return "No matching cheat file.";
                            return string.Join('\n', matches.Select(m => $"  {m.Game}   [{m.System}]"));
                        }
                        case "load":
                        {
                            if (parts.Length < 4) return "Usage: cheat db load <game>";
                            if (db.BestMatch(string.Join(' ', parts.Skip(3))) is not CheatDatabaseEntry match)
                            {
                                return "No matching cheat file - try `cheat db find` first.";
                            }

                            CheatImportResult found;
                            try { found = CheatImport.FromChtFile(cheats, match.Path, _autoDetectCodec); }
                            catch (Exception ex) { return $"cheat db load: {ex.Message}"; }

                            string dbSkipped = found.Skipped > 0 ? $", {found.Skipped} skipped" : "";
                            return $"Loaded {found.Loaded} cheat(s) for {match.Game}{dbSkipped} - all disabled, `cheat enable <id>` to turn one on.";
                        }
                        case "update":
                        {
                            CheatDatabaseInstallResult result;
                            try
                            {
                                using Stream zip = CheatDatabaseInstaller.Fetch(CheatDatabaseInstaller.LibretroCheatsUrl);
                                result = CheatDatabaseInstaller.Install(zip, db.Directory);
                            }
                            catch (Exception ex)
                            {
                                return $"cheat db update: {ex.Message}";
                            }

                            string offer = PruneOffer(new CheatDatabase(db.Directory));
                            return $"Installed {result.Installed} cheat file(s) into {db.Directory}{offer}\n\n{CheatDatabaseInstaller.Attribution}";
                        }
                        case "prune":
                        {
                            CheatPrunePlan plan = CheatDatabasePruner.Plan(db, _supportedCheatSystems?.Invoke() ?? Array.Empty<string>());
                            if (!plan.CanApply) return $"Nothing to prune - {plan.Reason}";

                            var listing = plan.Removing
                                .OrderByDescending(s => s.Files)
                                .Select(s => $"  {s.System}  ({s.Files} file(s), {CheatPrunePlan.Human(s.Bytes)})");

                            string header = $"Would delete {plan.Removing.Count} system(s), {plan.Files} file(s), {CheatPrunePlan.Human(plan.Bytes)}:";
                            string kept = $"Keeping: {string.Join(", ", plan.Keeping)}";

                            // Irreversible, and the way back is a 250MB download, so the flag is not optional.
                            if (!parts.Skip(3).Any(p => p.Equals("--apply", StringComparison.OrdinalIgnoreCase)))
                            {
                                return string.Join('\n', new[] { header }.Concat(listing).Append(kept)
                                    .Append("\nRe-run as `cheat db prune --apply` to delete them."));
                            }

                            (int pruned, IReadOnlyList<string> failed) = CheatDatabasePruner.Apply(db, plan);
                            string trouble = failed.Count > 0 ? $"\nCouldn't remove: {string.Join("; ", failed)}" : "";
                            return $"Deleted {pruned} system(s), {plan.Files} file(s), {CheatPrunePlan.Human(plan.Bytes)}.\n{kept}{trouble}";
                        }
                        default:
                        {
                            string[] dbSubcommands = { "status", "find", "load", "update", "prune" };
                            return $"Unknown 'cheat db' subcommand '{dbSub}'.{Suggestion.Hint(dbSub, dbSubcommands)} Try {string.Join('/', dbSubcommands)}.";
                        }
                    }
                }
                case "files":
                {
                    IReadOnlyList<string> names = CheatFile.ListNames();
                    if (names.Count == 0) return $"No cheat files in {CheatFile.DirectoryPath}";
                    return string.Join('\n', new[] { CheatFile.DirectoryPath }.Concat(names.Select(n => "  " + n)));
                }
                default:
                {
                    // Named once so the suggestion and the "Try" list cannot drift.
                    string[] subcommands = { "add", "poke", "gg", "rompatch", "list", "master", "enable", "disable", "remove", "clear", "save", "load", "files", "import", "export", "db" };
                    return $"Unknown 'cheat' subcommand '{sub}'.{Suggestion.Hint(sub, subcommands)} Try {string.Join('/', subcommands)}.";
                }
            }
        }
    }
}
