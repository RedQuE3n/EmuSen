using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Mistress.BigPicture;
using EmuSen.WiseMan.Mistress;
using SkiaSharp;

namespace EmuSen.WiseMan.Fixtures
{
    // One theme of the fake list: its repository on a fake GitHub or GitLab, what the list states, and what the host answers - see EmuSen_BigPicture.md §25.
    public sealed class FakeTheme
    {
        public required string Name { get; init; }
        public required ThemeSource Source { get; init; }
        public string Author { get; init; } = "Nobody";
        public string Sha { get; set; } = "1111111aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        public DateTimeOffset Date { get; set; } = new(2026, 9, 20, 12, 0, 0, TimeSpan.Zero);
        public string? Readme { get; set; }
        public string? HostLicence { get; set; }
        public string Marker { get; set; } = "one";
        public IReadOnlyList<string> Variants { get; init; } = ["List", "Grid"];
        public IReadOnlyList<string> ColorSchemes { get; init; } = ["Dark", "Light", "Night"];
        public IReadOnlyList<string> AspectRatios { get; init; } = ["16:9", "16:10", "4:3"];
        public IReadOnlyList<string> FontSizes { get; init; } = ["Medium", "Large"];
        public IReadOnlyList<string> Transitions { get; init; } = [];
        public IReadOnlyList<string> Languages { get; init; } = [];
        public int Screenshots { get; init; } = 3;
        public Dictionary<string, string> Extra { get; } = new();
        public Func<byte[]>? Archive { get; set; }

        public string Top => $"{Source.Repository}-{Source.Branch}/";
        public string Url => Source.Url + ".git";
        public IEnumerable<string> Shots => Enumerable.Range(1, Screenshots).Select(i => $"screenshots/{Source.Repository}/{Source.Repository}_{i:00}.png");

        // The theme itself, zipped as the host zips a branch: one folder named for the repository and branch.
        public byte[] Zip()
        {
            if (Archive is not null) return Archive();
            using var memory = new MemoryStream();
            using (var zip = new ZipArchive(memory, ZipArchiveMode.Create, leaveOpen: true))
            {
                void Put(string name, string text)
                {
                    using var w = new StreamWriter(zip.CreateEntry(Top + name).Open());
                    w.Write(text);
                }

                zip.CreateEntry(Top);
                Put("capabilities.xml", $"<themeCapabilities><themeName>{Name}</themeName>{string.Concat(Variants.Select(v => $"<variant name=\"{v.ToLowerInvariant()}\"><label>{v}</label></variant>"))}</themeCapabilities>");
                Put("theme.xml", "<theme><view name=\"system\"><carousel name=\"c\"><pos>0 0.2</pos><size>1 0.5</size></carousel></view>" +
                                 "<view name=\"gamelist\"><textlist name=\"l\"><pos>0.05 0.1</pos><size>0.5 0.8</size></textlist></view></theme>");
                Put("marker.txt", Marker);
                Put("colors.xml", "<theme><variables><bg>101010</bg></variables></theme>");
                foreach ((string name, string text) in Extra) Put(name, text);
                if (Readme is not null) Put("README.md", Readme);
            }
            return memory.ToArray();
        }
    }

    // A fake GitLab holding ES-DE's list and its screenshots, and a fake GitHub and GitLab holding the listed themes; every request is counted and nothing reaches a network.
    public sealed class FakeThemeHosts
    {
        public readonly OnlineCoverTests.FakeServer Server;
        public readonly List<FakeTheme> Themes = new();
        public readonly List<string> NotListed = new();
        public readonly TaskCompletionSource Stalled = new(TaskCreationOptions.RunContinuationsAsynchronously);
        public string? StallArchiveOf { get; set; }
        public double? StallFraction { get; set; }
        public bool ListFails { get; set; }

        public FakeThemeHosts() => Server = new OnlineCoverTests.FakeServer { Answer = Answer };

        public int Requests => Server.Asked.Count;
        public string[] Asked => Server.Asked.ToArray();
        public int AskedFor(string address) => Server.Asked.Count(a => a == address);

        public HttpClient Client() => new(Server);

        // A download made before a UI test, off the dispatcher so its awaits cannot wait on the thread that blocks for them.
        public ThemeStamp Install(ThemeSource source) => Task.Run(async () =>
        {
            using HttpClient http = Client();
            return await ThemeDownloads.FetchAsync(http, source);
        }).GetAwaiter().GetResult();

        // Two GitHub themes and one on GitLab, each stating a different licence, or none.
        public static FakeThemeHosts Standard()
        {
            var hosts = new FakeThemeHosts();
            hosts.Themes.Add(new FakeTheme
            {
                Name = "Synthetic Book", Author = "Ada Example", Source = new ThemeSource("ada", "synthetic-book-es-de", "main"),
                Readme = "# Synthetic Book\n\n## Credits\n* Drawn by Ada\n\n## License\nCreative Commons BY-NC-SA 4.0, as stated for this synthetic theme.\n\n## More\nx\n",
                Transitions = ["Instant", "Slide"], Languages = ["en_US", "de_DE"], Screenshots = 3,
            });
            hosts.Themes.Add(new FakeTheme
            {
                Name = "Plain Shelf", Author = "Bo Sample", Source = new ThemeSource("bo", "plain-shelf-es-de", "master"),
                HostLicence = "MIT License", Readme = "# Plain Shelf\n\nNo licence section here.\n", Screenshots = 2, ColorSchemes = [], FontSizes = [],
            });
            hosts.Themes.Add(new FakeTheme
            {
                Name = "Lab Wheel", Author = "Cy Case", Source = new ThemeSource("cy-group/themes", "lab-wheel-es-de", "trunk", ThemeHost.GitLab),
                Screenshots = 1, Variants = ["Wheel"],
            });
            return hosts;
        }

        public FakeTheme this[string name] => Themes.First(t => t.Name == name);

        public string ListJson() => JsonSerializer.Serialize(new
        {
            comment = "Synthetic themes for EmuSen's tests",
            latestStableRelease = "99",
            themes = Themes.Select(t => (object)new
            {
                name = t.Name, reponame = t.Source.Repository, url = t.Url, author = t.Author, newEntry = t.Name == "Lab Wheel",
                variants = t.Variants, colorSchemes = t.ColorSchemes, fontSizes = t.FontSizes, aspectRatios = t.AspectRatios, transitions = t.Transitions, languages = t.Languages,
                screenshots = t.Shots.Select((s, i) => new { image = s, caption = $"{t.Name}, screenshot {i + 1}" }),
            }).Concat(NotListed.Select(n => (object)new { name = n, reponame = n, url = $"https://example.invalid/{n}.git", author = "x", screenshots = Array.Empty<object>() })),
            themesAndroid = new[] { new { name = "Android Only", reponame = "android-only", url = "https://github.com/x/android-only.git", author = "x" } },
        });

        private HttpResponseMessage Answer(string url)
        {
            if (url == ThemeList.Address)
                return ListFails ? new HttpResponseMessage(HttpStatusCode.ServiceUnavailable) : Text(ListJson());
            if (url.StartsWith(ThemeList.RepositoryRaw + "screenshots/", StringComparison.Ordinal))
                return Themes.SelectMany(t => t.Shots.Select((s, i) => (t, s, i))).FirstOrDefault(x => ThemeList.ScreenshotAddress(x.s) == url) is { t: not null } hit
                    ? Bytes(Picture(hit.t.Name, hit.i + 1), "image/png")
                    : new HttpResponseMessage(HttpStatusCode.NotFound);
            foreach (FakeTheme t in Themes)
            {
                ThemeSource s = t.Source;
                if (url == s.RepositoryAddress)
                    return Text(t.Source.Host == ThemeHost.GitHub
                        ? JsonSerializer.Serialize(new { default_branch = s.Branch, license = t.HostLicence is null ? null : new { key = "x", name = t.HostLicence, spdx_id = "X" } })
                        : JsonSerializer.Serialize(new { default_branch = s.Branch, license = t.HostLicence is null ? null : new { key = "x", name = t.HostLicence } }));
                if (url == s.CommitAddress)
                    return Text(s.Host == ThemeHost.GitHub
                        ? JsonSerializer.Serialize(new { sha = t.Sha, commit = new { committer = new { date = t.Date.ToString("O") } } })
                        : JsonSerializer.Serialize(new { id = t.Sha, committed_date = t.Date.ToString("O") }));
                if (url == s.ReadmeAddress)
                    return t.Readme is null ? new HttpResponseMessage(HttpStatusCode.NotFound) : Text(t.Readme);
                if (url == s.ArchiveAddress)
                {
                    if (StallArchiveOf != t.Name) return Bytes(t.Zip(), "application/zip");
                    if (StallFraction is not { } f) return new HttpResponseMessage(HttpStatusCode.OK) { Content = new StreamContent(new StallingStream(Stalled)) };
                    byte[] zip = t.Zip();
                    var content = new StreamContent(new StallingStream(Stalled, zip[..(int)(zip.Length * f)]));
                    content.Headers.ContentLength = zip.Length;
                    return new HttpResponseMessage(HttpStatusCode.OK) { Content = content };
                }
            }
            return new HttpResponseMessage(HttpStatusCode.NotFound);
        }

        private static HttpResponseMessage Text(string text) => new(HttpStatusCode.OK) { Content = new StringContent(text) };

        private static HttpResponseMessage Bytes(byte[] bytes, string type)
        {
            var content = new ByteArrayContent(bytes);
            content.Headers.ContentType = new System.Net.Http.Headers.MediaTypeHeaderValue(type);
            return new HttpResponseMessage(HttpStatusCode.OK) { Content = content };
        }

        // A synthetic screenshot: a band of colour per theme, the shot's number as bars, drawn here and nowhere else.
        public static byte[] Picture(string theme, int number)
        {
            int hue = theme.Sum(c => c * 7) % 360;
            using var bitmap = new SKBitmap(640, 360);
            using (var canvas = new SKCanvas(bitmap))
            {
                canvas.Clear(SKColor.FromHsl(hue, 45, 22));
                using var band = new SKPaint { Color = SKColor.FromHsl((hue + 40) % 360, 60, 55) };
                canvas.DrawRect(0, 250, 640, 110, band);
                using var bar = new SKPaint { Color = SKColors.White };
                for (int i = 0; i < number; i++) canvas.DrawRect(40 + i * 60, 60, 36, 140, bar);
            }
            using SKData png = bitmap.Encode(SKEncodedImageFormat.Png, 90);
            return png.ToArray();
        }

        // An archive whose body sends a part (one byte by default) and then waits until the request is cancelled.
        private sealed class StallingStream(TaskCompletionSource started, byte[]? part = null) : Stream
        {
            private bool _sent;
            private int _at;
            public override bool CanRead => true;
            public override bool CanSeek => false;
            public override bool CanWrite => false;
            public override long Length => throw new NotSupportedException();
            public override long Position { get => throw new NotSupportedException(); set => throw new NotSupportedException(); }
            public override void Flush() { }
            public override int Read(byte[] buffer, int offset, int count) => ReadAsync(buffer.AsMemory(offset, count)).AsTask().GetAwaiter().GetResult();
            public override long Seek(long offset, SeekOrigin origin) => throw new NotSupportedException();
            public override void SetLength(long value) => throw new NotSupportedException();
            public override void Write(byte[] buffer, int offset, int count) => throw new NotSupportedException();

            public override async ValueTask<int> ReadAsync(Memory<byte> buffer, CancellationToken cancel = default)
            {
                if (part is not null && _at < part.Length)
                {
                    int n = Math.Min(buffer.Length, part.Length - _at);
                    part.AsSpan(_at, n).CopyTo(buffer.Span);
                    _at += n;
                    return n;
                }
                if (part is null && !_sent)
                {
                    _sent = true;
                    buffer.Span[0] = (byte)'P';
                    return 1;
                }
                started.TrySetResult();
                await Task.Delay(Timeout.Infinite, cancel);
                return 0;
            }
        }
    }
}
