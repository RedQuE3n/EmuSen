using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Platform.Storage;

namespace EmuSen.LunaP.Windowing
{
    // The OS file/folder pickers and the two small modals, once, instead of per call site - see EmuSen_LunaP.md §6 and §9.4.
    public static class Dialogs
    {
        // False for cancel, for Escape, and for closing the window - anything that is not a deliberate yes.
        public static async Task<bool> ConfirmAsync(Window owner, string title, string message,
            string acceptText = "OK", string cancelText = "Cancel") =>
            await MessageWindow.Confirm(title, message, acceptText, cancelText).ShowDialog<bool>(owner);

        public static async Task ErrorAsync(Window owner, string title, string message) =>
            await MessageWindow.Notice(title, message, "Close").ShowDialog<bool>(owner);

        // Null means the user cancelled, or the control is not in a window yet.
        public static async Task<string?> PickFolderAsync(Visual owner, string title, string? startIn = null)
        {
            if (TopLevel.GetTopLevel(owner) is not { } top) return null;

            IReadOnlyList<IStorageFolder> picked = await top.StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions
            {
                Title = title,
                AllowMultiple = false,
                SuggestedStartLocation = await StartLocation(top, startIn),
            });

            return picked.Count > 0 ? picked[0].TryGetLocalPath() : null;
        }

        // The picked file's name comes back too - callers that show it want the leaf, not the whole path.
        public static async Task<(string Path, string Name)?> PickFileAsync(Visual owner, string title,
            IReadOnlyList<FilePickerFileType>? types = null, string? startIn = null)
        {
            if (TopLevel.GetTopLevel(owner) is not { } top) return null;

            IReadOnlyList<IStorageFile> picked = await top.StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions
            {
                Title = title,
                AllowMultiple = false,
                FileTypeFilter = types,
                SuggestedStartLocation = await StartLocation(top, startIn),
            });

            if (picked.Count == 0 || picked[0].TryGetLocalPath() is not { } path) return null;

            return (path, picked[0].Name);
        }

        public static async Task<string?> SaveFileAsync(Visual owner, string title, string? suggestedName = null,
            IReadOnlyList<FilePickerFileType>? types = null, string? startIn = null, string? defaultExtension = null)
        {
            if (TopLevel.GetTopLevel(owner) is not { } top) return null;

            IStorageFile? picked = await top.StorageProvider.SaveFilePickerAsync(new FilePickerSaveOptions
            {
                Title = title,
                SuggestedFileName = suggestedName,
                DefaultExtension = defaultExtension,
                FileTypeChoices = types,
                SuggestedStartLocation = await StartLocation(top, startIn),
            });

            return picked?.TryGetLocalPath();
        }

        // A path that no longer exists is not an error here; the picker just opens wherever it would have anyway.
        private static async Task<IStorageFolder?> StartLocation(TopLevel top, string? path)
        {
            if (string.IsNullOrEmpty(path)) return null;

            try
            {
                return await top.StorageProvider.TryGetFolderFromPathAsync(new Uri(path));
            }
            catch (UriFormatException)
            {
                return null;
            }
        }
    }
}
