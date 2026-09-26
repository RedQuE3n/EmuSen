using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;

namespace EmuSen.Mistress.Scraping
{
    public enum ScrapeOutcome { Found, Unknown, Error, Retry, Stopped, Missing }

    // How a run ended: its queue emptied, the quota stopped it (what is left stays queued for Resume), or the player cancelled.
    public enum ScrapeRunEnd { Done, Stopped, Cancelled }

    // What one game's turn came to; HasCover says whether the store now holds its cover.
    public sealed record ScrapeResult(string Path, string System, ScrapeOutcome Outcome, bool HasCover, IReadOnlyList<string> Written, string Detail, string? MatchedBy = null);

    // The queue's workers: each takes the next due game, identifies it as §5.2 orders, keeps its text and media, and leaves it queued when stopped - see EmuSen_BigPicture.md §17.
    public sealed class Scraper : IDisposable
    {
        // ScreenScraper's systemeid for each ES-DE system name the library's shelves carry (§5.3).
        public static readonly IReadOnlyDictionary<string, int> SystemIds = new Dictionary<string, int>(StringComparer.Ordinal)
        {
            ["nes"] = 3, ["snes"] = 4, ["gb"] = 9, ["gbc"] = 10, ["n64"] = 14,
        };

        public const int MaxAttempts = 5;

        private readonly ScreenScraperClient _client;
        private readonly MediaStore _store;
        private readonly ScrapeQuotaManager _quota;
        private readonly IScrapeClock _clock;
        private readonly Func<ScrapeChoices> _choices;
        private readonly Func<string, string, bool> _handCover;
        private readonly Func<string, Func<byte[], byte[]>?> _transform;
        private readonly Action<ScrapeResult> _done;
        private readonly CancellationTokenSource _stop = new();
        private readonly SemaphoreSlim _signal = new(0);
        private readonly HashSet<string> _held = new(StringComparer.Ordinal);
        private readonly List<Task> _workers = new();
        private bool _disposed;

        public Scraper(ScreenScraperClient client, MediaStore store, ScrapeQuotaManager quota, Func<ScrapeChoices> choices,
            Func<string, string, bool> handCover, Func<string, Func<byte[], byte[]>?> transform, Action<ScrapeResult> done, IScrapeClock? clock = null)
        {
            _client = client;
            _store = store;
            _quota = quota;
            _choices = choices;
            _handCover = handCover;
            _transform = transform;
            _done = done;
            _clock = clock ?? SystemScrapeClock.Instance;
        }

        // Called with every jeuInfos answer, for the live measurement; its Body carries credentials and must be redacted before it is kept.
        public Action<string, JeuInfosAnswer>? Answered { get; init; }

        // How long a worker with nothing due waits for another worker's game or a retry before looking again.
        public TimeSpan IdlePoll { get; init; } = TimeSpan.FromMilliseconds(100);

        public int Pending => _store.QueueLength;

        // A run is under way: started by the player, not yet emptied, stopped or cancelled.
        public bool IsRunning { get { lock (_workers) return _live > 0; } }

        // Raised once when a run's last worker ends: the queue empty, the quota stopped, or cancelled.
        public event Action<ScrapeRunEnd>? Finished;

        private int _live;
        private bool _stoppedByQuota;

        // One run over what is queued now; nothing is ever asked outside a run, and a run never starts itself - see EmuSen_BigPicture.md §17.14.
        public void Start()
        {
            lock (_workers)
            {
                if (_disposed || _live > 0) return;
                _workers.Clear();
                _stoppedByQuota = false;
                EnsureWorkers();
            }
        }

        // Up to the quota's threads, and only while a run is live.
        private void EnsureWorkers()
        {
            lock (_workers)
            {
                if (_stop.IsCancellationRequested || (_workers.Count > 0 && _live == 0)) return;
                int wanted = _quota.Threads(_choices().Threads);
                while (_workers.Count < wanted)
                {
                    int index = _workers.Count;
                    _live++;
                    _workers.Add(Task.Run(() => WorkAsync(index)));
                }
            }
        }

        // Queues a game for the next run; it asks nothing by itself.
        public bool Enqueue(string path, string system, ScrapePriority priority)
        {
            if (_disposed || !SystemIds.ContainsKey(system)) return false;
            bool queued = _store.Enqueue(path, system, priority);
            try { if (queued) _signal.Release(); }
            catch (ObjectDisposedException) { }
            return queued;
        }

        private async Task WorkAsync(int index)
        {
            CancellationToken stop = _stop.Token;
            try
            {
                while (!stop.IsCancellationRequested)
                {
                    if (_quota.Check(out _, out _) == QuotaGate.Stopped)
                    {
                        _stoppedByQuota = true;
                        break;
                    }
                    if (index >= _quota.Threads(_choices().Threads)) break;

                    QueuedGame? item;
                    bool othersBusy;
                    lock (_held)
                    {
                        item = _store.NextDue(_clock.Now, _held);
                        if (item is not null) _held.Add(item.Path);
                        othersBusy = _held.Count > (item is null ? 0 : 1);
                    }
                    if (item is null)
                    {
                        if (_store.QueueLength == 0 && !othersBusy) break;
                        if (!othersBusy && _store.EarliestRetry() is { } due && due > _clock.Now) await _clock.Delay(due - _clock.Now, stop);
                        else await IdleAsync(stop);
                        continue;
                    }

                    try
                    {
                        ScrapeResult result = await ProcessAsync(item, stop);
                        _done(result);
                    }
                    catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
                    {
                        _store.Retry(item.Path, _clock.Now + Backoff(item.Attempts), countAttempt: true);
                        _done(new ScrapeResult(item.Path, item.System, ScrapeOutcome.Retry, false, [], ScrapeRedactor.Redact(ex.Message)));
                    }
                    finally
                    {
                        lock (_held) _held.Remove(item.Path);
                    }
                    EnsureWorkers();
                }
            }
            catch (OperationCanceledException)
            {
            }
            catch (ObjectDisposedException) when (stop.IsCancellationRequested)
            {
            }
            finally
            {
                bool last;
                lock (_workers) last = --_live == 0;
                if (last) Finished?.Invoke(stop.IsCancellationRequested ? ScrapeRunEnd.Cancelled : _stoppedByQuota ? ScrapeRunEnd.Stopped : ScrapeRunEnd.Done);
            }
        }

        private Task IdleAsync(CancellationToken stop) => _signal.WaitAsync(IdlePoll, stop);

        private static TimeSpan Backoff(int attempts) => TimeSpan.FromMinutes(Math.Min(60, 1 << Math.Min(attempts, 6)));

        private async Task<ScrapeResult> ProcessAsync(QueuedGame item, CancellationToken stop)
        {
            string path = item.Path;
            if (!File.Exists(path))
            {
                _store.Dequeue(path);
                return new ScrapeResult(path, item.System, ScrapeOutcome.Missing, false, [], "the file is gone");
            }

            int systemId = SystemIds[item.System];
            var info = new FileInfo(path);
            long modified = info.LastWriteTimeUtc.Ticks;
            RomHashes? hashes = null;
            string? md5 = _store.CachedMd5(path, info.Length, modified);
            if (md5 is null)
            {
                if (info.Length > Library.CoverFetcher.HashLimitBytes) return Final(item, ScrapeState.Error, "the file is too large to hash", null, null);
                hashes = RomHashes.Of(path);
                md5 = hashes.Md5;
                _store.RememberFile(path, info.Length, modified, md5);
            }

            ScrapeChoices choices = _choices();
            bool handCover = _handCover(item.System, path);
            var kinds = choices.Kinds().Where(k => !(handCover && k == ScrapeRules.Cover)).ToList();

            // A game answered before costs no request: its media follow the file's name, and only a kind it offered and we lack is asked again.
            if (_store.Game(md5, info.Length) is { } known && known.State != ScrapeState.Error)
            {
                if (known.State == ScrapeState.Unknown)
                {
                    _store.Dequeue(path);
                    return new ScrapeResult(path, item.System, ScrapeOutcome.Unknown, false, [], "known to be unknown");
                }
                List<string> kept = FollowRename(md5, info.Length, item.System, path);
                string[] offered = (known.Offered ?? "").Split(',', StringSplitOptions.RemoveEmptyEntries);
                IReadOnlyList<StoredMedia> stored = _store.Media(md5, info.Length);
                if (!kinds.Any(k => offered.Contains(k.EsdeType) && !stored.Any(s => s.Type == k.EsdeType)))
                {
                    _store.Dequeue(path);
                    return new ScrapeResult(path, item.System, ScrapeOutcome.Found, HasCover(md5, info.Length, item.System, path), kept, "already scraped", known.MatchedBy);
                }
            }

            if (!await _quota.TakeTurnAsync(stop)) return Stopped(item);
            hashes ??= RomHashes.Of(path);
            string fileName = Path.GetFileName(path);
            JeuInfosAnswer answer = await AskAsync(path, systemId, hashes, fileName, stop);
            string matchedBy = "file";

            // Step (2) of §5.2: only after a 404, and only when the core's transform changes the bytes.
            if (answer.Status == ScrapeStatus.NotFound && _transform(path) is { } transform)
            {
                byte[] bytes = await File.ReadAllBytesAsync(path, stop);
                byte[] changed = transform(bytes);
                if (!changed.AsSpan().SequenceEqual(bytes))
                {
                    if (!await _quota.TakeTurnAsync(stop)) return Stopped(item);
                    JeuInfosAnswer second = await AskAsync(path, systemId, RomHashes.Of(changed), fileName, stop);
                    if (second.Status != ScrapeStatus.NotFound) answer = second;
                    if (second.Status == ScrapeStatus.Found) matchedBy = "transformed";
                }
            }

            switch (answer.Status)
            {
                case ScrapeStatus.Found:
                    return await KeepAsync(item, md5, info.Length, answer.Game!, choices, kinds, matchedBy, stop);
                case ScrapeStatus.NotFound:
                    return Final(item, ScrapeState.Unknown, "not in ScreenScraper", md5, info.Length);
                case ScrapeStatus.TooManyRequests or ScrapeStatus.ServerBusy:
                    _store.Retry(path, _clock.Now + (answer.Status == ScrapeStatus.ServerBusy ? ScrapeQuotaManager.BusyWait : ScrapeQuotaManager.TooManyWait), countAttempt: false);
                    return new ScrapeResult(path, item.System, ScrapeOutcome.Retry, false, [], answer.Detail);
                case ScrapeStatus.DailyQuota or ScrapeStatus.DailyKoQuota or ScrapeStatus.BadCredentials or ScrapeStatus.ApiClosed or ScrapeStatus.Blacklisted:
                    return Stopped(item);
                default:
                    if (item.Attempts + 1 >= MaxAttempts || answer.Status == ScrapeStatus.BadRequest)
                        return Final(item, ScrapeState.Error, $"{answer.Status}: {answer.Detail}", md5, info.Length);
                    _store.Retry(path, _clock.Now + Backoff(item.Attempts), countAttempt: true);
                    return new ScrapeResult(path, item.System, ScrapeOutcome.Retry, false, [], $"{answer.Status}: {answer.Detail}");
            }
        }

        private async Task<JeuInfosAnswer> AskAsync(string path, int systemId, RomHashes hashes, string fileName, CancellationToken stop)
        {
            JeuInfosAnswer answer = await _client.JeuInfosAsync(systemId, hashes, fileName, stop);
            _quota.Observe(answer.Quota);
            if (answer.Status is ScrapeStatus.Found or ScrapeStatus.NotFound or ScrapeStatus.Malformed or ScrapeStatus.BadRequest)
                _quota.Counted(answer.Status == ScrapeStatus.NotFound);
            _quota.Answered(answer.Status);
            Answered?.Invoke(path, answer);
            return answer;
        }

        private ScrapeResult Stopped(QueuedGame item)
        {
            _quota.Check(out _, out string? why);
            return new ScrapeResult(item.Path, item.System, ScrapeOutcome.Stopped, false, [], why ?? "stopped");
        }

        private ScrapeResult Final(QueuedGame item, ScrapeState state, string detail, string? md5, long? bytes)
        {
            if (md5 is not null && bytes is long b)
                _store.Record(new ScrapedRecord(md5, b, state) { Detail = ScrapeRedactor.Redact(detail), FetchedAt = _clock.Now.UtcDateTime });
            _store.Dequeue(item.Path);
            return new ScrapeResult(item.Path, item.System, state == ScrapeState.Unknown ? ScrapeOutcome.Unknown : ScrapeOutcome.Error, false, [], ScrapeRedactor.Redact(detail));
        }

        // The text of §3.7's variants and the media of each wanted kind, by the region and language rules of §5.4.
        private async Task<ScrapeResult> KeepAsync(QueuedGame item, string md5, long bytes, ScrapedGame game, ScrapeChoices choices,
            IReadOnlyList<ScrapeMediaKind> kinds, string matchedBy, CancellationToken stop)
        {
            string path = item.Path;
            string stem = Path.GetFileNameWithoutExtension(path);
            IReadOnlyList<string> regions = ScrapeRules.RegionOrder(Path.GetFileName(path), choices.Region, choices.RegionFallback);
            IReadOnlyList<string> languages = ScrapeRules.LanguageOrder(choices.Language);
            string offered = string.Join(",", ScrapeRules.AllKinds.Where(k => ScrapeRules.ChooseMedia(game.Media, k, regions, fallback: true) is not null).Select(k => k.EsdeType));
            DateTimeOffset now = _clock.Now;

            _store.Record(new ScrapedRecord(md5, bytes, ScrapeState.Found)
            {
                GameId = game.Id, RomId = game.RomId, SystemId = game.SystemId ?? SystemIds[item.System],
                Name = ScrapeRules.ChooseText(game.Names, [.. regions, "ss"], fallback: true)?.Text,
                Description = ScrapeRules.ChooseText(game.Synopses, languages, choices.RegionFallback)?.Text,
                Developer = game.Developer, Publisher = game.Publisher, Genre = ScrapeRules.Genre(game.Genres, languages), Players = game.Players,
                Rating = ScrapeRules.Rating(game.Note), ReleaseDate = ScrapeRules.Date(ScrapeRules.ChooseText(game.Dates, regions, fallback: true)?.Text),
                Region = regions[0], Language = languages[0], MatchedBy = matchedBy, Offered = offered, FetchedAt = now.UtcDateTime,
            });

            var written = new List<string>();
            IReadOnlyList<StoredMedia> stored = _store.Media(md5, bytes);
            foreach (ScrapeMediaKind kind in kinds)
            {
                if (stored.Any(s => s.Type == kind.EsdeType && File.Exists(Path.Combine(_store.Root, s.RelativePath)))) continue;
                if (ScrapeRules.ChooseMedia(game.Media, kind, regions, choices.RegionFallback) is not { } media) continue;
                string relative = Path.Combine(item.System, kind.Folder, stem + ScrapeRules.Extension(media));
                if (!await _quota.TakeTurnAsync(stop)) break;
                MediaAnswer got = await _client.DownloadAsync(media.Url, Path.Combine(_store.Root, relative), stop);
                await ThrottleAsync(got, stop);
                if (got.Outcome is not (MediaOutcome.Saved or MediaOutcome.AlreadyThere)) continue;
                _store.RecordMedia(md5, bytes, new StoredMedia(kind.EsdeType, relative, media.Region, got.Sha1), _clock.Now);
                written.Add(relative);
            }

            _store.Dequeue(path);
            return new ScrapeResult(path, item.System, ScrapeOutcome.Found, HasCover(md5, bytes, item.System, path), written, "", matchedBy);
        }

        // The member's maxdownloadspeed, in KB/s, held by waiting after a file that came faster.
        private async Task ThrottleAsync(MediaAnswer got, CancellationToken stop)
        {
            if (got.Outcome != MediaOutcome.Saved || _quota.Limits?.MaxDownloadKBps is not int speed || speed <= 0) return;
            TimeSpan owed = TimeSpan.FromSeconds(got.Bytes / (speed * 1024.0)) - got.Took;
            if (owed > TimeSpan.Zero) await _clock.Delay(owed, stop);
        }

        private bool HasCover(string md5, long bytes, string system, string path) =>
            _store.Media(md5, bytes).Any(m => m.Type == ScrapeRules.Cover.EsdeType && File.Exists(Path.Combine(_store.Root, m.RelativePath)));

        // A renamed file takes its media to its new name; a copy of the same game beside the old one gets copies, so the old file keeps its own.
        private List<string> FollowRename(string md5, long bytes, string system, string path)
        {
            var moved = new List<string>();
            string stem = Path.GetFileNameWithoutExtension(path);
            foreach (StoredMedia media in _store.Media(md5, bytes))
            {
                string old = Path.Combine(_store.Root, media.RelativePath);
                string relative = Path.Combine(Path.GetDirectoryName(media.RelativePath)!, stem + Path.GetExtension(media.RelativePath));
                string target = Path.Combine(_store.Root, relative);
                if (relative == media.RelativePath || !File.Exists(old) || File.Exists(target)) continue;
                string oldStem = Path.GetFileNameWithoutExtension(media.RelativePath);
                bool oldStillThere = _store.PathsOf(md5, bytes).Any(p => p != path && Path.GetFileNameWithoutExtension(p) == oldStem && File.Exists(p));
                if (oldStillThere) File.Copy(old, target, overwrite: false);
                else File.Move(old, target, overwrite: false);
                _store.RecordMedia(md5, bytes, media with { RelativePath = relative }, _clock.Now);
                moved.Add(relative);
            }
            return moved;
        }

        public void Dispose()
        {
            if (_disposed) return;
            _disposed = true;
            _stop.Cancel();
            Task[] workers;
            lock (_workers) workers = _workers.ToArray();
            try { Task.WaitAll(workers, TimeSpan.FromSeconds(5)); }
            catch (AggregateException) { }
            _stop.Dispose();
            _signal.Dispose();
        }
    }
}
