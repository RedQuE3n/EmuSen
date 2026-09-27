using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Views
{
    // A console's folders in the themed gamelist: their options, their folder links in games.db, and the flatten switch - see EmuSen_Settings_Reference.md §4.67.
    public partial class MainWindow
    {
        private FolderEditorWindow? _folderEditor;

        internal FolderEditorWindow? FolderEditorShown => _folderEditor;

        // Every folder link the player set, by the folder's path on disk, to the file's full path.
        private IReadOnlyDictionary<string, string> FolderLinks()
        {
            var links = new Dictionary<string, string>(StringComparer.Ordinal);
            foreach ((string folder, IReadOnlyDictionary<string, string> edits) in _editSnapshot)
                if (edits.TryGetValue(GameMetadata.FolderLink, out string? link) && link.Length > 0)
                    links[folder] = Path.GetFullPath(Path.Combine(folder, link.Replace('/', Path.DirectorySeparatorChar)));
            return links;
        }

        // A folder's entries in ES-DE's menu: Enter Folder where a link would launch a game, and its metadata.
        private void AddFolderEntries(SceneGame folder, List<GameOption> options)
        {
            if (folder.FolderLink is not null && _editedCollection is null)
                options.Add(new GameOption("Enter Folder", () => { _themed?.EnterFolder(folder, UiClock()); ScheduleThemedFrame(); }));
            options.Add(new GameOption("Edit This Folder's Metadata", () => ShowFolderEditor(folder)));
        }

        // The games below a folder, by their paths from it, as the folder link offers them.
        private IReadOnlyList<string> FilesBelow(string folder)
        {
            string root = Path.GetFullPath(folder).TrimEnd(Path.DirectorySeparatorChar) + Path.DirectorySeparatorChar;
            return _allScan.Entries.Where(e => e.FullPath.StartsWith(root, StringComparison.Ordinal))
                .Select(e => Path.GetRelativePath(root, e.FullPath).Replace(Path.DirectorySeparatorChar, '/'))
                .Order(StringComparer.OrdinalIgnoreCase).ToList();
        }

        internal void ShowFolderEditor(SceneGame folder)
        {
            if (_folderEditor is not null || _metadataEditor is not null) return;
            string? stored = _records.Edits(folder.File).GetValueOrDefault(GameMetadata.FolderLink);
            string where = _appSettings.RomDirectory is { Length: > 0 } roms ? Path.GetRelativePath(roms, folder.File) : folder.File;
            var window = _folderEditor = new FolderEditorWindow(folder.Name, where, FilesBelow(folder.File), stored, link => SaveFolderLink(folder.File, link));
            window.Closed += (_, _) => { if (ReferenceEquals(_folderEditor, window)) _folderEditor = null; };
            _ = SheetLayer.Show(window, this);
        }

        // A folder's link is an edit of games.db on the folder's own path; nothing on disk but games.db changes.
        private void SaveFolderLink(string folder, string? link)
        {
            _records.SaveEdits(folder, new Dictionary<string, string?> { [GameMetadata.FolderLink] = link }, DateTime.Now);
            StatusText.Text = link is null ? $"{Path.GetFileName(folder)} opens as a folder again" : $"{Path.GetFileName(folder)} now launches {Path.GetFileNameWithoutExtension(link)}";
            ShowLibraryEntries();
        }
    }
}
