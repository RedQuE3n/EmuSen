using System;
using System.Linq;

namespace EmuSen.Mistress.BigPicture
{
    public enum ThemeHost { GitHub, GitLab }

    // A theme's repository on GitHub or GitLab and the branch fetched: every address Mistress asks is built here - see EmuSen_BigPicture.md §6 and §25.
    public sealed record ThemeSource(string Owner, string Repository, string Branch, ThemeHost Host = ThemeHost.GitHub)
    {
        // The ES-DE edition chosen in §10.1 (Q1); fetched only when the player asks, never bundled.
        public static ThemeSource ArtBookNext { get; } = new("anthonycaccese", "art-book-next-es-de", "main");

        private string Path => $"{Owner}/{Repository}";
        private string Project => Uri.EscapeDataString(Path);
        private string Ref => Uri.EscapeDataString(Branch);

        public string Url => Host == ThemeHost.GitHub ? $"https://github.com/{Path}" : $"https://gitlab.com/{Path}";

        public string ArchiveAddress => Host == ThemeHost.GitHub
            ? $"https://codeload.github.com/{Path}/zip/refs/heads/{Ref}"
            : $"https://gitlab.com/{Path}/-/archive/{Ref}/{Repository}-{Ref}.zip";

        public string CommitAddress => Host == ThemeHost.GitHub
            ? $"https://api.github.com/repos/{Path}/commits/{Ref}"
            : $"https://gitlab.com/api/v4/projects/{Project}/repository/commits/{Ref}";

        // The repository's own record: its default branch, and the licence the host detects.
        public string RepositoryAddress => Host == ThemeHost.GitHub
            ? $"https://api.github.com/repos/{Path}"
            : $"https://gitlab.com/api/v4/projects/{Project}?license=true";

        public string ReadmeAddress => Host == ThemeHost.GitHub
            ? $"https://raw.githubusercontent.com/{Path}/{Ref}/README.md"
            : $"https://gitlab.com/{Path}/-/raw/{Ref}/README.md";

        public ThemeSource OnBranch(string branch) => this with { Branch = branch };

        public bool SameRepository(ThemeSource other) =>
            Host == other.Host && string.Equals(Owner, other.Owner, StringComparison.OrdinalIgnoreCase) && string.Equals(Repository, other.Repository, StringComparison.OrdinalIgnoreCase);

        // A list entry's clone URL, https://github.com/<owner>/<repo>.git or https://gitlab.com/<group>/<subgroup>/<repo>.git; null for any other host.
        public static ThemeSource? FromUrl(string url, string branch = "")
        {
            if (!Uri.TryCreate(url.Trim(), UriKind.Absolute, out Uri? uri) || uri.Scheme != Uri.UriSchemeHttps) return null;
            ThemeHost? host = uri.Host.ToLowerInvariant() switch { "github.com" or "www.github.com" => ThemeHost.GitHub, "gitlab.com" or "www.gitlab.com" => ThemeHost.GitLab, _ => null };
            string[] parts = uri.AbsolutePath.Trim('/').Split('/', StringSplitOptions.RemoveEmptyEntries).Select(Uri.UnescapeDataString).ToArray();
            if (host is null || parts.Length < 2 || (host == ThemeHost.GitHub && parts.Length != 2)) return null;
            string repository = parts[^1].EndsWith(".git", StringComparison.OrdinalIgnoreCase) ? parts[^1][..^4] : parts[^1];
            if (repository.Length == 0 || parts.Any(p => p is "." or ".." || p.IndexOfAny(['\\', ':']) >= 0)) return null;
            return new ThemeSource(string.Join('/', parts[..^1]), repository, branch, host.Value);
        }
    }
}
