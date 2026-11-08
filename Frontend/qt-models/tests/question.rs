// EmuSen_Interface.md §15.6, §18.3: the safe answer first and focused, B's answer stated, the Resume question's words and caption.

use emusen_qt_models::help::{action_lines, entries, Screen};
use emusen_qt_models::question::{back_index, confirm, resume};

#[test]
fn a_confirming_question_puts_no_first_and_b_answers_it() {
    let q = confirm("Quit Game?", &["Progress since the last save is lost."]);
    assert_eq!(q.choices, ["No", "Yes"]);
    assert_eq!(back_index(&q), 0);
    let labels: Vec<&str> = entries(Screen::Question).iter().map(|e| e.1).collect();
    assert_eq!(labels, ["Choose", "No"]);
}

#[test]
fn the_resume_question_continues_first_names_its_setting_and_b_goes_back() {
    let q = resume("Cobalt Harbor");
    assert_eq!(q.title, "Continue where you left off?");
    assert_eq!(q.choices, ["Continue", "Start again"]);
    assert_eq!(q.sentences.len(), 1, "one sentence");
    assert!(q.sentences[0].starts_with("Cobalt Harbor "));
    assert!(q.caption.contains("Resume ◂ Ask each time ▸"));
    assert_eq!(back_index(&q), -1);
    let labels: Vec<&str> = entries(Screen::QuestionBack).iter().map(|e| e.1).collect();
    assert_eq!(labels, ["Choose", "Back"]);
}

#[test]
fn the_search_declares_y_as_the_keyboard_s_space_and_x_as_its_done() {
    let lines = action_lines(Screen::Search);
    assert!(lines.starts_with("space Search\ndone Screensaver\n"), "{lines}");
}
