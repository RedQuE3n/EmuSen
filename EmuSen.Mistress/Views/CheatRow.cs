using System;
using System.ComponentModel;
using System.Runtime.CompilerServices;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.Mistress.Views
{
    // One row of the Active Cheats list. Its checkbox writes straight
    // through to the registry - see EmuSen_Settings_Reference.md §4.14.
    public sealed class CheatRow : INotifyPropertyChanged
    {
        private readonly CheatRegistry _registry;
        private bool _enabled;

        public CheatRow(CheatRegistry registry, CheatInfo cheat, string detail)
        {
            _registry = registry;
            Id = cheat.Id;
            _enabled = cheat.Enabled;
            Description = string.IsNullOrWhiteSpace(cheat.Description) ? $"cheat #{cheat.Id}" : cheat.Description;
            Kind = cheat.Kind == CheatKind.RamPoke ? "RAM" : "ROM";
            Detail = detail;
        }

        public int Id { get; }
        public string Description { get; }
        public string Kind { get; }
        public string Detail { get; }

        public bool Enabled
        {
            get => _enabled;
            set
            {
                if (_enabled == value) return;
                _enabled = value;
                _registry.SetEnabled(Id, value);
                OnPropertyChanged();
            }
        }

        public event PropertyChangedEventHandler? PropertyChanged;

        private void OnPropertyChanged([CallerMemberName] string? name = null) =>
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(name));
    }
}
