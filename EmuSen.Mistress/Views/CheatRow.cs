namespace EmuSen.Mistress.Views
{
    // One row of the Active Cheats table; its checkbox writes straight through to the list - see EmuSen_Settings_Reference.md §4.14.
    public sealed class CheatRow
    {
        private readonly CheatList _list;

        public CheatRow(CheatList list, CheatLine cheat)
        {
            _list = list;
            Id = cheat.Id;
            _enabled = cheat.Enabled;
            Description = string.IsNullOrWhiteSpace(cheat.Description) ? $"cheat #{cheat.Id}" : cheat.Description;
            Kind = cheat.RomPatch ? "ROM" : "RAM";
            Detail = cheat.Detail;
        }

        public int Id { get; }
        public string Description { get; }
        public string Kind { get; }
        public string Detail { get; }

        private bool _enabled;

        public bool Enabled
        {
            get => _enabled;
            set
            {
                if (_enabled == value) return;
                _enabled = value;
                _list.SetEnabled(Id, value);
            }
        }
    }
}
