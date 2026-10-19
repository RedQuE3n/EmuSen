using System.Globalization;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Telemetry;

namespace EmuSen.WiseMan.Crystal
{
    // The parity scripts' verbs run over the C# debugger, each answer a line as emusen-crystal-debug prints it - see EmuSen_Backend.md §29.30.
    public sealed partial class CrystalDebugScript
    {
        private readonly FakeTarget _fake = new();
        private string _chosen = "target";
        private string _chosenSet = "cpu";
        private string? _chosenSpace;
        private readonly List<CoreUnit> _units = new();
        private IReadOnlyList<string> _taken = Array.Empty<string>();

        // Every line of a script with $NAME replaced by its value, answered as "<line> => <answer>".
        public static string Run(string text, IReadOnlyDictionary<string, string>? values = null)
        {
            foreach (var (name, value) in values ?? new Dictionary<string, string>()) text = text.Replace("$" + name, value);
            var script = new CrystalDebugScript();
            var output = new System.Text.StringBuilder();
            try
            {
                foreach (string raw in text.Split('\n'))
                {
                    string line = raw.TrimEnd('\r');
                    if (line.Trim().Length == 0 || line.StartsWith('#')) continue;
                    int space = line.IndexOf(' ');
                    string verb = space < 0 ? line : line[..space];
                    string rest = space < 0 ? "" : line[(space + 1)..];
                    output.Append(line).Append(" => ").Append(script.Answer(verb, rest)).Append('\n');
                }
            }
            finally { script.Dispose(); }
            return output.ToString();
        }

        private string Answer(string verb, string rest)
        {
            string[] words = rest.Split(' ', StringSplitOptions.RemoveEmptyEntries);
            return RecordingAnswer(verb, words) ?? MachineAnswer(verb, rest, words) ?? FakeAnswer(verb, rest, words);
        }

        private static long Number(string word)
        {
            string w = word.Trim();
            if (w.StartsWith("0x", StringComparison.Ordinal) || w.StartsWith('$'))
                return ulong.TryParse(w.StartsWith('$') ? w[1..] : w[2..], NumberStyles.HexNumber, CultureInfo.InvariantCulture, out ulong h) ? unchecked((long)h) : 0;
            return long.TryParse(w, NumberStyles.AllowLeadingSign, CultureInfo.InvariantCulture, out long d) ? d : 0;
        }

        private static string Lower(bool value) => value ? "true" : "false";

        private FakeProvider? SetNamed(string word) => word switch
        {
            "cpu" => _fake.Cpu,
            "video" => _fake.Video,
            "apu" => _fake.Apu,
            "cop" => _fake.Cop,
            _ => null,
        };

        private IExpressionContext Chosen()
        {
            _fake.Expressions = _chosen switch
            {
                "cpu" => new DebugCpuExpressionContext(_fake, () => SetNamed(_chosenSet)!.Current, _chosenSpace),
                "main" => new DebugCpuExpressionContext(_fake, () => _fake.Cpu.Current, null,
                    _fake.Pack.Select(p => p.Name).Concat(_fake.Stack is null ? Array.Empty<string>() : new[] { "stackdepth" }).ToArray(),
                    name => _fake.Stack is long depth && string.Equals(name, "stackdepth", StringComparison.OrdinalIgnoreCase) ? depth
                        : _fake.Pack.FirstOrDefault(p => string.Equals(p.Name, name, StringComparison.OrdinalIgnoreCase)) is { Name: not null } p ? p.Value : null),
                _ => null,
            };
            return DebugExpressions.For(_fake);
        }

        // A registry whose evaluators BreakpointExpressions pointed at the chosen context, as a breakpoint's are.
        private BreakpointRegistry Prepared()
        {
            Chosen();
            var registry = new BreakpointRegistry();
            BreakpointExpressions.Add(_fake, registry, 0, 0, "1", null);
            return registry;
        }

        private string FakeAnswer(string verb, string rest, string[] words)
        {
            switch (verb)
            {
                case "regs":
                {
                    if (SetNamed(words[0]) is not { } set) return "bad set";
                    var rows = words.Skip(1).Select(w =>
                    {
                        int eq = w.IndexOf('=');
                        string name = eq < 0 ? w : w[..eq], v = eq < 0 ? "0" : w[(eq + 1)..];
                        int slash = v.IndexOf('/');
                        string value = slash < 0 ? v : v[..slash], bits = slash < 0 ? "8" : v[(slash + 1)..];
                        return new DebugRegisterValue(name, unchecked((ulong)Number(value)), (int)Number(bits));
                    }).ToList();
                    set.Live = rows.ToList();
                    set.Shown = rows.ToArray();
                    return "ok";
                }
                case "set":
                {
                    if (SetNamed(words[0]) is not { } set) return "bad set";
                    int i = set.Live.FindIndex(r => r.Name == words[1]);
                    if (i >= 0) set.Live[i] = new DebugRegisterValue(set.Live[i].Name, unchecked((ulong)Number(words[2])), set.Live[i].BitWidth);
                    return "ok";
                }
                case "refresh": DebugExpressions.RefreshRegisters(_fake); return "ok";
                case "refreshes": return _fake.Cpu.Refreshes.ToString(CultureInfo.InvariantCulture);
                case "label":
                    _fake.Labels.Add((int)Number(words[0]), words[1], words.Length > 2 ? string.Join(" ", words.Skip(2)) : null);
                    return "ok";
                case "unlabel": return Lower(_fake.Labels.RemoveByName(rest));
                case "unlabelat": return Lower(_fake.Labels.RemoveByAddress((int)Number(rest)));
                case "labels": return string.Join(",", _fake.Labels.All().Select(l => $"{l.Address}:{l.Name}:{l.Comment ?? "-"}"));
                case "labelload":
                {
                    var (added, errors) = _fake.Labels.LoadFromLines(rest.Replace("\\n", "\n").Replace("\\t", "\t").Split('\n'));
                    return $"{added} [{string.Join("|", errors)}]";
                }
                case "labelsave": return string.Join("|", _fake.Labels.ToLines());
                case "describe": return _fake.Labels.Describe((int)Number(words[0]), (int)Number(words[1]));
                case "space":
                {
                    var bytes = new byte[Number(words[1])];
                    for (int i = 2; i < words.Length && i - 2 < bytes.Length; i++) bytes[i - 2] = byte.TryParse(words[i], NumberStyles.HexNumber, CultureInfo.InvariantCulture, out byte b) ? b : (byte)0;
                    _fake.Spaces.Add(DelegateDebugMemorySpace.Over(words[0], bytes));
                    return "ok";
                }
                case "frame": _fake.Frame = Number(rest); return "ok";
                case "stack": _fake.Stack = rest == "none" ? null : Number(rest); return "ok";
                case "pack": _fake.Pack.Add((words[0], Number(words[1]))); return "ok";
                case "context":
                    _chosen = words.Length > 0 && words[0] is "cpu" or "main" ? words[0] : "target";
                    _chosenSet = words.Length > 1 && SetNamed(words[1]) is not null ? words[1] : "cpu";
                    _chosenSpace = words.Length > 2 ? words[2] : null;
                    return "ok";
                case "eval":
                    try { return ExpressionEvaluator.Format(new ExpressionEvaluator(Chosen()).Evaluate(rest)); }
                    catch (Exception e) { return "! " + e.Message; }
                case "check":
                    try { new ExpressionEvaluator(Chosen()).Check(rest); return "ok"; }
                    catch (Exception e) { return "! " + e.Message; }
                case "cond":
                {
                    var (result, error) = Prepared().ConditionEvaluator!(rest);
                    return error is null ? Lower(result) : "! " + error;
                }
                case "render":
                {
                    var (text, error) = Prepared().LogEvaluator!(rest);
                    return error is null ? text : "! " + error;
                }
                case "prepare":
                {
                    int bar = rest.IndexOf(" | ", StringComparison.Ordinal);
                    string c = bar < 0 ? rest : rest[..bar], l = bar < 0 ? "-" : rest[(bar + 3)..];
                    Chosen();
                    var registry = new BreakpointRegistry();
                    try
                    {
                        BreakpointExpressions.Add(_fake, registry, 0, 0, c == "-" ? null : c, l == "-" ? null : l);
                        return Lower(registry.ConditionEvaluator is not null);
                    }
                    catch (ExpressionException e) { return "! " + e.Message; }
                }
                case "symbols": return string.Join(",", Chosen().SymbolNames);
                case "format": return ExpressionEvaluator.Format(Number(rest));
                case "unit":
                    _units.Add(new CoreUnit((uint)Number(words[0]), words[1].Replace('~', ' '), words[2], words.Skip(3).Select(w =>
                    {
                        int slash = w.IndexOf('/');
                        return (slash < 0 ? w : w[..slash], (int)Number(slash < 0 ? "8" : w[(slash + 1)..]));
                    }).ToArray()));
                    return "ok";
                case "taken": _taken = words; return "ok";
                case "unitnames": return string.Join(",", new UnitSymbols(Info(_units), _taken).Names);
                case "unitfind": return new UnitSymbols(Info(_units), _taken).TryFind(rest, out uint u, out int index, out int count) ? $"{u} {index} {count}" : "none";
                default: return RegistryAnswer(verb, rest, words) ?? $"unknown verb {verb}";
            }
        }

        private static CoreMachineInfo Info(IReadOnlyList<CoreUnit> units) =>
            new("test", null, 60, 1, 0, 0, 0, 0, 0, 0, 0, Array.Empty<string>(), Array.Empty<CorePort>(), Array.Empty<CoreSpace>(), Array.Empty<CoreProcessor>(),
                Array.Empty<CoreBatteryFile>(), "", 0, new uint[] { 0 }, Array.Empty<string>(), null, null, false) { Units = units };

        // A register set the script sets, shown as its snapshot until refreshed.
        private sealed class FakeProvider : IRealtimeProvider<IReadOnlyList<DebugRegisterValue>>
        {
            public List<DebugRegisterValue> Live = new();
            public IReadOnlyList<DebugRegisterValue> Shown = Array.Empty<DebugRegisterValue>();
            public int Refreshes;
            public IReadOnlyList<DebugRegisterValue> Current => Shown;

            public void Refresh()
            {
                Refreshes++;
                Shown = Live.ToArray();
            }
        }

        // A target of plain data: the script's registers, spaces, labels, frame and the context it chose.
        private sealed class FakeTarget : IDebugTarget
        {
            public readonly FakeProvider Cpu = new(), Video = new(), Apu = new(), Cop = new();
            public readonly List<IDebugMemorySpace> Spaces = new();
            public readonly List<(string Name, long Value)> Pack = new();
            public long Frame;
            public long? Stack;

            public string CoreName => "fake";
            public long FrameCount => Frame;
            public int MaxSprites => 0;
            public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CpuRegisters => Cpu;
            public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> VideoRegisters => Video;
            public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> ApuRegisters => Apu;
            public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CoprocessorRegisters => Cop;
            public IRealtimeProvider<IReadOnlyList<DebugSpriteInfo>> Sprites { get; } = new PollingProvider<IReadOnlyList<DebugSpriteInfo>>(() => Array.Empty<DebugSpriteInfo>(), Array.Empty<DebugSpriteInfo>());
            public IRealtimeProvider<IReadOnlyList<DebugPaletteInfo>> Palettes { get; } = new PollingProvider<IReadOnlyList<DebugPaletteInfo>>(() => Array.Empty<DebugPaletteInfo>(), Array.Empty<DebugPaletteInfo>());
            public IRealtimeProvider<IReadOnlyList<DebugAudioChannelInfo>> AudioChannels { get; } = new PollingProvider<IReadOnlyList<DebugAudioChannelInfo>>(() => Array.Empty<DebugAudioChannelInfo>(), Array.Empty<DebugAudioChannelInfo>());
            public IRealtimeProvider<IReadOnlyList<DebugLoadInfo>> HardwareLoad { get; } = new PollingProvider<IReadOnlyList<DebugLoadInfo>>(() => Array.Empty<DebugLoadInfo>(), Array.Empty<DebugLoadInfo>());
            public void RefreshProviders() => throw new InvalidOperationException("a breakpoint refreshes the registers alone");
            public (byte[] Rgba, int Width, int Height) RenderTileSheet() => (Array.Empty<byte>(), 0, 0);
            public (byte[] Rgba, int Width, int Height) RenderPaletteSwatch() => (Array.Empty<byte>(), 0, 0);
            public (short[] Samples, int SampleRate) GetAudioSamples() => (Array.Empty<short>(), 0);
            public string GetSummaryText() => "";
            public IReadOnlyList<IDebugMemorySpace> GetMemorySpaces() => Spaces;
            public WatchRegistry Watches { get; } = new();
            public FrameLogRegistry FrameLog { get; } = new();
            public BreakpointRegistry Breakpoints { get; } = new();
            public CheatRegistry Cheats { get; } = new();
            public LabelRegistry Labels { get; } = new();
            public FreezeRegistry Freezes { get; } = new();
            FreezeRegistry? IDebugTarget.Freezes => Freezes;
            LabelRegistry? IDebugTarget.Labels => Labels;
            public IExpressionContext? Expressions { get; set; }
            IExpressionContext? IDebugTarget.Expressions => Expressions;
            public IReadOnlyList<DisassembledInstruction> Disassemble(string spaceName, int address, int count) => Array.Empty<DisassembledInstruction>();
            public (StaticReferenceKind Kind, int Target)? ClassifyStaticReference(DisassembledInstruction instr) => null;
            public int TilemapEntryStride => 0;
            public string DecodeTilemapEntry(IDebugMemorySpace space, int address) => "";
            public byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp) => Array.Empty<byte>();
            public void SetChannelMuted(int index, bool muted) { }
        }
    }
}
