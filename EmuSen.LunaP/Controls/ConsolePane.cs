using System;
using System.Collections.Generic;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Input;

namespace EmuSen.LunaP.Controls
{
    // A terminal-shaped pane: scrolling output, a prompt, an input box with history recall. Knows nothing about DianaOS - see EmuSen_LunaP.md §5.6.
    public class ConsolePane : TemplatedControl
    {
        public static readonly StyledProperty<string> PromptProperty =
            AvaloniaProperty.Register<ConsolePane, string>(nameof(Prompt), string.Empty);

        private TextBox? _input;
        private SelectableTextBlock? _output;
        private ScrollViewer? _scroll;

        // The output is held here, not in the TextBlock: callers print a welcome banner from their constructor, long before a template exists.
        private string _text = "";

        // -1 means "not recalling, editing whatever is live in the box" - the algorithm DianaOS's own ConsoleLineReader uses.
        private int _historyIndex = -1;
        private string _pendingInputText = "";

        // Raised on Enter, with the line as typed; the caller decides what running it means.
        public event Action<string>? Submitted;

        // Oldest-first. A delegate because one caller reads a live core's history and the other its own interpreter's.
        public Func<IReadOnlyList<string>>? HistorySource { get; set; }

        public string Prompt
        {
            get => GetValue(PromptProperty);
            set => SetValue(PromptProperty, value);
        }

        public string OutputText => _text;

        protected override void OnApplyTemplate(TemplateAppliedEventArgs e)
        {
            base.OnApplyTemplate(e);

            if (_input is not null) _input.KeyDown -= OnInputKeyDown;

            _input = e.NameScope.Find<TextBox>("PART_Input");
            _output = e.NameScope.Find<SelectableTextBlock>("PART_Output");
            _scroll = e.NameScope.Find<ScrollViewer>("PART_Scroll");

            if (_input is not null) _input.KeyDown += OnInputKeyDown;
            Flush();
        }

        public void AppendLine(string text)
        {
            _text = _text.Length == 0 ? text : _text + "\n" + text;
            Flush();
            _scroll?.ScrollToEnd();
        }

        public void Clear()
        {
            _text = string.Empty;
            Flush();
        }

        private void Flush()
        {
            if (_output is not null) _output.Text = _text;
        }

        public void FocusInput() => _input?.Focus();

        // Resets recall too: after a target swap the old history is gone and a half-finished recall would point at nothing.
        public void ResetHistoryRecall() => _historyIndex = -1;

        private void OnInputKeyDown(object? sender, KeyEventArgs e)
        {
            switch (e.Key)
            {
                case Key.Enter:
                    e.Handled = true;
                    string line = _input?.Text ?? "";
                    if (_input is not null) _input.Text = "";
                    _historyIndex = -1;
                    Submitted?.Invoke(line);
                    break;

                case Key.Up:
                    e.Handled = true;
                    RecallHistory(older: true);
                    break;

                case Key.Down:
                    e.Handled = true;
                    RecallHistory(older: false);
                    break;
            }
        }

        private void RecallHistory(bool older)
        {
            IReadOnlyList<string> entries = HistorySource?.Invoke() ?? Array.Empty<string>();
            if (entries.Count == 0 || _input is null) return;

            if (older)
            {
                if (_historyIndex == -1)
                {
                    _pendingInputText = _input.Text ?? "";
                    _historyIndex = entries.Count - 1;
                }
                else if (_historyIndex > 0)
                {
                    _historyIndex--;
                }
            }
            else
            {
                if (_historyIndex == -1) return;

                if (_historyIndex < entries.Count - 1)
                {
                    _historyIndex++;
                }
                else
                {
                    _historyIndex = -1;
                    SetInputText(_pendingInputText);
                    return;
                }
            }

            SetInputText(entries[_historyIndex]);
        }

        private void SetInputText(string text)
        {
            if (_input is null) return;

            _input.Text = text;
            _input.CaretIndex = text.Length;
        }
    }
}
