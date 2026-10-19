using System.Globalization;
using System.Text.Json;
using System.Text.Json.Nodes;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Cheats;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;

namespace EmuSen.WiseMan.Crystal
{
    // The cheat registry's verbs over plain data, as emusen-crystal-debug's regs module runs them - see EmuSen_Backend.md §29.33.
    public sealed partial class CrystalDebugScript
    {
        private static CheatWrite? WriteOf(string text)
        {
            string[] f = text.Trim().Split(':');
            string G(int i) => i < f.Length ? f[i] : "";
            if (!CheatRegistry.TryParseWriteType(G(4), out CheatWriteType type)) return null;
            return new CheatWrite
            {
                Space = G(0) == "-" ? "" : G(0),
                Address = (int)Number(G(1)),
                Value = unchecked((uint)Number(G(2))),
                Width = (int)Number(G(3)),
                Type = type,
                BitPosition = G(5) is "-" or "" ? null : (int)Number(G(5)),
                BigEndian = G(6) == "1",
                RepeatCount = (int)Number(G(7)),
                RepeatAddAddress = (int)Number(G(8)),
                RepeatAddValue = unchecked((uint)Number(G(9))),
            };
        }

        private string? CheatAnswer(string verb, string rest, string[] words)
        {
            CheatRegistry c = _fake.Cheats;
            string W(int i) => i < words.Length ? words[i] : "";
            static string Desc(string s) => s.Replace('~', ' ');
            static string Id(Func<int> add)
            {
                try { return add().ToString(CultureInfo.InvariantCulture); }
                catch (ArgumentException e) { return "! " + e.Message; }
            }
            switch (verb)
            {
                case "cheat.add":
                {
                    string[] fields = rest.Split(" ; ");
                    string[] head = fields[0].Split(' ', StringSplitOptions.RemoveEmptyEntries);
                    var kind = head.Length > 0 && head[0] == "romPatch" ? CheatKind.RomPatch : CheatKind.RamPoke;
                    byte? compare = head.Length > 1 && head[1] != "-" ? (byte)Number(head[1]) : null;
                    var writes = new List<CheatWrite>();
                    foreach (string f in fields.Skip(1))
                    {
                        if (WriteOf(f) is not { } w) return "bad write";
                        writes.Add(w);
                    }
                    return Id(() => c.AddCheat(kind, writes, compare, Desc(head.Length > 2 ? head[2] : ""), !(head.Length > 3 && head[3] == "0")));
                }
                case "cheat.poke": return Id(() => c.AddRamPoke(W(0), (int)Number(W(1)), (byte)Number(W(2)), Desc(W(3))));
                case "cheat.patch": return Id(() => c.AddRomPatch((int)Number(W(0)), (byte)Number(W(1)), W(2) == "-" ? null : (byte)Number(W(2)), Desc(W(3))));
                case "cheat.remove": return Lower(c.RemoveCheat((int)Number(W(0))));
                case "cheat.enable": return Lower(c.SetEnabled((int)Number(W(0)), W(1) == "1"));
                case "cheat.master": c.MasterEnabled = W(0) == "1"; return "ok";
                case "cheat.clear": c.Clear(); return "ok";
                case "cheat.version": return $"{c.Version} {c.EnabledRomPatches} {Lower(c.MasterEnabled)}";
                case "cheat.list":
                    return string.Join(";", c.GetCheats().Select(x =>
                        $"{x.Id}:{x.Kind}:{(x.Compare is byte b ? b.ToString(CultureInfo.InvariantCulture) : "-")}:{x.Description}:{Lower(x.Enabled)}:[" +
                        string.Join(",", x.Writes.Select(v => $"{v.Space}:{v.Address}:{v.Value}:{v.Width}:{(int)v.Type}:{(v.BitPosition is int p ? p.ToString(CultureInfo.InvariantCulture) : "-")}:{Lower(v.BigEndian)}:{v.RepeatCount}:{v.RepeatAddAddress}:{v.RepeatAddValue}")) + "]"));
                case "cheat.apply": c.ApplyAll(Byte, Put); return "ok";
                case "cheat.rom": return c.TryPatchRom((uint)Number(W(0)), (byte)Number(W(1)), out byte patched) ? patched.ToString(CultureInfo.InvariantCulture) : "none";
                case "cheat.resolve": return string.Join(";", c.ResolveRomPatches(Number(W(0))).Select(p => $"{p.Address}:{p.Value}:{(p.Compare is byte b ? b.ToString(CultureInfo.InvariantCulture) : "-")}"));
                case "cheat.triples": return string.Join(";", CoreEngine.RomPatchTriples(c, Number(W(0)), Number(W(1))));
                case "cheat.file":
                {
                    // The saved list's own fields, without the two the serializer computes beside them.
                    var node = JsonNode.Parse(JsonSerializer.Serialize(c.ToCheatFile()))!;
                    foreach (JsonNode? entry in node["Cheats"]!.AsArray())
                    {
                        entry!.AsObject().Remove("IsRomPatch");
                        entry.AsObject().Remove("EffectiveWrites");
                    }
                    return node.ToJsonString();
                }
                case "cheat.load":
                {
                    CheatFile? file;
                    try { file = JsonSerializer.Deserialize<CheatFile>(rest, ConfigJson.Options); }
                    catch (JsonException) { return "bad json"; }
                    if (file is null) return "bad json";
                    var (loaded, skipped) = c.LoadFrom(file);
                    return $"{loaded} {skipped}";
                }
                default: return null;
            }
        }
    }
}
