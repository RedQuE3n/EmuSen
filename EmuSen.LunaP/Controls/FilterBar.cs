using System;
using System.Collections;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Input;

namespace EmuSen.LunaP.Controls
{
    // A search box, optionally preceded by a labelled facet dropdown - see EmuSen_LunaP.md §14.2.
    public class FilterBar : TemplatedControl
    {
        public static readonly StyledProperty<string> SearchTextProperty =
            AvaloniaProperty.Register<FilterBar, string>(nameof(SearchText), string.Empty,
                defaultBindingMode: Avalonia.Data.BindingMode.TwoWay);

        public static readonly StyledProperty<string> PlaceholderProperty =
            AvaloniaProperty.Register<FilterBar, string>(nameof(Placeholder), "Search");

        public static readonly StyledProperty<string> FacetLabelProperty =
            AvaloniaProperty.Register<FilterBar, string>(nameof(FacetLabel), string.Empty);

        public static readonly StyledProperty<bool> ShowFacetProperty =
            AvaloniaProperty.Register<FilterBar, bool>(nameof(ShowFacet));

        private TextBox? _search;
        private Dropdown? _facet;
        private IEnumerable? _pendingFacets;
        private object? _pendingSelection;

        // Raised whenever the search text or the facet changes, from any cause.
        public event Action? Changed;

        // Raised for Enter in the search box, which the library uses to launch the first match.
        public event Action? Submitted;

        public string SearchText
        {
            get => GetValue(SearchTextProperty);
            set => SetValue(SearchTextProperty, value);
        }

        public string Placeholder
        {
            get => GetValue(PlaceholderProperty);
            set => SetValue(PlaceholderProperty, value);
        }

        // "Console:" and the like; only shown alongside the dropdown.
        public string FacetLabel
        {
            get => GetValue(FacetLabelProperty);
            set => SetValue(FacetLabelProperty, value);
        }

        public bool ShowFacet
        {
            get => GetValue(ShowFacetProperty);
            set => SetValue(ShowFacetProperty, value);
        }

        public object? Facet => _facet?.SelectedItem ?? _pendingSelection;

        // Held until the template exists, so a caller can fill the facets from its constructor.
        public void SetFacets(IEnumerable items, object? selected)
        {
            _pendingFacets = items;
            _pendingSelection = selected;
            _facet?.Fill(items, selected);
        }

        protected override void OnApplyTemplate(TemplateAppliedEventArgs e)
        {
            base.OnApplyTemplate(e);

            if (_search is not null)
            {
                _search.PropertyChanged -= OnSearchPropertyChanged;
                _search.KeyDown -= OnSearchKeyDown;
            }
            if (_facet is not null) _facet.Chose -= OnFacetChose;

            _search = e.NameScope.Find<TextBox>("PART_Search");
            _facet = e.NameScope.Find<Dropdown>("PART_Facet");

            if (_search is not null)
            {
                // The property, not the TextChanged event - only this reacts to a Text set that did not come from typing.
                _search.PropertyChanged += OnSearchPropertyChanged;
                _search.KeyDown += OnSearchKeyDown;
            }

            if (_facet is not null)
            {
                _facet.Chose += OnFacetChose;
                if (_pendingFacets is not null) _facet.Fill(_pendingFacets, _pendingSelection);
            }
        }

        private void OnSearchPropertyChanged(object? sender, AvaloniaPropertyChangedEventArgs e)
        {
            if (e.Property != TextBox.TextProperty) return;

            SetCurrentValue(SearchTextProperty, _search?.Text ?? "");
            Changed?.Invoke();
        }

        private void OnSearchKeyDown(object? sender, KeyEventArgs e)
        {
            if (e.Key != Key.Enter) return;

            e.Handled = true;
            Submitted?.Invoke();
        }

        private void OnFacetChose(object? _)
        {
            _pendingSelection = _facet?.SelectedItem;
            Changed?.Invoke();
        }

        public void FocusSearch() => _search?.Focus();

        // The match every caller wants: case-insensitive, and an empty search matches everything.
        public static bool Matches(string? search, string? candidate) =>
            string.IsNullOrWhiteSpace(search)
            || (candidate ?? "").Contains(search.Trim(), StringComparison.OrdinalIgnoreCase);
    }
}
