using System.Collections.Generic;

namespace EmuSen.Galaxia.Library.Catalogue
{
    // What the librarian knows about one image without opening it again - see EmuSen_Galaxia.md §7.
    public sealed record RomEntry(
        string Path,
        string Name,
        string System,
        long Bytes,

        // Null until something has paid for the hash; identity across renames.
        string? Md5,

        // The board the loader resolved, as its number for iNES or a name elsewhere.
        string? Board,

        // clean | unverifiable | archaic, mirroring Cartridge.HeaderTrust.
        string? HeaderTrust,

        string? Region,
        long PrgBytes,
        long ChrBytes,
        bool Playable,
        string? IndexedAt);

    // The catalogue contract. Galaxia owns this and the schema beside it; she does
    // not own the driver, because she is a leaf and a database engine is not - see
    // EmuSen_Galaxia.md §7.1 for why that split is where it is.
    public interface ICatalogue
    {
        // Everything the last scan found, newest index first.
        IReadOnlyList<RomEntry> All();

        IReadOnlyList<RomEntry> ForSystem(string system);

        // The query that has no cheap answer without an index: every image on one
        // board. Answering it by walking the tree costs a full re-parse of the
        // library, which is what this table exists to stop.
        IReadOnlyList<RomEntry> ForBoard(string system, string board);

        RomEntry? ByPath(string path);

        // Replaces what is known about one image. Idempotent on path.
        void Put(RomEntry entry);

        // One transaction for a whole scan, because a half-written catalogue that
        // looks complete is worse than an obviously missing one.
        void PutAll(IEnumerable<RomEntry> entries);

        // Rows whose file no longer exists. Never deletes anything on disk.
        IReadOnlyList<RomEntry> Missing();

        int Count { get; }
    }
}
