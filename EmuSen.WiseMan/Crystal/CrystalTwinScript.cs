using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Text;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Crystal.Session;

namespace EmuSen.WiseMan.Crystal
{
    // A script of the session service's calls run on one implementation, each through its twin or through the session's table, its answers in order - see EmuSen_Backend.md §29.21.
    public sealed unsafe class CrystalTwinScript : IDisposable
    {
        // Each function's arguments after the session, by letter (u uint, i int, l long, q ulong, t text, b bytes, o the answer), and whether it returns an int.
        private static readonly Dictionary<string, (string Args, bool Int)> Signatures = new()
        {
            ["info"] = ("o", false), ["summary"] = ("o", false), ["run_control"] = ("uul", false), ["halt_state"] = ("uo", false),
            ["spaces"] = ("o", false), ["space_read"] = ("tuo", false), ["space_write"] = ("tub", true), ["resolve_physical"] = ("uo", false),
            ["processors"] = ("o", false), ["registers"] = ("uo", false), ["set_register"] = ("utq", true), ["disassemble"] = ("utuuto", false),
            ["bp_add"] = ("ut", false), ["bp_add_data"] = ("ut", false), ["bp_set_enabled"] = ("uuu", true), ["bp_remove"] = ("uu", true),
            ["bp_list"] = ("uo", false), ["bp_log"] = ("uuo", false), ["bp_log_clear"] = ("u", true), ["bp_arm_condition"] = ("utu", true),
            ["bp_conditions"] = ("uo", false), ["bp_depth_guard"] = ("ui", true), ["bp_forbid_range"] = ("uuu", false), ["bp_uninitialized"] = ("utu", true),
            ["watch_add"] = ("tuuu", false), ["watch_remove"] = ("u", true), ["watch_list"] = ("o", false), ["watch_events"] = ("uuo", false),
            ["watch_summary"] = ("uo", false), ["watch_clear"] = ("u", true), ["framelog_add"] = ("tuu", false), ["framelog_remove"] = ("u", true),
            ["framelog_list"] = ("o", false), ["framelog_samples"] = ("uuo", false), ["framelog_clear"] = ("u", true), ["coverage_op"] = ("uubo", false),
            ["profile_op"] = ("uubo", false), ["counters_op"] = ("ubo", false), ["call_stack"] = ("uuo", false), ["freeze_add"] = ("tuu", false),
            ["freeze_remove"] = ("u", true), ["freeze_list"] = ("o", false), ["freeze_clear"] = ("", true), ["label_add"] = ("utt", true),
            ["label_remove"] = ("tu", true), ["label_list"] = ("o", false), ["label_at"] = ("uo", false), ["label_load"] = ("to", false),
            ["label_save"] = ("o", false), ["label_clear"] = ("", true), ["dma_channels"] = ("o", false), ["dma_log_op"] = ("ubo", false),
            ["register_flow_op"] = ("ubo", false), ["coprocessor_history"] = ("uo", false), ["vectors"] = ("o", false), ["tilemap_entry"] = ("tuo", false),
            ["tile_pixels"] = ("tuuo", false), ["sprites"] = ("o", false), ["palettes"] = ("o", false), ["audio_channels"] = ("o", false),
            ["set_channel_muted"] = ("uu", true), ["evaluate"] = ("uto", false), ["symbols"] = ("uto", false), ["cheat_op"] = ("ubo", false),
            ["cheat_codecs"] = ("o", false), ["trace"] = ("uu", true), ["trace_off"] = ("u", true), ["telemetry"] = ("uo", false),
            ["tile_sheet"] = ("o", false), ["palette_swatch"] = ("o", false), ["audio_peek"] = ("o", false), ["state_save"] = ("t", true),
            ["state_load"] = ("t", true), ["state_default_path"] = ("o", false), ["events_drain"] = ("uo", false), ["open_game"] = ("tt", false),
            ["open_console"] = ("tt", false),
        };

        // The calls a change may answer with a record: made once, into a buffer that holds any answer.
        private static readonly HashSet<string> OneCall = new() { "label_load", "cheat_op", "coverage_op", "profile_op", "counters_op", "dma_log_op", "register_flow_op", "events_drain" };

        private static readonly HashSet<string> Binary = new() { "space_read", "tile_pixels", "tile_sheet", "palette_swatch", "audio_peek" };

        private readonly CrystalImpl _impl;
        private readonly CrystalExports* _x;
        private readonly nint _rt;
        private readonly IReadOnlyDictionary<string, string> _values;
        private uint _session;
        private string _route = "twin";
        private nint _subscription;

        public CrystalTwinScript(CrystalImpl impl, nint rt, IReadOnlyDictionary<string, string> values)
        {
            _impl = impl;
            _rt = rt;
            _values = values;
            _x = (CrystalExports*)NativeMemory.AllocZeroed((nuint)sizeof(CrystalExports));
            // The contract layer's own Fill, each export found by its name.
            EmuSen.Crystal.Client.Contract.Crystal.Fill(_x, name => impl.Has(name["emusen_crystal_".Length..]) ? impl.Export(name["emusen_crystal_".Length..]) : 0);
        }

        public void Dispose()
        {
            if (_subscription != 0) _x->unsubscribe(_subscription);
            NativeMemory.Free(_x);
        }

        // Every line's answer, the line first.
        public List<string> Run(string script)
        {
            var answers = new List<string>();
            foreach (string raw in script.Split('\n'))
            {
                string line = raw.Trim();
                if (line.Length == 0 || line.StartsWith('#')) continue;
                answers.Add($"{line} => {Line(line)}");
            }
            return answers;
        }

        private string Line(string line)
        {
            List<string?> words = Words(line);
            string verb = words[0]!;
            var args = words.Skip(1).ToList();
            switch (verb)
            {
                case "via":
                    _route = args[0]!;
                    return "ok";
                case "open":
                    return Opened(_impl.Open(_rt, args[0], engine: args.Count > 1 ? args[1] : null));
                case "console":
                    return Opened(_impl.Open(_rt, null, console: args[0]));
                case "use":
                    _session = uint.Parse(args[0]!);
                    return "ok";
                case "close":
                    return _impl.Call("close", _rt, _session);
                case "reset":
                    return _impl.Call("reset", _rt, _session);
                case "frames":
                    return Frames(int.Parse(args[0]!));
                case "info_of":
                    return _impl.Read("session_info", _rt, _session);
                case "subscribe":
                    _subscription = _impl.Subscribe(_rt, 0, 0);
                    return _subscription == 0 ? $"refused: {_impl.LastError()}" : "ok";
                case "drain":
                    return _impl.Drain(_subscription);
                case "log_switch":
                    return Status(_route == "twin" ? _x->log_switch(_rt, U(args[0])) : Table()->log_switch(Table()->ctx, U(args[0])));
                case "log_status":
                    return Status(_route == "twin" ? _x->log_status(_rt) : Table()->log_status(Table()->ctx));
                case "log_status_null":
                    int answered = Table()->log_status(null);
                    return answered < 0 ? $"status {answered}: {TableWords(Table())}" : answered.ToString();
                case "sessions":
                    return _route == "twin" ? _impl.Read("sessions", _rt) : CopyOut("sessions", (o, n) => Table()->sessions(Table()->ctx, o, n));
                case "games":
                    return _route == "twin" ? _impl.Read("systems", _rt) : CopyOut("games", (o, n) => Table()->games(Table()->ctx, o, n));
                case "record_start":
                    return Recorded(args[0], uint.Parse(args[1]!));
                case "record_stop":
                    return Recorded(null, 0);
                case "record_files":
                    return RecordedFiles();
                case "service":
                    nint t = _x->session_service(_rt, _session, U(args[0]));
                    return t == 0 ? $"null: {_impl.LastError()}" : "a table";
            }
            bool lengthOnly = verb.EndsWith('?');
            string name = verb.TrimEnd('?');
            if (!Signatures.TryGetValue(name, out var signature)) throw new InvalidOperationException($"no function {name}");
            return Call(name, signature.Args, signature.Int, args, lengthOnly);
        }

        private string? _recording;

        // record_start into a folder under base, or record_stop with none; the folder's time hidden.
        private string Recorded(string? at, uint stride)
        {
            byte[]? b = at is null ? null : Encoding.UTF8.GetBytes(at);
            long n;
            var buffer = new byte[4096];
            fixed (byte* bp = b)
            fixed (byte* o = buffer)
                n = at is null && stride == 0 && b is null ? _x->record_stop(_rt, _session, o, (nuint)buffer.Length) : _x->record_start(_rt, _session, bp, (nuint)(b?.Length ?? 0), stride, o, (nuint)buffer.Length);
            if (n < 0) return $"status {n}: {Hidden(_impl.LastError())}";
            _recording = Encoding.UTF8.GetString(buffer, 0, (int)n);
            return Hidden(_recording);
        }

        private static string Hidden(string text) => System.Text.RegularExpressions.Regex.Replace(text, "_[0-9]{8}_[0-9]{6}", "_<time>");

        // What the last recording left: its files by name, the ledger's lines, and each archived picture's pixels by digest.
        private string RecordedFiles()
        {
            if (_recording is null || !System.IO.Directory.Exists(_recording)) return "none";
            var names = System.IO.Directory.EnumerateFiles(_recording).Select(System.IO.Path.GetFileName).Order(StringComparer.Ordinal).ToList();
            string ledger = System.IO.File.Exists(System.IO.Path.Combine(_recording, "frames.log")) ? $"{System.IO.File.ReadAllLines(System.IO.Path.Combine(_recording, "frames.log")).Length} lines" : "no ledger";
            var pictures = new List<string>();
            string zip = System.IO.Path.Combine(_recording, "frames.zip");
            if (System.IO.File.Exists(zip))
            {
                using var archive = System.IO.Compression.ZipFile.OpenRead(zip);
                foreach (var e in archive.Entries.OrderBy(e => e.FullName, StringComparer.Ordinal))
                {
                    using var s = e.Open();
                    using var m = new System.IO.MemoryStream();
                    s.CopyTo(m);
                    using var bitmap = SkiaSharp.SKBitmap.Decode(m.ToArray());
                    using var rgba = bitmap?.Copy(SkiaSharp.SKColorType.Rgba8888);
                    pictures.Add($"{e.FullName}:{(rgba is null ? "undecoded" : $"{rgba.Width}x{rgba.Height} {Convert.ToHexString(SHA256.HashData(rgba.Bytes))[..16]}")}");
                }
            }
            return $"{string.Join(",", names)} | {ledger} | {string.Join(",", pictures)}";
        }

        private string Opened(string answer)
        {
            if (uint.TryParse(answer, out uint id)) _session = id;
            return answer;
        }

        private string Frames(int count)
        {
            string last = "none";
            for (int i = 0; i < count; i++)
            {
                var info = new CrystalFrameInfo { size = (uint)sizeof(CrystalFrameInfo) };
                int status = _x->frame(_rt, _session, &info);
                if (status != 0) return $"status {status}: {_impl.LastError()}";
                last = $"ran {info.ran} halted {info.halted} paused {info.paused} total {info.total_frames} at {info.halted_processor}:{info.halted_address:X}";
            }
            return last;
        }

        // The table the route names: the full one, or the read-only one.
        private EmuSenSessionV1* Table()
        {
            nint t = _x->session_service(_rt, _session, _route == "ro" ? CrystalAbi.ServiceReadOnly : 0);
            return t != 0 ? (EmuSenSessionV1*)t : throw new InvalidOperationException($"{_impl.Name}: no table: {_impl.LastError()}");
        }

        // The function's address and its first argument and session: the twin's, or the table's.
        private (nint Fn, nint First, uint Session, Func<string> Words) Endpoint(string name)
        {
            if (_route == "twin" || _route == "twin0")
                return (*(nint*)((byte*)_x + Marshal.OffsetOf<CrystalExports>(name)), _rt, _route == "twin0" ? 0 : _session, _impl.LastError);
            EmuSenSessionV1* t = Table();
            nint fn = *(nint*)((byte*)t + Marshal.OffsetOf<EmuSenSessionV1>(name));
            uint session = _route switch { "stranger" => 77u, "tableid" => _session, _ => 0u };
            return (fn, (nint)t->ctx, session, () => TableWords(t));
        }

        private static string TableWords(EmuSenSessionV1* t)
        {
            long n = t->last_error(t->ctx, null, 0);
            var bytes = new byte[n];
            fixed (byte* b = bytes) t->last_error(t->ctx, b, (nuint)n);
            return Encoding.UTF8.GetString(bytes);
        }

        private static uint U(string? word) => unchecked((uint)L(word));

        private static long L(string? word)
        {
            string w = word ?? "0";
            bool negative = w.StartsWith('-');
            string digits = negative ? w[1..] : w;
            long v = digits.StartsWith("0x") ? (long)Convert.ToUInt64(digits[2..], 16) : long.Parse(digits);
            return negative ? -v : v;
        }

        // A text or bytes argument: null for "-", else its UTF-8, or the hexadecimal after x:.
        private static byte[]? Bytes(string? word) =>
            word is null ? null : word.StartsWith("x:") ? Convert.FromHexString(word[2..]) : Encoding.UTF8.GetBytes(word);

        private delegate long Fill(byte* o, nuint n);

        // One call by its signature, through the route's endpoint.
        private string Call(string name, string spec, bool returnsInt, List<string?> args, bool lengthOnly)
        {
            var (fn, first, session, words) = Endpoint(name);
            if (fn == 0) return "no such function";
            var pins = new List<GCHandle>();
            var values = new List<nint>();
            int a = 0;
            try
            {
                foreach (char c in spec)
                {
                    if (c == 'o') continue;
                    string? word = a < args.Count ? args[a] : null;
                    a++;
                    switch (c)
                    {
                        case 't':
                        case 'b':
                            byte[]? bytes = Bytes(word);
                            if (bytes is null) { values.Add(0); values.Add(0); break; }
                            var h = GCHandle.Alloc(bytes, GCHandleType.Pinned);
                            pins.Add(h);
                            values.Add(bytes.Length == 0 ? (nint)1 : h.AddrOfPinnedObject());
                            values.Add(bytes.Length);
                            break;
                        default:
                            values.Add((nint)L(word));
                            break;
                    }
                }
                bool output = spec.EndsWith('o');
                if (name == "space_read")
                {
                    int len = (int)L(a < args.Count ? args[a] : "0");
                    if (lengthOnly) return $"length {Invoke(fn, first, session, spec, returnsInt, values, null, (nuint)len)}";
                    var read = new byte[len];
                    long got;
                    fixed (byte* b = read) got = Invoke(fn, first, session, spec, returnsInt, values, len == 0 ? (byte*)1 : b, (nuint)len);
                    return got < 0 ? $"status {got}: {words()}" : Shown(name, read, args);
                }
                if (!output)
                {
                    long r = Invoke(fn, first, session, spec, returnsInt, values, null, 0);
                    return r < 0 ? $"status {r}: {words()}" : r.ToString();
                }
                Fill fill = (o, n) => Invoke(fn, first, session, spec, returnsInt, values, o, n);
                if (lengthOnly)
                {
                    long n = fill(null, 0);
                    return n < 0 ? $"status {n}: {words()}" : $"length {n}";
                }
                if (OneCall.Contains(name))
                {
                    var buffer = new byte[1 << 20];
                    long n;
                    fixed (byte* b = buffer) n = fill(b, (nuint)buffer.Length);
                    // A read longer than the buffer, a coverage map, is asked again into one that holds it.
                    if (n > buffer.Length)
                    {
                        buffer = new byte[n];
                        fixed (byte* b = buffer) n = fill(b, (nuint)buffer.Length);
                    }
                    return n < 0 ? $"status {n}: {words()}" : Shown(name, buffer.AsSpan(0, (int)n).ToArray(), args);
                }
                return CopyOut(name, fill, words, args);
            }
            finally
            {
                foreach (var h in pins) h.Free();
            }
        }

        private string CopyOut(string name, Fill fill) => CopyOut(name, fill, _impl.LastError, new List<string?>());

        private string CopyOut(string name, Fill fill, Func<string> words, List<string?> args)
        {
            long n = fill(null, 0);
            if (n < 0) return $"status {n}: {words()}";
            var bytes = new byte[n];
            long again;
            fixed (byte* b = bytes) again = fill(b, (nuint)n);
            if (again < 0) return $"status {again}: {words()}";
            return again != n ? $"length {n} then {again}" : Shown(name, bytes, args);
        }

        // An answer as text, or binary as its length and its bytes or their SHA-256.
        private static string Shown(string name, byte[] bytes, List<string?> args)
        {
            bool binary = Binary.Contains(name) || (name == "coverage_op" && args.Count > 1 && args[1] == "9");
            // The hardware load is each phase's wall-clock share, compared by its names and kinds alone.
            if (name == "telemetry" && args.Count > 0 && args[0] == "7") return System.Text.RegularExpressions.Regex.Replace(Encoding.UTF8.GetString(bytes), "\"percent\":[^,}]+", "\"percent\":#");
            if (!binary) return Encoding.UTF8.GetString(bytes);
            return bytes.Length <= 64 ? $"{bytes.Length} bytes {Convert.ToHexString(bytes)}" : $"{bytes.Length} bytes {Convert.ToHexString(SHA256.HashData(bytes))}";
        }

        // The typed call each signature needs; every pointer and length travels as a native integer.
        private static long Invoke(nint fn, nint first, uint session, string spec, bool returnsInt, List<nint> v, byte* o, nuint n)
        {
            nint V(int i) => v[i];
            switch (spec, returnsInt)
            {
                case ("o", false): return ((delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, long>)fn)(first, session, o, n);
                case ("uo", false): return ((delegate* unmanaged[Cdecl]<nint, uint, uint, byte*, nuint, long>)fn)(first, session, (uint)V(0), o, n);
                case ("uuo", false): return ((delegate* unmanaged[Cdecl]<nint, uint, uint, uint, byte*, nuint, long>)fn)(first, session, (uint)V(0), (uint)V(1), o, n);
                case ("uul", false): return ((delegate* unmanaged[Cdecl]<nint, uint, uint, uint, long, long>)fn)(first, session, (uint)V(0), (uint)V(1), V(2));
                case ("tuo", false): return ((delegate* unmanaged[Cdecl]<nint, uint, nint, nuint, uint, byte*, nuint, long>)fn)(first, session, V(0), (nuint)V(1), (uint)V(2), o, n);
                case ("tub", true): return ((delegate* unmanaged[Cdecl]<nint, uint, nint, nuint, uint, nint, nuint, int>)fn)(first, session, V(0), (nuint)V(1), (uint)V(2), V(3), (nuint)V(4));
                case ("utq", true): return ((delegate* unmanaged[Cdecl]<nint, uint, uint, nint, nuint, ulong, int>)fn)(first, session, (uint)V(0), V(1), (nuint)V(2), (ulong)V(3));
                case ("utuuto", false): return ((delegate* unmanaged[Cdecl]<nint, uint, uint, nint, nuint, uint, uint, nint, nuint, byte*, nuint, long>)fn)(first, session, (uint)V(0), V(1), (nuint)V(2), (uint)V(3), (uint)V(4), V(5), (nuint)V(6), o, n);
                case ("ut", false): return ((delegate* unmanaged[Cdecl]<nint, uint, uint, nint, nuint, long>)fn)(first, session, (uint)V(0), V(1), (nuint)V(2));
                case ("uuu", true): return ((delegate* unmanaged[Cdecl]<nint, uint, uint, uint, uint, int>)fn)(first, session, (uint)V(0), (uint)V(1), (uint)V(2));
                case ("uuu", false): return ((delegate* unmanaged[Cdecl]<nint, uint, uint, uint, uint, long>)fn)(first, session, (uint)V(0), (uint)V(1), (uint)V(2));
                case ("uu", true): return ((delegate* unmanaged[Cdecl]<nint, uint, uint, uint, int>)fn)(first, session, (uint)V(0), (uint)V(1));
                case ("u", true): return ((delegate* unmanaged[Cdecl]<nint, uint, uint, int>)fn)(first, session, (uint)V(0));
                case ("utu", true): return ((delegate* unmanaged[Cdecl]<nint, uint, uint, nint, nuint, uint, int>)fn)(first, session, (uint)V(0), V(1), (nuint)V(2), (uint)V(3));
                case ("ui", true): return ((delegate* unmanaged[Cdecl]<nint, uint, uint, int, int>)fn)(first, session, (uint)V(0), (int)V(1));
                case ("tuuu", false): return ((delegate* unmanaged[Cdecl]<nint, uint, nint, nuint, uint, uint, uint, long>)fn)(first, session, V(0), (nuint)V(1), (uint)V(2), (uint)V(3), (uint)V(4));
                case ("tuu", false): return ((delegate* unmanaged[Cdecl]<nint, uint, nint, nuint, uint, uint, long>)fn)(first, session, V(0), (nuint)V(1), (uint)V(2), (uint)V(3));
                case ("uubo", false): return ((delegate* unmanaged[Cdecl]<nint, uint, uint, uint, nint, nuint, byte*, nuint, long>)fn)(first, session, (uint)V(0), (uint)V(1), V(2), (nuint)V(3), o, n);
                case ("ubo", false): return ((delegate* unmanaged[Cdecl]<nint, uint, uint, nint, nuint, byte*, nuint, long>)fn)(first, session, (uint)V(0), V(1), (nuint)V(2), o, n);
                case ("", true): return ((delegate* unmanaged[Cdecl]<nint, uint, int>)fn)(first, session);
                case ("utt", true): return ((delegate* unmanaged[Cdecl]<nint, uint, uint, nint, nuint, nint, nuint, int>)fn)(first, session, (uint)V(0), V(1), (nuint)V(2), V(3), (nuint)V(4));
                case ("tu", true): return ((delegate* unmanaged[Cdecl]<nint, uint, nint, nuint, uint, int>)fn)(first, session, V(0), (nuint)V(1), (uint)V(2));
                case ("to", false): return ((delegate* unmanaged[Cdecl]<nint, uint, nint, nuint, byte*, nuint, long>)fn)(first, session, V(0), (nuint)V(1), o, n);
                case ("tuuo", false): return ((delegate* unmanaged[Cdecl]<nint, uint, nint, nuint, uint, uint, byte*, nuint, long>)fn)(first, session, V(0), (nuint)V(1), (uint)V(2), (uint)V(3), o, n);
                case ("uto", false): return ((delegate* unmanaged[Cdecl]<nint, uint, uint, nint, nuint, byte*, nuint, long>)fn)(first, session, (uint)V(0), V(1), (nuint)V(2), o, n);
                case ("t", true): return ((delegate* unmanaged[Cdecl]<nint, uint, nint, nuint, int>)fn)(first, session, V(0), (nuint)V(1));
                case ("tt", false): return ((delegate* unmanaged[Cdecl]<nint, uint, nint, nuint, nint, nuint, long>)fn)(first, session, V(0), (nuint)V(1), V(2), (nuint)V(3));
                default: throw new InvalidOperationException($"no call for ({spec}, {returnsInt})");
            }
        }

        // A line's words: '…' verbatim, "…" with \n and \" read, - for none, $NAME and $S replaced.
        private List<string?> Words(string line)
        {
            var words = new List<string?>();
            int i = 0;
            while (i < line.Length)
            {
                if (line[i] == ' ') { i++; continue; }
                if (line[i] == '\'')
                {
                    int end = line.IndexOf('\'', i + 1);
                    words.Add(Expand(line[(i + 1)..end]));
                    i = end + 1;
                    continue;
                }
                if (line[i] == '"')
                {
                    var sb = new StringBuilder();
                    i++;
                    while (line[i] != '"')
                    {
                        if (line[i] == '\\') { i++; sb.Append(line[i] switch { 'n' => '\n', 't' => '\t', _ => line[i] }); }
                        else sb.Append(line[i]);
                        i++;
                    }
                    words.Add(Expand(sb.ToString()));
                    i++;
                    continue;
                }
                int stop = line.IndexOf(' ', i);
                if (stop < 0) stop = line.Length;
                string word = line[i..stop];
                words.Add(word == "-" ? null : Expand(word));
                i = stop;
            }
            return words;
        }

        private string Expand(string text)
        {
            string filled = text;
            foreach (var (key, value) in _values.OrderByDescending(kv => kv.Key.Length)) filled = filled.Replace("$" + key, value);
            return filled.Replace("$S", _session.ToString());
        }

        private string Status(int status) => status >= 0 ? status.ToString() : $"status {status}: {(_route == "twin" ? _impl.LastError() : TableWords(Table()))}";
    }
}
