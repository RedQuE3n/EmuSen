using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;

namespace EmuSen.Mistress.Library
{
    public enum CoverOutcome { Found, Unknown, NoArt, Failed }

    // What one lookup came to; Saved is the picture written, when there is one.
    public sealed record CoverResult(string RomPath, string Console, CoverOutcome Outcome, string? RomName, string? Saved, string? Detail);

    // Missing covers, one game at a time on one worker: OpenVGDB names it, libretro's thumbnail server has its box - see EmuSen_Settings_Reference.md §4.39.
    public sealed class CoverFetcher : IDisposable
    {
        public const string ThumbnailServer = "https://thumbnails.libretro.com/";

        // ES-DE's default ceiling for hashing a file, so an enormous image is looked up by name only.
        public const long HashLimitBytes = 384L << 20;

        private static readonly char[] Unsafe = { '&', '*', '/', ':', '`', '<', '>', '?', '\\', '|', '"' };

        private readonly HttpClient _http;
        private readonly string _databasePath;
        private readonly string _artworkDirectory;
        private readonly Action<CoverResult> _done;
        private readonly BlockingCollection<(RomEntry Game, CoreDescriptor Core)> _queue = new();
        private readonly ConcurrentDictionary<string, byte> _asked = new(StringComparer.Ordinal);
        private readonly CancellationTokenSource _stop = new();
        private readonly Task _worker;

        // Between requests to either server; one lookup a quarter second is far under anything a person scrolling would ask for.
        public TimeSpan Spacing { get; init; } = TimeSpan.FromMilliseconds(250);

        public CoverFetcher(HttpClient http, string databasePath, string artworkDirectory, Action<CoverResult> done)
        {
            _http = http;
            _databasePath = databasePath;
            _artworkDirectory = artworkDirectory;
            _done = done;
            _worker = Task.Factory.StartNew(Run, TaskCreationOptions.LongRunning);
        }

        public int Pending => _queue.Count;

        // Once per game per session; the caller's record of past outcomes decides whether to ask at all.
        public bool Enqueue(RomEntry game, CoreDescriptor core)
        {
            if (_stop.IsCancellationRequested || !_asked.TryAdd(game.FullPath, 0)) return false;
            _queue.Add((game, core));
            return true;
        }

        public void Forget(string romPath) => _asked.TryRemove(romPath, out _);

        private void Run()
        {
            OpenVgdb? database = null;
            try
            {
                foreach ((RomEntry game, CoreDescriptor core) in _queue.GetConsumingEnumerable(_stop.Token))
                {
                    database ??= OpenVgdb.Open(_databasePath);
                    CoverResult result;
                    try
                    {
                        result = database is null
                            ? new CoverResult(game.FullPath, core.Console, CoverOutcome.Failed, null, null, "OpenVGDB is not downloaded")
                            : LookUp(database, game, core);
                    }
                    catch (Exception ex) when (ex is IOException or HttpRequestException or TaskCanceledException or UnauthorizedAccessException)
                    {
                        result = new CoverResult(game.FullPath, core.Console, CoverOutcome.Failed, null, null, ex.Message);
                    }
                    _done(result);
                }
            }
            catch (OperationCanceledException)
            {
            }
            finally
            {
                database?.Dispose();
            }
        }

        private CoverResult LookUp(OpenVgdb database, RomEntry game, CoreDescriptor core)
        {
            string stem = Path.GetFileNameWithoutExtension(game.FullPath);
            OpenVgdbMatch? match = database.ByName(stem, core.OpenVgdbSystemNames);
            if (match is null && core.OpenVgdbBytes is { } bytesOf && new FileInfo(game.FullPath).Length <= HashLimitBytes)
                match = database.ByMd5(Convert.ToHexString(System.Security.Cryptography.MD5.HashData(bytesOf(File.ReadAllBytes(game.FullPath)))), core.OpenVgdbSystemNames);
            if (match is null) return new CoverResult(game.FullPath, core.Console, CoverOutcome.Unknown, null, null, null);

            string target = Path.Combine(_artworkDirectory, core.Console, Safe(stem) + ".png");
            if (File.Exists(target)) return new CoverResult(game.FullPath, core.Console, CoverOutcome.Found, match.RomName, target, "already there");

            foreach (string name in Names(match.RomName))
                foreach (string system in core.CheatSystemNames)
                {
                    string url = ThumbnailServer + Uri.EscapeDataString(system) + "/Named_Boxarts/" + Uri.EscapeDataString(Safe(name)) + ".png";
                    if (Fetch(url, target) is string saved) return new CoverResult(game.FullPath, core.Console, CoverOutcome.Found, match.RomName, saved, url);
                }

            // OpenEmu's own address, tried last: its host refused every request measured on 2026-09-21.
            if (match.CoverUrl is string cover && Uri.TryCreate(cover, UriKind.Absolute, out Uri? address) && address.Scheme is "https" or "http")
            {
                string fallback = Path.ChangeExtension(target, Path.GetExtension(address.AbsolutePath) is ".jpg" or ".jpeg" ? ".jpg" : ".png");
                if (Fetch(cover, fallback) is string saved) return new CoverResult(game.FullPath, core.Console, CoverOutcome.Found, match.RomName, saved, cover);
            }
            return new CoverResult(game.FullPath, core.Console, CoverOutcome.NoArt, match.RomName, null, null);
        }

        // Only an image that decodes as one is kept, written beside its final name and moved into place.
        private string? Fetch(string url, string target)
        {
            _stop.Token.WaitHandle.WaitOne(Spacing);
            _stop.Token.ThrowIfCancellationRequested();
            using HttpResponseMessage response = _http.GetAsync(url, _stop.Token).GetAwaiter().GetResult();
            if (response.StatusCode != HttpStatusCode.OK) return null;
            if (response.Content.Headers.ContentType?.MediaType is not string type || !type.StartsWith("image/", StringComparison.Ordinal)) return null;
            byte[] picture = response.Content.ReadAsByteArrayAsync(_stop.Token).GetAwaiter().GetResult();
            if (picture.Length < 80) return null;

            Directory.CreateDirectory(Path.GetDirectoryName(target)!);
            string temp = target + ".part";
            File.WriteAllBytes(temp, picture);
            File.Move(temp, target, overwrite: false);
            return target;
        }

        // The exact name, then with trailing tags dropped one at a time, keeping the first: libretro keeps a revision's box under the plain name.
        public static IEnumerable<string> Names(string romName)
        {
            var tags = System.Text.RegularExpressions.Regex.Matches(romName, @"\s*\([^)]*\)");
            yield return romName;
            for (int keep = tags.Count - 1, tries = 1; keep >= 1 && tries < 3; keep--, tries++)
                yield return romName[..(tags[keep].Index)];
        }

        // libretro-thumbnails' substitution, the same as the art index's.
        public static string Safe(string name) => string.Concat(name.Select(c => Array.IndexOf(Unsafe, c) >= 0 ? '_' : c));

        public void Dispose()
        {
            _stop.Cancel();
            _queue.CompleteAdding();
            try { _worker.Wait(TimeSpan.FromSeconds(2)); } catch (AggregateException) { }
            _stop.Dispose();
        }
    }
}
