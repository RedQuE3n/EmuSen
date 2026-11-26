//! `SettingsModel`, the Settings screen as a QAbstractListModel (EmuSen_Qt.md §7.7): settings.rs's rows of the section
//! or page shown, with the roles the kit's settings frame reads, and the focused row's pane as properties. A read runs
//! off the GUI thread and comes back through qt_thread().queue; a change is saved through Galaxia's model at once.

use crate::consoles_source::{ConsolesClients, ScriptedConsoles};
use crate::eclipse_list_model::json_to_variant;
use crate::help::Screen;
use crate::help_model::shown_screen;
use crate::settings::{self, Model, Outcome, SettingsView};
use crate::settings_source::ClientSettings;
use emusen_platform_client::galaxia::consts::{FILE_GRAPHICS, FILE_SETTINGS, MODEL_APP_SETTINGS, MODEL_GRAPHICS_CONFIG};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QList, QModelIndex, QString, QStringList, QVariant, QVector};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!(<QtCore/QAbstractListModel>);
        type QAbstractListModel = crate::library_model::qobject::QAbstractListModel;
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qbytearray.h");
        type QByteArray = cxx_qt_lib::QByteArray;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qvector.h");
        type QVector_i32 = cxx_qt_lib::QVector<i32>;
        include!("emusen-kit-qt/src/kinds.cxxqt.h");
        #[namespace = "Pad"]
        type PadButton = emusen_kit_qt::kinds::ffi::PadButton;
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        /// The platform library, through which the settings are read and saved.
        #[qproperty(QString, platform_library, cxx_name = "platformLibrary")]
        /// The runtime's cores folder, as the library's; empty for the installation's own.
        #[qproperty(QString, cores)]
        /// A file holding the consoles' world in place of the runtime's, as the recordings script theirs; empty for the runtime.
        #[qproperty(QString, world)]
        /// Whether Settings is shown: the settings are read each time it opens.
        #[qproperty(bool, open)]
        /// The program's facts a pane names, as LibraryModel.settingsFacts writes them.
        #[qproperty(QString, facts)]
        #[qproperty(QStringList, sections, READ, CONSTANT)]
        /// The section shown, which the strip's shoulders and clicks choose.
        #[qproperty(i32, section)]
        /// The pages open under the section, the section's name first; empty on the section.
        #[qproperty(QStringList, trail, READ, NOTIFY)]
        /// The focused row, which the frame's walk sets; the pane is its.
        #[qproperty(i32, focused)]
        /// The focused row's sentence, or the bindings page's, which has no row (EmuSen_Qt.md §7.7.H).
        #[qproperty(QString, sentence, READ, NOTIFY)]
        #[qproperty(QStringList, pane_lines, cxx_name = "paneLines", READ, NOTIFY)]
        #[qproperty(QStringList, pane_muted, cxx_name = "paneMuted", READ, NOTIFY)]
        /// The tone (Kit.Tones) of the focused row's sentence; `lineTone` gives each further line's.
        #[qproperty(i32, sentence_tone, cxx_name = "sentenceTone", READ, NOTIFY)]
        /// The pane's preview: 0 none, 1 the game's picture, 2 a meter, 3 the two variants, 4 the text sizes.
        #[qproperty(i32, preview, READ, NOTIFY)]
        #[qproperty(QString, caption, READ, NOTIFY)]
        #[qproperty(f64, meter, READ, NOTIFY)]
        /// The text sizes' lines, or the two variants' names; `sampleScale` gives each line's scale.
        #[qproperty(QStringList, samples, READ, NOTIFY)]
        /// What the settings ask of the whole program, once read: the text size (its word, which the kit's tokens scale
        /// by), the light variant, reduced motion.
        #[qproperty(bool, ready, READ, NOTIFY)]
        #[qproperty(QString, text_size, cxx_name = "textSize", READ, NOTIFY)]
        #[qproperty(bool, light, READ, NOTIFY)]
        #[qproperty(bool, reduced_motion, cxx_name = "reducedMotion", READ, NOTIFY)]
        /// And of the pads (EmuSen_Qt.md §7.7.H3): Swap A and B in menus, Controller type's family (Kit.Families; -1 for
        /// Automatic), Controller notices and Rumble; the rest the pads' hand-over is given (`gamePads`).
        #[qproperty(bool, swap_buttons, cxx_name = "swapButtons", READ, NOTIFY)]
        #[qproperty(i32, controller_family, cxx_name = "controllerFamily", READ, NOTIFY)]
        #[qproperty(bool, controller_notices, cxx_name = "controllerNotices", READ, NOTIFY)]
        #[qproperty(bool, rumble, READ, NOTIFY)]
        /// And of a running game and its window (EmuSen_Qt.md §1.8.6): the game's Volume, Output as stored, Sync to
        /// display, Pause in background, and the frame rate, messages and bar over the game.
        #[qproperty(f64, volume, READ, NOTIFY = play_changed)]
        #[qproperty(QString, output, READ, NOTIFY = play_changed)]
        #[qproperty(bool, sync_to_display, cxx_name = "syncToDisplay", READ, NOTIFY = play_changed)]
        #[qproperty(bool, pause_in_background, cxx_name = "pauseInBackground", READ, NOTIFY = play_changed)]
        #[qproperty(bool, show_fps, cxx_name = "showFps", READ, NOTIFY = play_changed)]
        #[qproperty(bool, show_status_text, cxx_name = "showStatusText", READ, NOTIFY = play_changed)]
        #[qproperty(bool, show_status_bar, cxx_name = "showStatusBar", READ, NOTIFY = play_changed)]
        /// How loud the navigation sounds play, a share of full (Sound volume), and Play time counted up to, in hours.
        #[qproperty(f64, navigation_volume, cxx_name = "navigationVolume", READ, NOTIFY)]
        #[qproperty(i32, play_hours, cxx_name = "playHours", READ, NOTIFY)]
        /// Whether Guide's chords are read, by Guide chords and `gameMode`, the program's Steam Game Mode (EmuSen_Qt.md §7.7.H6).
        #[qproperty(bool, guide_chords, cxx_name = "guideChords", READ, NOTIFY)]
        #[qproperty(bool, game_mode, cxx_name = "gameMode")]
        /// The help bar's screen (HelpModel's numbers).
        #[qproperty(i32, help, READ, NOTIFY)]
        /// The pads' hand-over (`PadReader.gamePads`), whose seats the Players page shows and changes (EmuSen_Qt.md §7.7.H).
        #[qproperty(u64, game_pads, cxx_name = "gamePads")]
        /// The bindings page: whether it is shown, its drawing file, controls, labels' words, focused and held controls,
        /// its strip and caption, and whether a press is listened for or the tester runs.
        #[qproperty(bool, bindings_shown, cxx_name = "bindingsShown", READ, NOTIFY = drawn)]
        /// Eclipse `#12` / `#75` bindings state: whether keys view, whether from Pause Card, and full JSON payload.
        #[qproperty(bool, bindings_keys, cxx_name = "bindingsKeys", READ, NOTIFY = drawn)]
        #[qproperty(bool, bindings_from_card, cxx_name = "bindingsFromCard", READ, NOTIFY = drawn)]
        #[qproperty(QString, bindings_pad, cxx_name = "bindingsPad", READ, NOTIFY = drawn)]
        #[qproperty(QVariant, pad_meta, cxx_name = "padMeta", READ, NOTIFY = drawn)]
        #[qproperty(QString, pad_svg, cxx_name = "padSvg", READ, NOTIFY = pad_scalar_changed)]
        #[qproperty(i32, pad_focus, cxx_name = "padFocus", READ, NOTIFY = pad_scalar_changed)]
        #[qproperty(i32, pad_header_focus, cxx_name = "padHeaderFocus", READ, NOTIFY = pad_scalar_changed)]
        #[qproperty(f64, pad_left_scroll_y, cxx_name = "padLeftScrollY", READ, NOTIFY = pad_scalar_changed)]
        #[qproperty(f64, pad_right_scroll_y, cxx_name = "padRightScrollY", READ, NOTIFY = pad_scalar_changed)]
        #[qproperty(QString, pad_left_rows, cxx_name = "padLeftRows", READ, NOTIFY = pad_rows_changed)]
        #[qproperty(QString, pad_right_rows, cxx_name = "padRightRows", READ, NOTIFY = pad_rows_changed)]
        #[qproperty(QString, drawing, READ, NOTIFY = drawn)]
        #[qproperty(QStringList, drawing_controls, cxx_name = "drawingControls", READ, NOTIFY = drawn)]
        #[qproperty(QStringList, drawing_texts, cxx_name = "drawingTexts", READ, NOTIFY = drawn)]
        #[qproperty(i32, drawing_focus, cxx_name = "drawingFocus", READ, NOTIFY = drawn)]
        #[qproperty(QStringList, drawing_held, cxx_name = "drawingHeld", READ, NOTIFY = drawn)]
        #[qproperty(QStringList, binding_tabs, cxx_name = "bindingTabs", READ, NOTIFY = drawn)]
        #[qproperty(QString, binding_caption, cxx_name = "bindingCaption", READ, NOTIFY = drawn)]
        #[qproperty(bool, listening, READ, NOTIFY = drawn)]
        #[qproperty(bool, listening_pad, cxx_name = "listeningPad", READ, NOTIFY = drawn)]
        #[qproperty(bool, testing, READ, NOTIFY = drawn)]
        /// The tester: how far each control is pushed (the drawing's knobs), whether the interface's B is held to leave
        /// it, and how long it is held to leave.
        #[qproperty(QVariant, drawing_pushes, cxx_name = "drawingPushes", READ, NOTIFY = drawn)]
        #[qproperty(bool, tester_back, cxx_name = "testerBack", READ, NOTIFY = drawn)]
        #[qproperty(i32, tester_hold_ms, cxx_name = "testerHoldMs", READ, NOTIFY = drawn)]
        /// The Pause Card's Game Options ▸ Controllers rows, as plain data for `PauseModel.controllers` (EmuSen_Interface.md §20.3).
        #[qproperty(QString, card_controllers, cxx_name = "cardControllers", READ, NOTIFY = drawn)]
        /// The Pause Card's Game Options ▸ Display or Sound rows, as plain data for `PauseModel.settings` (settings_card.rs).
        #[qproperty(QString, card_settings, cxx_name = "cardSettings", READ, NOTIFY = drawn)]
        /// A page's question waiting for typed words: whether one is, its title and its prompt.
        #[qproperty(bool, typing, READ, NOTIFY = asked)]
        #[qproperty(QString, typing_title, cxx_name = "typingTitle", READ, NOTIFY = asked)]
        #[qproperty(QString, typing_prompt, cxx_name = "typingPrompt", READ, NOTIFY = asked)]
        /// Whether the current typing prompt is a password field masked on screen (#62).
        #[qproperty(bool, typing_secret, cxx_name = "typingSecret", READ, NOTIFY = asked)]
        /// Built-in theme manifests for Settings -> Library -> Themes (#61).
        #[qproperty(QStringList, built_ins, cxx_name = "builtIns")]
        /// Whether an EmuSen theme draws Settings, enabling Eclipse's board-projected pages.
        #[qproperty(bool, themed_settings, cxx_name = "themedSettings")]
        /// Eclipse `#20` Shaders overlay state and live Serenity preview feed.
        #[qproperty(bool, shaders_shown, cxx_name = "shadersShown", READ, NOTIFY = shaders_changed)]
        #[qproperty(QString, shaders_board, cxx_name = "shadersBoard", READ, NOTIFY = shaders_changed)]
        #[qproperty(QString, shaders_help, cxx_name = "shadersHelp", READ, NOTIFY = shaders_changed)]
        #[qproperty(QVariant, shaders_meta, cxx_name = "shadersMeta", READ, NOTIFY = shaders_changed)]
        #[qproperty(QString, shaders_presets_json, cxx_name = "shadersPresetsJson", READ, NOTIFY = shaders_lists_changed)]
        #[qproperty(QString, shaders_categories_json, cxx_name = "shadersCategoriesJson", READ, NOTIFY = shaders_lists_changed)]
        #[qproperty(QString, shaders_sliders_json, cxx_name = "shadersSlidersJson", READ, NOTIFY = shaders_lists_changed)]
        #[qproperty(f64, shaders_split, cxx_name = "shadersSplit", READ, NOTIFY = shaders_scalar_changed)]
        #[qproperty(bool, shaders_comparing, cxx_name = "shadersComparing", READ, NOTIFY = shaders_scalar_changed)]
        #[qproperty(i32, shaders_focus_zone, cxx_name = "shadersFocusZone", READ, NOTIFY = shaders_scalar_changed)]
        #[qproperty(i32, shaders_focused_preset, cxx_name = "shadersFocusedPreset", READ, NOTIFY = shaders_scalar_changed)]
        #[qproperty(i32, shaders_focused_slider, cxx_name = "shadersFocusedSlider", READ, NOTIFY = shaders_scalar_changed)]
        #[qproperty(bool, shaders_confirm_open, cxx_name = "shadersConfirmOpen", READ, NOTIFY = shaders_scalar_changed)]
        #[qproperty(i32, shaders_source_idx, cxx_name = "shadersSourceIdx", READ, NOTIFY = shaders_scalar_changed)]
        #[qproperty(i32, shaders_category_idx, cxx_name = "shadersCategoryIdx", READ, NOTIFY = shaders_scalar_changed)]
        #[qproperty(i32, shaders_scope_idx, cxx_name = "shadersScopeIdx", READ, NOTIFY = shaders_scalar_changed)]
        #[qproperty(QString, shaders_counter, cxx_name = "shadersCounter", READ, NOTIFY = shaders_scalar_changed)]
        #[qproperty(u64, shader_preview_feed, cxx_name = "shaderPreviewFeed", READ, NOTIFY = shader_preview_changed)]
        #[qproperty(QString, shader_preview_source, cxx_name = "shaderPreviewSource", READ, NOTIFY = shader_preview_changed)]
        #[qproperty(bool, shader_preview_pending, cxx_name = "shaderPreviewPending", READ, NOTIFY = shader_preview_changed)]
        /// Why a read or a save failed, in the platform's words; empty when it did not.
        #[qproperty(QString, problem, READ, NOTIFY)]
        type SettingsModel = super::SettingsModelRust;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &SettingsModel, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &SettingsModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &SettingsModel, parent: &QModelIndex) -> i32;

        /// The settings read again.
        #[qinvokable]
        fn refresh(self: Pin<&mut SettingsModel>);

        /// The tone (Kit.Tones) of the pane's further line at `index`, the text's past the tones the pane gives.
        #[qinvokable]
        #[cxx_name = "lineTone"]
        fn line_tone(self: &SettingsModel, index: i32) -> i32;

        /// The scale of the sample at `index`, 1 past the samples.
        #[qinvokable]
        #[cxx_name = "sampleScale"]
        fn sample_scale(self: &SettingsModel, index: i32) -> f64;

        /// Formats typed text for the typing overlay, masking characters when `secret` is true.
        #[qinvokable]
        #[cxx_name = "typingDisplay"]
        fn typing_display(self: &SettingsModel, text: &QString, secret: bool) -> QString;

        /// Left (-1) or Right (+1) on a row: whether it did anything.
        #[qinvokable]
        fn step(self: Pin<&mut SettingsModel>, row: i32, by: i32) -> bool;

        /// A on a row: whether it did anything.
        #[qinvokable]
        fn choose(self: Pin<&mut SettingsModel>, row: i32) -> bool;

        /// A slider row moved to `amount`, by the pad's walk or the pointer: whether it did anything.
        #[qinvokable]
        fn slide(self: Pin<&mut SettingsModel>, row: i32, amount: f64) -> bool;

        /// A lever of the shader of `system` (a console by its stored name) moved to `value` and saved: a game of that
        /// console draws with it at once.
        #[qsignal]
        fn lever(self: Pin<&mut SettingsModel>, system: QString, id: QString, value: f64);

        /// B: the page closed (true), or `left` raised on a section (false).
        #[qinvokable]
        fn back(self: Pin<&mut SettingsModel>) -> bool;

        /// A door to another screen chosen, by its name in the description.
        #[qsignal]
        fn door(self: Pin<&mut SettingsModel>, name: QString);

        /// B on a section: Settings is left.
        #[qsignal]
        fn left(self: Pin<&mut SettingsModel>);

        /// Settings closed after a value was saved through Galaxia: what the library reads from the settings may have changed.
        #[qsignal]
        fn saved(self: Pin<&mut SettingsModel>);

        /// LB (-1) or RB (+1) on a console's page: the console before or after it; false elsewhere.
        #[qinvokable]
        fn shoulder(self: Pin<&mut SettingsModel>, by: i32) -> bool;

        /// A page closed: the row that opened it is focused again.
        #[qsignal]
        fn refocus(self: Pin<&mut SettingsModel>, row: i32);

        /// A folder row's key (`Model Key`) to choose a folder for in the system's dialog, starting at `start`.
        #[qsignal]
        #[cxx_name = "chooseFolder"]
        fn choose_folder(self: Pin<&mut SettingsModel>, key: QString, start: QString);

        /// The folder chosen for a folder row's key, saved at once.
        #[qinvokable]
        #[cxx_name = "setFolder"]
        fn set_folder(self: Pin<&mut SettingsModel>, key: &QString, path: &QString);

        /// A folder to show in the system's file manager, through the program's one action for it.
        #[qsignal]
        #[cxx_name = "openFolder"]
        fn open_folder(self: Pin<&mut SettingsModel>, path: QString);

        /// The bindings page's drawing changed.
        #[qsignal]
        fn drawn(self: Pin<&mut SettingsModel>);

        /// What a running game takes from the settings changed.
        #[qsignal]
        #[cxx_name = "playChanged"]
        fn play_changed(self: Pin<&mut SettingsModel>);

        /// Words to type for a page (a profile's name), starting with `start`; `typing`, `typingTitle` and
        /// `typingPrompt` say the question, and `typed` answers.
        #[qsignal]
        #[cxx_name = "typeWords"]
        fn type_words(self: Pin<&mut SettingsModel>, start: QString);

        /// The question's words changed.
        #[qsignal]
        fn asked(self: Pin<&mut SettingsModel>);

        /// A declared action the page did not take, for the screens (Start, Home).
        #[qsignal]
        fn passed(self: Pin<&mut SettingsModel>, key: QString);

        /// A pad's button while a key is listened for: B lets the listening go.
        #[qinvokable]
        #[cxx_name = "padPressed"]
        fn pad_pressed(self: Pin<&mut SettingsModel>, button: PadButton);

        /// A row of the card's Controllers page stepped (`by` ±1) or chosen (`by` 0).
        #[qinvokable]
        #[cxx_name = "cardAct"]
        fn card_act(self: Pin<&mut SettingsModel>, index: i32, by: i32);

        /// The consoles' section with `console`'s page open, as Choose a console's options ask; false where no engine runs it.
        #[qinvokable]
        #[cxx_name = "openConsole"]
        fn open_console(self: Pin<&mut SettingsModel>, console: &QString) -> bool;

        /// The card's Display (`page` "display", on `console`'s rows) or Sound opened.
        #[qinvokable]
        #[cxx_name = "cardOpen"]
        fn card_open(self: Pin<&mut SettingsModel>, page: &QString, console: &QString);

        /// A row of the card's Display or Sound stepped (`by` ±1) or chosen (`by` 0); true where it stored a display
        /// setting the picture is to be read again for.
        #[qinvokable]
        #[cxx_name = "cardSettingsAct"]
        fn card_settings_act(self: Pin<&mut SettingsModel>, index: i32, by: i32) -> bool;

        /// B on a page a door of the card's Display opened: the page closed.
        #[qinvokable]
        #[cxx_name = "cardBack"]
        fn card_back(self: Pin<&mut SettingsModel>);

        /// The words typed for the page that asked, or none for a question let go.
        #[qinvokable]
        fn typed(self: Pin<&mut SettingsModel>, words: &QString);

        /// The seats read again from the pads' hand-over, as the pads' reader says they changed.
        #[qinvokable]
        #[cxx_name = "seatsChanged"]
        fn seats_changed(self: Pin<&mut SettingsModel>);

        /// A direction on the bindings page (`dx`, `dy`): the nearest control that way on the drawing.
        #[qinvokable]
        #[cxx_name = "bindingMove"]
        fn binding_move(self: Pin<&mut SettingsModel>, dx: i32, dy: i32);

        /// A control of the bindings page focused by the pointer.
        #[qinvokable]
        #[cxx_name = "bindingFocus"]
        fn binding_focus(self: Pin<&mut SettingsModel>, index: i32);

        /// A on the bindings page: a press listened for.
        #[qinvokable]
        #[cxx_name = "bindingAccept"]
        fn binding_accept(self: Pin<&mut SettingsModel>);

        /// Selects a player seat (`1..=2`) on Eclipse Controller bindings.
        #[qinvokable]
        #[cxx_name = "bindingPlayer"]
        fn binding_player(self: Pin<&mut SettingsModel>, player: i32) -> bool;

        /// Switches Eclipse bindings or Keyboard keys directly to `console`.
        #[qinvokable]
        #[cxx_name = "bindingConsole"]
        fn binding_console(self: Pin<&mut SettingsModel>, console: &QString) -> bool;

        /// Switches among the five Eclipse controller sub-navigation tabs.
        #[qinvokable]
        #[cxx_name = "bindingNav"]
        fn binding_nav(self: Pin<&mut SettingsModel>, tab: i32) -> bool;

        /// Steps across the five Eclipse controller sub-navigation tabs.
        #[qinvokable]
        #[cxx_name = "bindingPageStep"]
        fn binding_page_step(self: Pin<&mut SettingsModel>, by: i32) -> bool;

        /// Returns directly to Controllers settings from the Eclipse bindings breadcrumb.
        #[qinvokable]
        #[cxx_name = "bindingSettings"]
        fn binding_settings(self: Pin<&mut SettingsModel>) -> bool;

        /// Handles a pad or navigation button press on Eclipse Controller bindings / Keyboard keys.
        #[qinvokable]
        #[cxx_name = "bindingPadButton"]
        fn binding_pad_button(self: Pin<&mut SettingsModel>, button: PadButton, repeat: bool) -> bool;

        /// One of the help bar's declared actions on the bindings page or Hotkeys (reset, player, clear, test); one the
        /// page does not take is `passed` on.
        #[qinvokable]
        fn act(self: Pin<&mut SettingsModel>, key: &QString);

        /// A key pressed while a press is listened for, by Qt's number and modifiers: bound, or Escape cancelling;
        /// true when it was taken.
        #[qinvokable]
        #[cxx_name = "heardKey"]
        fn heard_key(self: Pin<&mut SettingsModel>, key: i32, modifiers: i32) -> bool;

        /// A key let go while the tester runs; true when it was taken.
        #[qinvokable]
        #[cxx_name = "releasedKey"]
        fn released_key(self: Pin<&mut SettingsModel>, key: i32, modifiers: i32) -> bool;

        /// A button of the interface down or up on the pads while the tester runs, before the player's swap (`swap`):
        /// whether it is B, held to leave the tester. True for a release, which the router still hears, so that no
        /// button stays held there.
        #[qinvokable]
        #[cxx_name = "testerButton"]
        fn tester_button(self: Pin<&mut SettingsModel>, button: PadButton, down: bool, swap: bool) -> bool;

        /// Selects an inline kind pill on Settings -> Library -> Themes (#61).
        #[qinvokable]
        #[cxx_name = "chooseInlinePill"]
        fn choose_inline_pill(self: Pin<&mut SettingsModel>, index: i32) -> bool;

        /// An EmuSen theme was chosen from Settings -> Library -> Themes (#61): folder for ThemeHost, empty for reference.
        #[qsignal]
        #[cxx_name = "themeUsed"]
        fn theme_used(self: Pin<&mut SettingsModel>, folder: QString);

        /// The library's look was stored from Settings -> Library -> Themes (#61): whether an ES-DE theme draws it.
        #[qsignal]
        #[cxx_name = "themeRestyled"]
        fn theme_restyled(self: Pin<&mut SettingsModel>, esde: bool);

        #[qsignal]
        #[cxx_name = "shadersChanged"]
        fn shaders_changed(self: Pin<&mut SettingsModel>);

        #[qsignal]
        #[cxx_name = "shadersScalarChanged"]
        fn shaders_scalar_changed(self: Pin<&mut SettingsModel>);

        #[qsignal]
        #[cxx_name = "shadersListsChanged"]
        fn shaders_lists_changed(self: Pin<&mut SettingsModel>);

        #[qsignal]
        #[cxx_name = "padScalarChanged"]
        fn pad_scalar_changed(self: Pin<&mut SettingsModel>);

        #[qsignal]
        #[cxx_name = "padRowsChanged"]
        fn pad_rows_changed(self: Pin<&mut SettingsModel>);

        #[qsignal]
        #[cxx_name = "shaderPreviewChanged"]
        fn shader_preview_changed(self: Pin<&mut SettingsModel>);

        #[qsignal]
        #[cxx_name = "shaderPreviewReady"]
        fn shader_preview_ready(self: Pin<&mut SettingsModel>);

        #[qsignal]
        #[cxx_name = "shadersPreviewDebounce"]
        fn shaders_preview_debounce(self: Pin<&mut SettingsModel>);

        #[qinvokable]
        #[cxx_name = "shadersInit"]
        fn shaders_init(self: Pin<&mut SettingsModel>, host_feed: u64, paused_bytes: &QByteArray);

        #[qinvokable]
        #[cxx_name = "shadersRelease"]
        fn shaders_release(self: Pin<&mut SettingsModel>);

        #[qinvokable]
        #[cxx_name = "commitShaderPreview"]
        fn commit_shader_preview(self: Pin<&mut SettingsModel>);

        #[qinvokable]
        #[cxx_name = "shadersPadButton"]
        fn shaders_pad_button(self: Pin<&mut SettingsModel>, button: PadButton, down: bool, repeat: bool) -> bool;

        #[qinvokable]
        #[cxx_name = "shadersKeyNav"]
        fn shaders_key_nav(self: Pin<&mut SettingsModel>, code: i32) -> bool;

        #[qinvokable]
        #[cxx_name = "shadersSource"]
        fn shaders_source(self: Pin<&mut SettingsModel>, index: i32);

        #[qinvokable]
        #[cxx_name = "shadersCategory"]
        fn shaders_category(self: Pin<&mut SettingsModel>, index: i32);

        #[qinvokable]
        #[cxx_name = "shadersFocusPreset"]
        fn shaders_focus_preset(self: Pin<&mut SettingsModel>, index: i32, activate: bool);

        #[qinvokable]
        #[cxx_name = "shadersScope"]
        fn shaders_scope(self: Pin<&mut SettingsModel>, index: i32);

        #[qinvokable]
        #[cxx_name = "shadersToggleFavourite"]
        fn shaders_toggle_favourite(self: Pin<&mut SettingsModel>, index: i32);

        #[qinvokable]
        #[cxx_name = "shadersSearch"]
        fn shaders_search(self: Pin<&mut SettingsModel>);

        #[qinvokable]
        #[cxx_name = "shadersAllSettings"]
        fn shaders_all_settings(self: Pin<&mut SettingsModel>);

        #[qinvokable]
        #[cxx_name = "shadersAskRemove"]
        fn shaders_ask_remove(self: Pin<&mut SettingsModel>);

        #[qinvokable]
        #[cxx_name = "shadersConfirmRemove"]
        fn shaders_confirm_remove(self: Pin<&mut SettingsModel>, confirm: bool);

        #[qinvokable]
        #[cxx_name = "shadersStepSlider"]
        fn shaders_step_slider(self: Pin<&mut SettingsModel>, slot: i32, by: i32);

        #[qinvokable]
        #[cxx_name = "shadersSlideSlider"]
        fn shaders_slide_slider(self: Pin<&mut SettingsModel>, slot: i32, norm: f64);

        #[qinvokable]
        #[cxx_name = "shadersSetSplit"]
        fn shaders_set_split(self: Pin<&mut SettingsModel>, norm: f64);

        #[qinvokable]
        #[cxx_name = "shadersScrubSplit"]
        fn shaders_scrub_split(self: Pin<&mut SettingsModel>, delta: f64);

        #[qinvokable]
        #[cxx_name = "shadersHoldCompare"]
        fn shaders_hold_compare(self: Pin<&mut SettingsModel>, down: bool);

        #[qinvokable]
        #[cxx_name = "shadersBack"]
        fn shaders_back(self: Pin<&mut SettingsModel>);
    }

    unsafe extern "RustQt" {
        #[inherit]
        fn index(self: &SettingsModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;

        #[inherit]
        #[cxx_name = "beginRemoveRows"]
        fn begin_remove_rows(self: Pin<&mut SettingsModel>, parent: &QModelIndex, first: i32, last: i32);

        #[inherit]
        #[cxx_name = "endRemoveRows"]
        fn end_remove_rows(self: Pin<&mut SettingsModel>);

        #[inherit]
        #[cxx_name = "beginInsertRows"]
        fn begin_insert_rows(self: Pin<&mut SettingsModel>, parent: &QModelIndex, first: i32, last: i32);

        #[inherit]
        #[cxx_name = "endInsertRows"]
        fn end_insert_rows(self: Pin<&mut SettingsModel>);

        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(self: Pin<&mut SettingsModel>, top_left: &QModelIndex, bottom_right: &QModelIndex, roles: &QVector_i32);
    }

    impl cxx_qt::Threading for SettingsModel {}
    impl cxx_qt::Initialize for SettingsModel {}
}

pub struct SettingsModelRust {
    platform_library: QString,
    cores: QString,
    world: QString,
    open: bool,
    facts: QString,
    sections: QStringList,
    section: i32,
    trail: QStringList,
    focused: i32,
    sentence: QString,
    pane_lines: QStringList,
    pane_muted: QStringList,
    sentence_tone: i32,
    line_tones: Vec<i32>,
    preview: i32,
    caption: QString,
    meter: f64,
    samples: QStringList,
    sample_scales: Vec<f64>,
    ready: bool,
    text_size: QString,
    light: bool,
    reduced_motion: bool,
    swap_buttons: bool,
    controller_family: i32,
    controller_notices: bool,
    rumble: bool,
    volume: f64,
    output: QString,
    sync_to_display: bool,
    pause_in_background: bool,
    show_fps: bool,
    show_status_text: bool,
    show_status_bar: bool,
    navigation_volume: f64,
    play_hours: i32,
    guide_chords: bool,
    game_mode: bool,
    help: i32,
    problem: QString,
    view: SettingsView,
    rows: Vec<settings::Row>,
    /// The reads asked for, so an older read that finishes late is dropped.
    asked: u64,
    /// The scrape mutations asked for, so an older scrape task finishing late is dropped.
    scrape_asked: u64,
    /// The typing prompts asked for, so a deferred `typeWords` from a closed prompt is dropped.
    type_asked: u64,
    /// Whether the last read asked for is still to come back.
    reading: bool,
    game_pads: u64,
    bindings_shown: bool,
    bindings_keys: bool,
    bindings_from_card: bool,
    bindings_pad: QString,
    pad_meta: QVariant,
    pad_svg: QString,
    pad_focus: i32,
    pad_header_focus: i32,
    pad_left_scroll_y: f64,
    pad_right_scroll_y: f64,
    pad_left_rows: QString,
    pad_right_rows: QString,
    pad_meta_json: String,
    drawing: QString,
    drawing_controls: QStringList,
    drawing_texts: QStringList,
    drawing_focus: i32,
    drawing_held: QStringList,
    drawing_pushes: QVariant,
    tester_back: bool,
    tester_hold_ms: i32,
    binding_tabs: QStringList,
    binding_caption: QString,
    listening: bool,
    listening_pad: bool,
    testing: bool,
    card_controllers: QString,
    card_settings: QString,
    typing: bool,
    typing_title: QString,
    typing_prompt: QString,
    typing_secret: bool,
    built_ins: QStringList,
    themed_settings: bool,
    shaders_shown: bool,
    shaders_board: QString,
    shaders_help: QString,
    shaders_meta: QVariant,
    shaders_presets_json: QString,
    shaders_categories_json: QString,
    shaders_sliders_json: QString,
    shaders_split: f64,
    shaders_comparing: bool,
    shaders_focus_zone: i32,
    shaders_focused_preset: i32,
    shaders_focused_slider: i32,
    shaders_confirm_open: bool,
    shaders_source_idx: i32,
    shaders_category_idx: i32,
    shaders_scope_idx: i32,
    shaders_counter: QString,
    shaders_meta_json: String,
    shaders_open_gen: u64,
    shaders_meta_gen: u64,
    shader_preview_feed: u64,
    shader_preview_source: QString,
    shader_preview_pending: bool,
    shaders_state: Option<crate::settings_shaders::ShadersBoard>,
    shader_preview: crate::shader_preview::ShaderPreview,
    clients: ConsolesClients,
    /// The page's question waiting for typed words.
    typed_for: Option<crate::settings_controllers::Typed>,
    /// Each pad's buttons at the last read of the seats, so a press listened for is the one newly down.
    pressed: Vec<(u64, u32)>,
    /// A value saved since Settings opened, told as `saved` when it closes.
    saved_since_open: bool,
}

fn list(words: impl Iterator<Item = String>) -> QStringList {
    let mut out = QList::<QString>::default();
    for w in words {
        out.append(QString::from(w.as_str()));
    }
    QStringList::from(&out)
}

impl Drop for SettingsModelRust {
    fn drop(&mut self) {
        self.shader_preview.release();
    }
}

impl Default for SettingsModelRust {
    fn default() -> Self {
        let view = SettingsView::new();
        SettingsModelRust {
            platform_library: QString::default(),
            cores: QString::default(),
            world: QString::default(),
            open: false,
            saved_since_open: false,
            facts: QString::default(),
            sections: list(view.sections().into_iter()),
            section: 0,
            trail: QStringList::default(),
            focused: 0,
            sentence: QString::default(),
            pane_lines: QStringList::default(),
            pane_muted: QStringList::default(),
            sentence_tone: 0,
            line_tones: Vec::new(),
            preview: 0,
            caption: QString::default(),
            meter: 0.0,
            samples: QStringList::default(),
            sample_scales: Vec::new(),
            ready: false,
            text_size: QString::default(),
            light: false,
            reduced_motion: false,
            swap_buttons: false,
            controller_family: -1,
            controller_notices: false,
            rumble: false,
            volume: 1.0,
            output: QString::default(),
            sync_to_display: true,
            pause_in_background: true,
            show_fps: false,
            show_status_text: false,
            show_status_bar: false,
            navigation_volume: 1.0,
            play_hours: 0,
            guide_chords: false,
            game_mode: false,
            help: shown_screen(Screen::Settings),
            problem: QString::default(),
            rows: view.rows(),
            view,
            asked: 0,
            scrape_asked: 0,
            type_asked: 0,
            reading: false,
            game_pads: 0,
            bindings_shown: false,
            bindings_keys: false,
            bindings_from_card: false,
            bindings_pad: QString::default(),
            pad_meta: QVariant::default(),
            pad_svg: QString::default(),
            pad_focus: 0,
            pad_header_focus: -1,
            pad_left_scroll_y: 0.0,
            pad_right_scroll_y: 0.0,
            pad_left_rows: QString::from("[]"),
            pad_right_rows: QString::from("[]"),
            pad_meta_json: String::new(),
            drawing: QString::default(),
            drawing_controls: QStringList::default(),
            drawing_texts: QStringList::default(),
            drawing_focus: -1,
            drawing_held: QStringList::default(),
            drawing_pushes: QVariant::from(&QList::<f64>::default()),
            tester_back: false,
            tester_hold_ms: 0,
            binding_tabs: QStringList::default(),
            binding_caption: QString::default(),
            listening: false,
            listening_pad: false,
            testing: false,
            card_controllers: QString::default(),
            card_settings: QString::default(),
            typing: false,
            typing_title: QString::default(),
            typing_prompt: QString::default(),
            typing_secret: false,
            built_ins: QStringList::default(),
            themed_settings: false,
            shaders_shown: false,
            shaders_board: QString::from("{}"),
            shaders_help: QString::from("[]"),
            shaders_meta: QVariant::default(),
            shaders_presets_json: QString::from("[]"),
            shaders_categories_json: QString::from("[]"),
            shaders_sliders_json: QString::from("[]"),
            shaders_split: 0.5,
            shaders_comparing: false,
            shaders_focus_zone: 4,
            shaders_focused_preset: 0,
            shaders_focused_slider: 0,
            shaders_confirm_open: false,
            shaders_source_idx: 0,
            shaders_category_idx: 1,
            shaders_scope_idx: 1,
            shaders_counter: QString::default(),
            shaders_meta_json: String::new(),
            shaders_open_gen: 0,
            shaders_meta_gen: 0,
            shader_preview_feed: 0,
            shader_preview_source: QString::default(),
            shader_preview_pending: false,
            shaders_state: None,
            shader_preview: crate::shader_preview::ShaderPreview::default(),
            clients: ConsolesClients::default(),
            typed_for: None,
            pressed: Vec::new(),
        }
    }
}

impl cxx_qt::Initialize for qobject::SettingsModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut()
            .on_open_changed(|mut model| {
                if model.open {
                    model.read_if_named();
                } else {
                    let was_typing = {
                        let mut r = model.as_mut().rust_mut();
                        r.scrape_asked = r.scrape_asked.wrapping_add(1);
                        r.type_asked = r.type_asked.wrapping_add(1);
                        r.view.scrape.pending_user = None;
                        r.typed_for = None;
                        let was = r.typing || r.typing_secret;
                        r.typing = false;
                        r.typing_secret = false;
                        was
                    };
                    if was_typing {
                        model.as_mut().asked();
                    }
                    if model.view.consoles_with_games() {
                        model.as_mut().rust_mut().view.reset_consoles_filter();
                        model.as_mut().shown(true);
                    }
                    if std::mem::take(&mut model.as_mut().rust_mut().saved_since_open) {
                        model.as_mut().saved();
                    }
                    model.as_mut().close_shaders_state();
                }
            })
            .release();
        self.as_mut().on_platform_library_changed(|model| model.read_if_named()).release();
        self.as_mut()
            .on_facts_changed(|mut model| {
                let facts = serde_json::from_str(&model.facts.to_string()).unwrap_or(serde_json::Value::Null);
                model.as_mut().rust_mut().view.facts = facts;
                model.shown(false);
            })
            .release();
        self.as_mut()
            .on_section_changed(|mut model| {
                model.as_mut().close_shaders_state();
                let section = model.section.max(0) as usize;
                model.as_mut().rust_mut().view.choose_section(section);
                model.shown(true);
            })
            .release();
        self.as_mut()
            .on_game_mode_changed(|model| {
                if model.ready {
                    model.applied();
                }
            })
            .release();
        self.as_mut()
            .on_game_pads_changed(|mut model| {
                model.as_mut().settle_pads();
                model.seats_changed();
            })
            .release();
        self.as_mut()
            .on_built_ins_changed(|model| {
                if !model.platform_library.is_empty() {
                    model.refresh();
                }
            })
            .release();
        self.as_mut()
            .on_themed_settings_changed(|mut model| {
                let themed = model.themed_settings;
                model.as_mut().rust_mut().view.themed_settings = themed;
                if !themed && matches!(model.view.page(), Some(settings::Page::Themes)) {
                    if let Some(row) = model.as_mut().rust_mut().view.back() {
                        model.as_mut().shown(true);
                        model.as_mut().set_focused(row as i32);
                        model.refocus(row as i32);
                        return;
                    }
                }
                model.shown(false);
            })
            .release();
        self.on_focused_changed(|mut model| {
            let focused = model.focused.max(0) as usize;
            model.as_mut().rust_mut().view.focused_row = focused;
            let help = shown_screen(model.view.help());
            if model.help != help {
                model.as_mut().rust_mut().help = help;
                model.as_mut().help_changed();
            }
            model.pane();
        })
        .release();
    }
}

const LABEL: i32 = 0;
const VALUE: i32 = 1;
const SENTENCE: i32 = 2;
const KIND: i32 = 3;
const HELD: i32 = 4;
const MARKED: i32 = 5;
const KEY: i32 = 6;
const MINIMUM: i32 = 7;
const MAXIMUM: i32 = 8;
const STEP: i32 = 9;
const AMOUNT: i32 = 10;
const CODE: i32 = 11;

impl qobject::SettingsModel {
    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(row) = self.rows.get(index.row() as usize) else { return QVariant::default() };
        let text = |s: &str| QVariant::from(&QString::from(s));
        match role {
            LABEL => text(&row.label),
            VALUE => text(&row.value),
            SENTENCE => text(&self.view.pane(index.row() as usize).sentence),
            KIND => QVariant::from(&(row.kind.index() as i32)),
            HELD => QVariant::from(&row.held),
            MARKED => QVariant::from(&row.marked),
            KEY => text(&row.id),
            CODE => QVariant::from(&row.code),
            MINIMUM..=AMOUNT => self.view.slider(index.row() as usize).map_or_else(QVariant::default, |s| QVariant::from(&s[(role - MINIMUM) as usize])),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        for (role, name) in [(LABEL, "label"), (VALUE, "value"), (SENTENCE, "sentence"), (KIND, "kind"), (HELD, "held"), (MARKED, "marked"), (KEY, "key")]
            .into_iter()
            .chain([(MINIMUM, "minimum"), (MAXIMUM, "maximum"), (STEP, "step"), (AMOUNT, "amount"), (CODE, "code")])
        {
            roles.insert(role, QByteArray::from(name));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.rows.len() as i32
    }

    /// The screen's view as the rows were last told from it, for a theme's read-only view (EmuSen_Qt.md §3.2, `Settings`).
    pub fn view(&self) -> &SettingsView {
        &self.view
    }

    /// The rows shown, as the list's roles give them.
    pub fn shown_rows(&self) -> &[settings::Row] {
        &self.rows
    }

    pub fn line_tone(&self, index: i32) -> i32 {
        usize::try_from(index).ok().and_then(|i| self.line_tones.get(i)).copied().unwrap_or(0)
    }

    pub fn sample_scale(&self, index: i32) -> f64 {
        usize::try_from(index).ok().and_then(|i| self.sample_scales.get(i)).copied().unwrap_or(1.0)
    }

    /// Formats typed text for the typing overlay, masking characters when `secret` is true.
    pub fn typing_display(&self, text: &QString, secret: bool) -> QString {
        if secret {
            QString::from("•".repeat(text.to_string().chars().count()).as_str())
        } else {
            text.clone()
        }
    }

    pub fn refresh(mut self: Pin<&mut Self>) {
        let library = self.platform_library.to_string();
        if library.is_empty() {
            return;
        }
        let asked = self.asked.wrapping_add(1);
        self.as_mut().rust_mut().asked = asked;
        self.as_mut().rust_mut().reading = true;
        // Cleared at the start of each attempt, so the same failure twice is said twice (EmuSen_Qt.md §7.9).
        self.as_mut().said(String::new());
        let thread = self.qt_thread();
        let clients = self.clients.clone();
        let (cores, world) = (self.cores.to_string(), self.world.to_string());
        let manifests: Vec<String> = self.built_ins.iter().map(|m| m.to_string()).collect();
        let board_snap = self.shaders_state.clone();
        self.clients.spawn(move || {
            let read = ClientSettings::open(&library).and_then(|source| {
                let mut read = settings::read(&source)?;
                read.consoles = if world.is_empty() {
                    crate::consoles::read(&clients.consoles(&library, &cores, true)?)?
                } else {
                    let scripted = ScriptedConsoles::open(&world)?;
                    if let Some(outputs) = scripted.outputs() {
                        read.outputs = outputs;
                    }
                    crate::consoles::read(&scripted)?
                };
                let names: Vec<&str> = read.consoles.consoles.iter().map(|c| c.console.as_str()).collect();
                read.bindings = source.bindings(&names)?;
                let store = source.themes_store();
                let built_ins: Vec<(&str, &str)> = crate::themes::BUILT_IN.iter().copied().zip(manifests.iter().map(String::as_str)).collect();
                let listing = crate::themes::listing(&read.app, store.as_deref(), &built_ins);
                let scrape = source.scrape_snapshot();
                let shader_data = if let Some(board) = board_snap.as_ref() {
                    let mut tmp = SettingsView::new();
                    tmp.read = read.clone();
                    let platform = clients.platform(&library).ok();
                    Some(crate::settings_shaders::ShadersBoard::collect_open_data(platform.as_deref(), &tmp, board, true))
                } else {
                    None
                };
                Ok((read, source.chooser()?, listing, scrape, shader_data))
            });
            let _ = thread.queue(move |mut model| {
                if model.asked != asked {
                    return;
                }
                model.as_mut().rust_mut().reading = false;
                match read {
                    Ok((read, chooser, mut listing, scrape, shader_data)) => {
                        if let Some(prev) = model.view.themes.as_ref() {
                            for row in &mut listing.rows {
                                if let Some(before) = prev.rows.iter().find(|r| r.id == row.id && r.kind == row.kind && row.checked == crate::themes::Checked::Not) {
                                    row.checked = before.checked.clone();
                                }
                            }
                        }
                        let mut r = model.as_mut().rust_mut();
                        let (note, pending) = (r.view.scrape.status_note.clone(), r.view.scrape.pending_user.clone());
                        r.view.read = read;
                        r.view.chooser = Some(chooser);
                        r.view.themes = Some(listing);
                        r.view.scrape = scrape;
                        r.view.scrape.status_note = note;
                        r.view.scrape.pending_user = pending;
                        if let (Some(mut board), Some(data)) = (r.shaders_state.take(), shader_data) {
                            board.apply_open_data(data, &r.view);
                            r.shaders_state = Some(board);
                        }
                        model.as_mut().said(String::new());
                    }
                    Err(words) => model.as_mut().said(format!("The settings could not be read: {words}")),
                }
                model.as_mut().applied();
                model.as_mut().shown(false);
                if model.shaders_state.is_some() {
                    model.as_mut().sync_shaders_view();
                    if model.shaders_shown {
                        model.as_mut().sync_shaders_preview(false);
                    }
                }
            });
        });
    }

    pub fn step(mut self: Pin<&mut Self>, row: i32, by: i32) -> bool {
        let outcome = self.as_mut().rust_mut().view.step(row.max(0) as usize, by);
        self.outcome(outcome)
    }

    /// X on a row, a theme's `Actions.act("default")`: its value back to its default; whether it did anything.
    pub fn to_default(mut self: Pin<&mut Self>, row: i32) -> bool {
        let outcome = self.as_mut().rust_mut().view.to_default(row.max(0) as usize);
        self.outcome(outcome)
    }

    /// Y on Consoles: toggles the has-games filter and resets focus to the first row (board #27).
    pub fn toggle_consoles_with_games(mut self: Pin<&mut Self>) -> bool {
        if !self.as_mut().rust_mut().view.toggle_consoles_with_games() {
            return false;
        }
        self.shown(true);
        true
    }

    /// LT/RT on Consoles: jumps focus to the first console of the previous or next starting letter (board #27).
    pub fn jump_console_letter(self: Pin<&mut Self>, by: i32) -> bool {
        let Some(to) = self.view.jump_console_letter(self.focused.max(0) as usize, by) else { return false };
        self.set_focused(to as i32);
        true
    }

    pub fn choose(mut self: Pin<&mut Self>, row: i32) -> bool {
        let index = row.max(0) as usize;
        if self.themed_settings && self.view.is_shaders_door(index) {
            self.open_shaders_board(index);
            return true;
        }
        let outcome = self.as_mut().rust_mut().view.choose(index);
        self.outcome(outcome)
    }

    pub fn slide(mut self: Pin<&mut Self>, row: i32, amount: f64) -> bool {
        let outcome = self.as_mut().rust_mut().view.slide(row.max(0) as usize, amount);
        self.outcome(outcome)
    }

    pub fn back(mut self: Pin<&mut Self>) -> bool {
        if self.shaders_shown && self.shaders_state.is_some() {
            self.close_shaders_to_display();
            return true;
        }
        if self.as_mut().rust_mut().view.back_taken() {
            self.shown(false);
            return true;
        }
        match self.as_mut().rust_mut().view.back() {
            Some(row) => {
                if self.shaders_state.as_ref().is_some_and(|b| b.return_params) {
                    if let Some(mut board) = self.as_mut().rust_mut().shaders_state.take() {
                        board.return_params = false;
                        board.confirm_remove = false;
                        board.sync_preview_stored_in_memory(&self.view);
                        self.as_mut().rust_mut().shaders_state = Some(board);
                    }
                    self.as_mut().sync_shaders_view();
                    self.as_mut().maybe_fetch_shaders_meta();
                    self.sync_shaders_preview(true);
                    return true;
                }
                self.as_mut().shown(true);
                self.refocus(row as i32);
                true
            }
            None => {
                if self.bindings_shown {
                    self.as_mut().shown(true);
                }
                self.left();
                false
            }
        }
    }

    pub fn set_folder(mut self: Pin<&mut Self>, key: &QString, path: &QString) {
        if path.is_empty() {
            return;
        }
        let outcome = self.as_mut().rust_mut().view.set_folder(&key.to_string(), &path.to_string());
        self.outcome(outcome);
    }

    pub fn shoulder(mut self: Pin<&mut Self>, by: i32) -> bool {
        if self.view.binding_open() {
            let stepped = self.as_mut().rust_mut().view.binding_shoulder(by);
            if stepped {
                self.shown(false);
            }
            return stepped;
        }
        let stepped = self.as_mut().rust_mut().view.shoulder(by);
        if stepped {
            self.shown(true);
        }
        stepped
    }

    pub fn choose_inline_pill(mut self: Pin<&mut Self>, index: i32) -> bool {
        let changed = self.as_mut().rust_mut().view.choose_inline_pill(index);
        if changed {
            self.shown(true);
        }
        changed
    }

    fn read_if_named(self: Pin<&mut Self>) {
        if !self.platform_library.is_empty() {
            self.refresh();
        }
    }

    /// What a press did carried out; whether it did anything.
    fn outcome(self: Pin<&mut Self>, outcome: Outcome) -> bool {
        let did = !matches!(outcome, Outcome::Nothing);
        self.carry(outcome);
        did
    }

    fn carry(mut self: Pin<&mut Self>, outcome: Outcome) {
        if matches!(outcome, Outcome::Saved(..) | Outcome::Set { .. } | Outcome::NewCard(_)) {
            self.as_mut().said(String::new());
        }
        match outcome {
            Outcome::Nothing => {}
            Outcome::Saved(model, document) => {
                let was_reading = self.reading;
                let asked = {
                    let mut r = self.as_mut().rust_mut();
                    r.saved_since_open = true;
                    if was_reading {
                        r.asked = r.asked.wrapping_add(1);
                    }
                    r.asked
                };
                self.as_mut().applied();
                self.as_mut().shown(false);
                let library = self.platform_library.to_string();
                let clients = self.clients.clone();
                let qt = self.qt_thread();
                self.clients.spawn(move || {
                    let saved = clients.platform(&library).and_then(|platform| {
                        let (file_const, schema) = match model {
                            Model::App => (FILE_SETTINGS, MODEL_APP_SETTINGS),
                            Model::Graphics => (FILE_GRAPHICS, MODEL_GRAPHICS_CONFIG),
                        };
                        let file = platform.file(file_const).map_err(|e| e.to_string())?;
                        let name = file.file_name().and_then(|n| n.to_str()).ok_or_else(|| "missing settings file name".to_owned())?;
                        platform.model_save(schema, None, name, &document).map_err(|e| e.to_string())
                    });
                    let _ = qt.queue(move |mut model| match saved {
                        Ok(()) => {
                            if was_reading && model.asked == asked {
                                model.refresh();
                            }
                        }
                        Err(e) => {
                            if was_reading && model.asked == asked {
                                model.as_mut().rust_mut().reading = false;
                            }
                            model.as_mut().said(format!("The settings could not be saved: {e}"));
                        }
                    });
                });
            }
            Outcome::Opened => self.shown(true),
            Outcome::Shown => self.shown(false),
            Outcome::Door(name) => self.door(QString::from(name.as_str())),
            Outcome::OpenFolder(path) => self.open_folder(QString::from(path.as_str())),
            Outcome::ChooseFolder { key, start } => self.choose_folder(QString::from(key.as_str()), QString::from(start.as_str())),
            Outcome::Set { .. } if !self.world.is_empty() => {
                let page_changed = list(self.view.trail().into_iter()) != self.trail;
                self.shown(page_changed);
            }
            Outcome::NewCard(_) | Outcome::Cards { .. } if !self.world.is_empty() => self.shown(false),
            Outcome::Cards { op, args } => {
                let (library, cores) = (self.platform_library.to_string(), self.cores.to_string());
                let clients = self.clients.clone();
                let qt = self.qt_thread();
                self.clients.spawn(move || {
                    let (done, words) = match clients.consoles(&library, &cores, false) {
                        Ok(c) => c.cards_change(op, &args),
                        Err(e) => (false, e),
                    };
                    let _ = qt.queue(move |mut model| {
                        model.as_mut().rust_mut().view.cards_said(op, done, words);
                        model.as_mut().shown(true);
                        model.refresh();
                    });
                });
            }
            Outcome::Set { console, key, value } => {
                let page_changed = list(self.view.trail().into_iter()) != self.trail;
                self.as_mut().shown(page_changed);
                let was_reading = self.reading;
                let asked = if was_reading {
                    let next = self.asked.wrapping_add(1);
                    self.as_mut().rust_mut().asked = next;
                    next
                } else {
                    self.asked
                };
                let (library, cores) = (self.platform_library.to_string(), self.cores.to_string());
                let clients = self.clients.clone();
                let qt = self.qt_thread();
                let engine_change = key == crate::consoles::ENGINE_KEY;
                self.clients.spawn(move || {
                    let set = clients.consoles(&library, &cores, false).and_then(|c| c.set(&console, &key, value.as_deref()));
                    let _ = qt.queue(move |mut model| {
                        let failed = set.is_err();
                        model.as_mut().said(set.err().map(|e| format!("The console's setting could not be stored: {e}")).unwrap_or_default());
                        if failed || engine_change || model.reading || (was_reading && model.asked == asked) {
                            model.refresh();
                        }
                    });
                });
            }
            Outcome::NewCard(path) => {
                let (library, cores) = (self.platform_library.to_string(), self.cores.to_string());
                let clients = self.clients.clone();
                let qt = self.qt_thread();
                self.clients.spawn(move || {
                    let made = clients.consoles(&library, &cores, false).and_then(|c| c.new_card(&path));
                    let _ = qt.queue(move |mut model| {
                        model.as_mut().said(made.err().map(|e| format!("The memory card could not be made: {e}")).unwrap_or_default());
                        model.refresh();
                    });
                });
            }
            Outcome::Reread => self.refresh(),
            Outcome::Lever { console, id, value } => self.lever(QString::from(console.as_str()), QString::from(id.as_str()), f64::from(value)),
            Outcome::Failed(words) => {
                self.as_mut().said(words);
                self.shown(false);
            }
            Outcome::Bindings(files) => {
                let page_changed = list(self.view.trail().into_iter()) != self.trail;
                self.as_mut().shown(page_changed);
                let library = self.platform_library.to_string();
                let bindings = self.view.read.bindings.clone();
                let qt = self.qt_thread();
                self.clients.spawn(move || {
                    let saved = ClientSettings::open(&library).and_then(|s| s.save_bindings(&bindings, &files));
                    let _ = qt.queue(move |mut model| {
                        // SAFETY: `game_pads` is `PadReader.gamePads` outliving SettingsModel.
                        if let Some(pads) = unsafe { emusen_qt_endymion::game::GamePads::at(model.game_pads) } {
                            pads.rebind();
                        }
                        model.as_mut().said(saved.err().map(|e| format!("The bindings could not be saved: {e}")).unwrap_or_default());
                        model.refresh();
                    });
                });
            }
            Outcome::Seat(ask) => {
                // SAFETY: the address is PadReader.gamePads, whose reader outlives the screens.
                if let Some(pads) = unsafe { emusen_qt_endymion::game::GamePads::at(self.game_pads) } {
                    pads.ask(ask);
                }
            }
            Outcome::Type { title, prompt, start, typed } => {
                let asked = {
                    let secret = matches!(typed, crate::settings_controllers::Typed::ScrapePassword);
                    let mut r = self.as_mut().rust_mut();
                    r.type_asked = r.type_asked.wrapping_add(1);
                    r.typed_for = Some(typed);
                    r.typing = true;
                    r.typing_secret = secret;
                    r.typing_title = QString::from(title.as_str());
                    r.typing_prompt = QString::from(prompt.as_str());
                    r.type_asked
                };
                self.as_mut().asked();
                let qt = self.qt_thread();
                let _ = qt.queue(move |model| {
                    if model.type_asked == asked && model.typing {
                        model.type_words(QString::from(start.as_str()));
                    }
                });
            }
            Outcome::UseTheme(id) => self.use_theme_in_settings(id),
            Outcome::ThemeCheck(id) => self.check_theme_in_settings(id),
            Outcome::ThemeStepChoice { choice, by } => self.step_theme_choice(choice, by),
            Outcome::ScrapeRefresh => self.refresh_scrape(),
            Outcome::ScrapeSignIn { user, password } => self.scrape_sign_in(user, password),
            Outcome::ScrapeSignOut => self.scrape_sign_out(),
            Outcome::ScrapeCheck => self.scrape_check(),
            Outcome::ScrapeResume => self.scrape_resume(),
            Outcome::ScrapeCancel => self.scrape_cancel(),
            Outcome::All(all) => {
                for one in all {
                    self.as_mut().outcome(one);
                }
            }
        }
    }

    fn said(mut self: Pin<&mut Self>, words: String) {
        let words = QString::from(words.as_str());
        if self.problem != words {
            self.as_mut().rust_mut().problem = words;
            self.problem_changed();
        }
    }

    /// What the settings ask of the whole program, from the values read or changed.
    fn applied(mut self: Pin<&mut Self>) {
        let (size, light, reduced) = (QString::from(self.view.text_size().as_str()), self.view.light(), self.view.reduced_motion());
        if self.text_size != size {
            self.as_mut().rust_mut().text_size = size;
            self.as_mut().text_size_changed();
        }
        if self.light != light {
            self.as_mut().rust_mut().light = light;
            self.as_mut().light_changed();
        }
        if self.reduced_motion != reduced {
            self.as_mut().rust_mut().reduced_motion = reduced;
            self.as_mut().reduced_motion_changed();
        }
        let (swap, notices, rumble) = (self.view.swap_buttons(), self.view.controller_notices(), self.view.rumble());
        let family = self.view.controller_family().map_or(-1, |f| emusen_kit_qt::kinds::family_of(f).repr);
        if self.swap_buttons != swap {
            self.as_mut().rust_mut().swap_buttons = swap;
            self.as_mut().swap_buttons_changed();
        }
        if self.controller_family != family {
            self.as_mut().rust_mut().controller_family = family;
            self.as_mut().controller_family_changed();
        }
        if self.controller_notices != notices {
            self.as_mut().rust_mut().controller_notices = notices;
            self.as_mut().controller_notices_changed();
        }
        if self.rumble != rumble {
            self.as_mut().rust_mut().rumble = rumble;
            self.as_mut().rumble_changed();
        }
        let play = self.view.play();
        let output = QString::from(&play.output);
        let now = (self.volume, &self.output, self.sync_to_display, self.pause_in_background, self.show_fps, self.show_status_text, self.show_status_bar);
        if now != (play.volume, &output, play.sync_to_display, play.pause_in_background, play.show_fps, play.show_status_text, play.show_status_bar) {
            let mut model = self.as_mut().rust_mut();
            (model.volume, model.output, model.sync_to_display, model.pause_in_background) = (play.volume, output, play.sync_to_display, play.pause_in_background);
            (model.show_fps, model.show_status_text, model.show_status_bar) = (play.show_fps, play.show_status_text, play.show_status_bar);
            self.as_mut().play_changed();
        }
        let volume = self.view.navigation_volume();
        if self.navigation_volume != volume {
            self.as_mut().rust_mut().navigation_volume = volume;
            self.as_mut().navigation_volume_changed();
        }
        if let Some(hours) = self.view.play_hours().filter(|h| *h != self.play_hours) {
            self.as_mut().rust_mut().play_hours = hours;
            self.as_mut().play_hours_changed();
        }
        let chords = self.view.guide_chords(self.game_mode);
        if self.guide_chords != chords {
            self.as_mut().rust_mut().guide_chords = chords;
            self.as_mut().guide_chords_changed();
        }
        self.as_mut().settle_pads();
        if !self.ready {
            self.as_mut().rust_mut().ready = true;
            self.ready_changed();
        }
    }

    /// The settings the pads act on handed to the pads' reader, once read (Mistress's `SaveApp`: deadzone, stickdpad, mirror).
    fn settle_pads(mut self: Pin<&mut Self>) {
        // SAFETY: the address is PadReader.gamePads, whose reader outlives the screens.
        let pads = unsafe { emusen_qt_endymion::game::GamePads::at(self.game_pads) };
        if let Some(words) = crate::pads_reach::settle(&self.view, pads.as_deref()) {
            self.as_mut().said(words);
        }
    }

    /// The rows read again from the view: in place where the same rows stand, so a stepped value keeps its delegate,
    /// else reset with the focus on the first row.
    fn shown(mut self: Pin<&mut Self>, turned: bool) {
        let rows = self.view.rows();
        let same = !turned && rows.len() == self.rows.len() && rows.iter().zip(&self.rows).all(|(a, b)| a.id == b.id);
        if same {
            self.as_mut().rust_mut().rows = rows;
            let count = self.rows.len() as i32;
            if count > 0 {
                let (first, last) = (self.index(0, 0, &QModelIndex::default()), self.index(count - 1, 0, &QModelIndex::default()));
                self.as_mut().data_changed(&first, &last, &QVector::<i32>::default());
            }
        } else if !turned {
            self.as_mut().replace_rows(rows);
        } else {
            // The rows go and the new ones come as a removal and an insertion, not a reset: a reset cancels a delegate
            // the view is still making past the new count, which Qt logs as "index out range" (EmuSen_Qt.md §7.7).
            let root = QModelIndex::default();
            let (old, new) = (self.rows.len() as i32, rows.len() as i32);
            if old > 0 {
                self.as_mut().begin_remove_rows(&root, 0, old - 1);
                self.as_mut().rust_mut().rows.clear();
                self.as_mut().end_remove_rows();
            }
            if new > 0 {
                self.as_mut().begin_insert_rows(&root, 0, new - 1);
                self.as_mut().rust_mut().rows = rows;
                self.as_mut().end_insert_rows();
            }
            if self.focused != 0 {
                self.as_mut().rust_mut().focused = 0;
                self.as_mut().focused_changed();
            }
        }
        let focused = self.focused.max(0) as usize;
        self.as_mut().rust_mut().view.focused_row = focused;
        let trail = list(self.view.trail().into_iter());
        if self.trail != trail {
            self.as_mut().rust_mut().trail = trail;
            self.as_mut().trail_changed();
        }
        let help = shown_screen(self.view.help());
        if self.help != help {
            self.as_mut().rust_mut().help = help;
            self.as_mut().help_changed();
        }
        self.as_mut().draw();
        self.pane();
    }

    /// The same page's rows grown or shrunk under the focus (a crop's edges, a read or facts landing): only the rows
    /// that differ go and come, every other row keeps its delegate, and the focus stays on its row (EmuSen_Qt.md §7.7.E).
    fn replace_rows(mut self: Pin<&mut Self>, rows: Vec<settings::Row>) {
        let root = QModelIndex::default();
        let (old, new) = (self.rows.len(), rows.len());
        let head = self.rows.iter().zip(&rows).take_while(|(a, b)| a.id == b.id).count();
        let tail = self.rows.iter().rev().zip(rows.iter().rev()).take(old.min(new) - head).take_while(|(a, b)| a.id == b.id).count();
        let focused = usize::try_from(self.focused).ok().filter(|&f| f < old);
        let to = focused
            .and_then(|f| rows.iter().position(|r| r.id == self.rows[f].id))
            .or_else(|| focused.map(|f| f.min(new.saturating_sub(1))))
            .unwrap_or(0);
        if old - tail > head {
            self.as_mut().begin_remove_rows(&root, head as i32, (old - tail - 1) as i32);
            self.as_mut().rust_mut().rows.drain(head..old - tail);
            self.as_mut().end_remove_rows();
        }
        if new - tail > head {
            self.as_mut().begin_insert_rows(&root, head as i32, (new - tail - 1) as i32);
            let came: Vec<settings::Row> = rows[head..new - tail].to_vec();
            self.as_mut().rust_mut().rows.splice(head..head, came);
            self.as_mut().end_insert_rows();
        }
        self.as_mut().rust_mut().rows = rows;
        if new > 0 {
            let (first, last) = (self.index(0, 0, &QModelIndex::default()), self.index(new as i32 - 1, 0, &QModelIndex::default()));
            self.as_mut().data_changed(&first, &last, &QVector::<i32>::default());
        }
        self.refocus(to as i32);
    }

    /// The bindings page's drawing from the view.
    fn draw(mut self: Pin<&mut Self>) {
        let drawn = self.view.drawn();
        let shown = drawn.is_some();
        let d = drawn.unwrap_or_default();
        let (listening, listening_pad, testing) = (self.view.listening(), self.view.listening_pad(), self.view.testing());
        let hold = self.view.tester_hold_ms();
        let card = serde_json::Value::Array(self.view.game_rows().iter().map(|g| g.to_json()).collect()).to_string();
        let card_settings = self.as_mut().rust_mut().view.card_rows().to_json().to_string();
        let pad_board = self.as_mut().rust_mut().view.pad_board();
        let bindings_keys = pad_board.as_ref().and_then(|v| v["keys"].as_bool()).unwrap_or(false);
        let bindings_from_card = pad_board.as_ref().and_then(|v| v["fromCard"].as_bool()).unwrap_or(false);
        let (pad_svg, pad_focus, pad_header_focus, pad_left_scroll_y, pad_right_scroll_y, pad_left_rows, pad_right_rows, pad_meta_json, bindings_pad) =
            split_pad_board(pad_board.as_ref());
        let mut scalar_dirty = false;
        let mut rows_dirty = false;
        let mut meta_variant = None;
        {
            let mut r = self.as_mut().rust_mut();
            r.bindings_shown = shown;
            r.bindings_keys = bindings_keys;
            r.bindings_from_card = bindings_from_card;
            r.bindings_pad = QString::from(bindings_pad.as_str());
            if r.pad_meta_json != pad_meta_json {
                r.pad_meta_json = pad_meta_json;
                meta_variant = Some(json_to_variant(&r.pad_meta_json));
            }
            if let Some(mv) = meta_variant {
                r.pad_meta = mv;
            }
            let svg_q = QString::from(pad_svg.as_str());
            if r.pad_svg != svg_q || r.pad_focus != pad_focus || r.pad_header_focus != pad_header_focus || (r.pad_left_scroll_y - pad_left_scroll_y).abs() > 1e-6 || (r.pad_right_scroll_y - pad_right_scroll_y).abs() > 1e-6 {
                r.pad_svg = svg_q;
                r.pad_focus = pad_focus;
                r.pad_header_focus = pad_header_focus;
                r.pad_left_scroll_y = pad_left_scroll_y;
                r.pad_right_scroll_y = pad_right_scroll_y;
                scalar_dirty = true;
            }
            let left_q = QString::from(pad_left_rows.as_str());
            let right_q = QString::from(pad_right_rows.as_str());
            if r.pad_left_rows != left_q || r.pad_right_rows != right_q {
                r.pad_left_rows = left_q;
                r.pad_right_rows = right_q;
                rows_dirty = true;
            }
            r.drawing = QString::from(d.drawing.as_str());
            r.drawing_controls = list(d.controls.into_iter());
            r.drawing_texts = list(d.texts.into_iter());
            r.drawing_focus = d.focus;
            r.drawing_held = list(d.held.into_iter());
            r.drawing_pushes = QVariant::from(&d.pushes.iter().copied().collect::<QList<f64>>());
            r.tester_back = r.tester_back && testing;
            r.tester_hold_ms = hold;
            r.binding_tabs = list(d.tabs.into_iter());
            r.binding_caption = QString::from(d.caption.as_str());
            r.listening = listening;
            r.listening_pad = listening_pad;
            r.testing = testing;
            r.card_controllers = QString::from(card.as_str());
            r.card_settings = QString::from(card_settings.as_str());
        }
        let problem = self.view.controllers_problem();
        if let Some(words) = problem {
            self.as_mut().said(words);
        }
        if rows_dirty {
            self.as_mut().pad_rows_changed();
        }
        if scalar_dirty {
            self.as_mut().pad_scalar_changed();
        }
        self.drawn();
    }

    pub fn card_act(mut self: Pin<&mut Self>, index: i32, by: i32) {
        let outcome = self.as_mut().rust_mut().view.game_act(index.max(0) as usize, by);
        let opened = matches!(&outcome, Outcome::Door(d) if d == crate::settings_controllers::CARD_BINDINGS);
        if opened {
            self.as_mut().shown(true);
        }
        self.outcome(outcome);
    }

    pub fn open_console(mut self: Pin<&mut Self>, console: &QString) -> bool {
        if self.view.running_row(&console.to_string()).is_none() {
            return false;
        }
        // The strip's section first, whose change closes every page, then the console's page over it.
        let section = self.view.section_of("consoles") as i32;
        self.as_mut().set_section(section);
        let opened = self.as_mut().rust_mut().view.open_console(&console.to_string());
        self.draw();
        opened
    }

        pub fn card_open(mut self: Pin<&mut Self>, page: &QString, console: &QString) {
        let Some(page) = crate::settings_card::CardPage::of(&page.to_string()) else { return };
        self.as_mut().rust_mut().view.card_open(page, &console.to_string());
        self.draw();
    }

    pub fn card_settings_act(mut self: Pin<&mut Self>, index: i32, by: i32) -> bool {
        let outcome = self.as_mut().rust_mut().view.card_settings_act(index.max(0) as usize, by);
        let parts = match &outcome {
            Outcome::All(all) => all.clone(),
            one => vec![one.clone()],
        };
        // A lever is set on the picture's chain by itself (`lever`); any other display setting stored reads it again.
        let picture = parts.iter().any(|o| matches!(o, Outcome::Saved(crate::settings::Model::Graphics, _)))
            && !parts.iter().any(|o| matches!(o, Outcome::Lever { .. }));
        self.outcome(outcome);
        picture
    }

    pub fn card_back(mut self: Pin<&mut Self>) {
        if self.as_mut().rust_mut().view.card_back() {
            self.draw();
        }
    }

    pub fn typed(mut self: Pin<&mut Self>, words: &QString) {
        let Some(typed) = self.as_mut().rust_mut().typed_for.take() else { return };
        if self.shaders_shown && matches!(typed, crate::settings_controllers::Typed::SearchParameters) {
            {
                let mut r = self.as_mut().rust_mut();
                r.type_asked = r.type_asked.wrapping_add(1);
                r.typing = false;
                r.typing_secret = false;
            }
            self.as_mut().asked();
            self.shaders_apply_search(words.to_string());
            return;
        }
        let outcome = self.as_mut().rust_mut().view.typed(&typed, &words.to_string());
        if !matches!(outcome, Outcome::Type { .. }) {
            {
                let mut r = self.as_mut().rust_mut();
                r.type_asked = r.type_asked.wrapping_add(1);
                r.typing = false;
                r.typing_secret = false;
            }
            self.as_mut().asked();
        }
        self.outcome(outcome);
    }

    pub fn seats_changed(mut self: Pin<&mut Self>) {
        // SAFETY: the address is PadReader.gamePads, whose reader outlives the screens.
        let Some(pads) = (unsafe { emusen_qt_endymion::game::GamePads::at(self.game_pads) }) else { return };
        let seated = crate::settings_controllers::Seated { players: pads.players(), pads: pads.connected(), deadzone: pads.stick_deadzone() };
        let before = std::mem::take(&mut self.as_mut().rust_mut().pressed);
        let now: Vec<(u64, u32)> = seated.pads.iter().map(|p| (p.key, p.buttons)).collect();
        self.as_mut().rust_mut().pressed = now.clone();
        let fresh = now.iter().find_map(|(key, mask)| {
            let was = before.iter().find(|(k, _)| k == key).map_or(0, |(_, m)| *m);
            let down = mask & !was;
            (down != 0).then_some(down)
        });
        let (held, pushes) = (self.view.held_from(&seated), self.view.pushes_from(&seated));
        let held = match held {
            Ok(held) => held,
            Err(words) => {
                self.as_mut().said(words);
                Vec::new()
            }
        };
        {
            let mut r = self.as_mut().rust_mut();
            r.view.seats = seated;
            r.view.held = held;
            r.view.pushes = pushes;
        }
        if let (true, Some(down)) = (self.view.listening_pad(), fresh) {
            let outcome = self.as_mut().rust_mut().view.heard_down(down);
            self.outcome(outcome);
            return;
        }
        self.shown(false);
    }

    pub fn binding_move(mut self: Pin<&mut Self>, dx: i32, dy: i32) {
        if self.as_mut().rust_mut().view.binding_move(dx, dy) {
            self.shown(false);
        }
    }

    pub fn binding_focus(mut self: Pin<&mut Self>, index: i32) {
        if self.as_mut().rust_mut().view.binding_focus(index.max(0) as usize) {
            self.shown(false);
        }
    }

    pub fn binding_accept(mut self: Pin<&mut Self>) {
        if self.as_mut().rust_mut().view.binding_accept() {
            // The buttons held as listening starts are not the press listened for.
            self.as_mut().seats_changed();
            self.shown(false);
        }
    }

    pub fn binding_player(mut self: Pin<&mut Self>, player: i32) -> bool {
        let changed = self.as_mut().rust_mut().view.binding_player(player.max(0) as u32);
        if changed {
            self.shown(false);
        }
        changed
    }

    pub fn binding_console(mut self: Pin<&mut Self>, console: &QString) -> bool {
        let changed = self.as_mut().rust_mut().view.binding_console(&console.to_string());
        if changed {
            self.shown(false);
        }
        changed
    }

    pub fn binding_nav(mut self: Pin<&mut Self>, tab: i32) -> bool {
        let changed = self.as_mut().rust_mut().view.binding_nav(tab);
        if changed {
            self.shown(true);
        }
        changed
    }

    pub fn binding_page_step(mut self: Pin<&mut Self>, by: i32) -> bool {
        let changed = self.as_mut().rust_mut().view.binding_page_step(by);
        if changed {
            self.shown(true);
        }
        changed
    }

    pub fn binding_settings(mut self: Pin<&mut Self>) -> bool {
        let changed = self.as_mut().rust_mut().view.binding_settings();
        if changed {
            self.shown(true);
        }
        changed
    }

    pub fn binding_pad_button(mut self: Pin<&mut Self>, button: qobject::PadButton, repeat: bool) -> bool {
        use emusen_kit_core::input::UiButton;
        let Some(ui) = emusen_kit_qt::kinds::ui_button(button) else { return false };
        if !self.view.binding_open() {
            return false;
        }
        let dir = match ui {
            UiButton::Left => Some((-1, 0)),
            UiButton::Right => Some((1, 0)),
            UiButton::Up => Some((0, -1)),
            UiButton::Down => Some((0, 1)),
            _ => None,
        };
        if let Some((dx, dy)) = dir {
            if self.as_mut().rust_mut().view.binding_step(dx, dy) {
                self.shown(false);
            }
            return true;
        }
        if repeat {
            return false;
        }
        match ui {
            UiButton::Accept => {
                let (changed, started) = self.as_mut().rust_mut().view.binding_enter();
                if started {
                    self.as_mut().seats_changed();
                    self.shown(false);
                } else if changed {
                    self.shown(false);
                }
                true
            }
            UiButton::Back => {
                self.back();
                true
            }
            UiButton::Search => {
                let outcome = self.as_mut().rust_mut().view.binding_act(crate::help::RESET);
                self.outcome(outcome)
            }
            UiButton::Screensaver => {
                let outcome = self.as_mut().rust_mut().view.binding_act(crate::help::CLEAR);
                self.outcome(outcome)
            }
            UiButton::PageUp | UiButton::PageDown => {
                let by = if ui == UiButton::PageUp { -1 } else { 1 };
                if self.bindings_keys {
                    self.shoulder(by)
                } else {
                    self.binding_page_step(by)
                }
            }
            UiButton::First | UiButton::Last if !self.bindings_keys => {
                let by = if ui == UiButton::First { -1 } else { 1 };
                self.shoulder(by)
            }
            _ => false,
        }
    }

    pub fn act(mut self: Pin<&mut Self>, key: &QString) {
        let words = key.to_string();
        let outcome = if self.view.binding_open() {
            self.as_mut().rust_mut().view.binding_act(&words)
        } else if words == crate::help::CLEAR {
            let focused = self.focused.max(0) as usize;
            self.as_mut().rust_mut().view.clear_hotkey(focused)
        } else if words == crate::help::CHECK {
            let focused = self.focused.max(0) as usize;
            self.view.check_focused_theme(focused)
        } else {
            Outcome::Nothing
        };
        if outcome == Outcome::Nothing {
            self.passed(key.clone());
            return;
        }
        self.outcome(outcome);
    }

    pub fn pad_pressed(self: Pin<&mut Self>, button: qobject::PadButton) {
        if emusen_kit_qt::kinds::ui_button(button) == Some(emusen_kit_core::input::UiButton::Back) {
            self.back();
        }
    }

    pub fn heard_key(mut self: Pin<&mut Self>, key: i32, modifiers: i32) -> bool {
        if self.view.testing() {
            return self.tester_key(key, modifiers, true);
        }
        if !self.view.listening() || self.view.listening_pad() {
            return false;
        }
        let outcome = match emusen_qt_endymion::keys::heard(key, modifiers) {
            emusen_qt_endymion::keys::Heard::Nothing => return true,
            emusen_qt_endymion::keys::Heard::Escape => self.as_mut().rust_mut().view.heard_key(None),
            emusen_qt_endymion::keys::Heard::Key(word) => self.as_mut().rust_mut().view.heard_key(Some(word)),
        };
        self.outcome(outcome);
        true
    }

    pub fn released_key(self: Pin<&mut Self>, key: i32, modifiers: i32) -> bool {
        self.view.testing() && self.tester_key(key, modifiers, false)
    }

    /// A key down or up while the tester runs: Escape stops it, any other key lights what it is bound to.
    fn tester_key(mut self: Pin<&mut Self>, key: i32, modifiers: i32, down: bool) -> bool {
        match emusen_qt_endymion::keys::heard(key, modifiers) {
            emusen_qt_endymion::keys::Heard::Nothing => {}
            emusen_qt_endymion::keys::Heard::Escape if down => {
                self.as_mut().back();
            }
            emusen_qt_endymion::keys::Heard::Escape => {}
            emusen_qt_endymion::keys::Heard::Key(word) => {
                match self.as_mut().rust_mut().view.tester_key(word, down) {
                    Ok(true) => self.shown(false),
                    Ok(false) => {}
                    Err(words) => {
                        self.as_mut().said(words);
                        self.shown(false);
                    }
                }
            }
        }
        true
    }

    pub fn tester_button(mut self: Pin<&mut Self>, button: qobject::PadButton, down: bool, swap: bool) -> bool {
        let back = emusen_kit_qt::kinds::ui_button(button).map(|b| b.swapped(swap)) == Some(emusen_kit_core::input::UiButton::Back);
        let held = back && down && self.view.testing();
        if back && self.tester_back != held {
            self.as_mut().rust_mut().tester_back = held;
            self.as_mut().drawn();
        }
        !down
    }

    /// Uses a checked theme chosen on Settings -> Library -> Themes (#61).
    fn use_theme_in_settings(self: Pin<&mut Self>, id: String) {
        let Some((kind, store, folder)) = self.view.themes.as_ref().and_then(|l| {
            let row = l.rows.iter().find(|r| r.id == id && r.usable())?;
            let folder = l.store.as_ref().filter(|_| !crate::themes::is_built_in(&row.id)).map(|s| s.join(&row.id));
            Some((row.kind, l.store.clone(), folder))
        }) else {
            return;
        };
        let root = folder.as_deref().map(emusen_esde::check::theme_root);
        let library = self.platform_library.to_string();
        let clients = self.clients.clone();
        let qt = self.qt_thread();
        let expected_section = self.section;
        let _ = expected_section;
        self.clients.spawn(move || {
            let saved = clients.platform(&library).and_then(|p| {
                crate::clients::change_app_settings(&p, |settings| match (kind, &root) {
                    (crate::themes::Kind::Esde, Some(root)) => crate::themes::choose_esde(settings, root),
                    _ => crate::themes::choose_own(settings, &id),
                })
            });
            let _ = qt.queue(move |mut model| {
                let settings = match saved {
                    Ok(s) => s,
                    Err(words) => return model.said(format!("The theme could not be stored: {words}")),
                };
                {
                    let mut r = model.as_mut().rust_mut();
                    r.view.read.app = settings.clone();
                    r.saved_since_open = true;
                    if let Some(listing) = r.view.themes.as_mut() {
                        listing.esde = crate::themes::mark(&mut listing.rows, &settings, store.as_deref());
                        listing.chosen = crate::themes::chosen_of(&settings);
                    }
                }
                model.as_mut().shown(false);
                if model.reading {
                    model.as_mut().refresh();
                }
                if kind == crate::themes::Kind::EmuSen {
                    let host_folder = crate::themes::resolve(&id, store.as_deref()).host_folder();
                    model.as_mut().theme_used(QString::from(host_folder.as_str()));
                }
                model.theme_restyled(kind == crate::themes::Kind::Esde);
            });
        });
    }

    /// Checks a non-built-in theme package off the GUI thread on Settings -> Library -> Themes (#61).
    fn check_theme_in_settings(mut self: Pin<&mut Self>, id: String) {
        let Some((at, kind, folder)) = self.view.themes.as_ref().and_then(|l| {
            let at = l.rows.iter().position(|r| r.id == id)?;
            let row = &l.rows[at];
            if crate::themes::is_built_in(&row.id) || row.checked == crate::themes::Checked::Running {
                return None;
            }
            Some((at, row.kind, l.store.as_ref()?.join(&row.id)))
        }) else {
            return;
        };
        if let Some(listing) = self.as_mut().rust_mut().view.themes.as_mut() {
            listing.rows[at].checked = crate::themes::Checked::Running;
        }
        self.as_mut().shown(false);
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let checked = crate::themes::check(kind, &folder);
            let _ = qt.queue(move |mut model| {
                if let Some(row) = model.as_mut().rust_mut().view.themes.as_mut().and_then(|l| l.rows.iter_mut().find(|r| r.id == id)) {
                    row.checked = checked;
                }
                model.shown(false);
            });
        });
    }

    /// Steps an active ES-DE theme choice off the GUI thread on Settings -> Library -> Themes (#61).
    fn step_theme_choice(self: Pin<&mut Self>, index: usize, by: i32) {
        let Some((root, mut choice)) = self.view.themes.as_ref().and_then(|l| l.esde.as_ref()).and_then(|e| Some((e.root.clone(), e.choices.get(index)?.clone()))) else {
            return;
        };
        let value = choice.step(by).to_owned();
        let library = self.platform_library.to_string();
        let clients = self.clients.clone();
        let qt = self.qt_thread();
        self.clients.spawn(move || {
            let saved = clients
                .platform(&library)
                .and_then(|p| crate::clients::change_app_settings(&p, |settings| crate::themes::store_choice(settings, &root, choice.key, &value)));
            let _ = qt.queue(move |mut model| match saved {
                Ok(settings) => {
                    let mut r = model.as_mut().rust_mut();
                    r.view.read.app = settings;
                    r.saved_since_open = true;
                    if let Some(esde) = r.view.themes.as_mut().and_then(|l| l.esde.as_mut()) {
                        if let Some(slot) = esde.choices.get_mut(index) {
                            *slot = choice;
                        }
                    }
                    model.as_mut().shown(false);
                    if model.reading {
                        model.as_mut().refresh();
                    }
                    model.theme_restyled(true);
                }
                Err(words) => model.said(format!("The choice could not be stored: {words}")),
            });
        });
    }

    /// Reads live ScreenScraper account, queue, run and quota state off the GUI thread (#62).
    fn refresh_scrape(self: Pin<&mut Self>) {
        let asked = self.scrape_asked.wrapping_add(1);
        let library = self.platform_library.to_string();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let snap = ClientSettings::open(&library).map(|s| s.scrape_snapshot()).ok();
            let _ = qt.queue(move |mut model| {
                if model.scrape_asked != asked {
                    return;
                }
                if let Some(snap) = snap {
                    let mut r = model.as_mut().rust_mut();
                    let (note, pending) = (r.view.scrape.status_note.clone(), r.view.scrape.pending_user.clone());
                    r.view.scrape = snap;
                    r.view.scrape.status_note = note;
                    r.view.scrape.pending_user = pending;
                }
                model.shown(false);
            });
        });
    }

    /// Spawns a background ScreenScraper operation and merges its refreshed snapshot back onto the model (#62).
    fn run_scrape_task(
        mut self: Pin<&mut Self>,
        checking: bool,
        clear_said: bool,
        work: impl FnOnce(&ClientSettings) -> Result<Option<String>, String> + Send + 'static,
    ) {
        let asked = self.scrape_asked.wrapping_add(1);
        self.as_mut().rust_mut().scrape_asked = asked;
        let library = self.platform_library.to_string();
        let qt = self.qt_thread();
        if checking {
            self.as_mut().rust_mut().view.scrape.status_note = "Checking account…".to_owned();
            self.as_mut().shown(false);
        }
        std::thread::spawn(move || {
            let answer = ClientSettings::open(&library).and_then(|s| Ok((work(&s), s.scrape_snapshot())));
            let _ = qt.queue(move |mut model| {
                if model.scrape_asked != asked {
                    return;
                }
                match answer {
                    Ok((Ok(note), snap)) => {
                        let mut r = model.as_mut().rust_mut();
                        let pending = r.view.scrape.pending_user.clone();
                        r.view.scrape = snap;
                        r.view.scrape.pending_user = pending;
                        if let Some(said) = note {
                            r.view.scrape.status_note = said;
                        }
                        if clear_said {
                            model.as_mut().said(String::new());
                        }
                    }
                    Ok((Err(why), snap)) => {
                        let mut r = model.as_mut().rust_mut();
                        let pending = r.view.scrape.pending_user.clone();
                        r.view.scrape = snap;
                        r.view.scrape.pending_user = pending;
                        r.view.scrape.status_note = why;
                    }
                    Err(why) => model.as_mut().rust_mut().view.scrape.status_note = why,
                }
                model.shown(false);
            });
        });
    }

    /// Signs into a ScreenScraper account off the GUI thread (#62).
    fn scrape_sign_in(self: Pin<&mut Self>, user: String, password: String) {
        self.run_scrape_task(true, true, move |s| s.scrape_sign_in(&user, &password).map(|val| scrape_account_note(&val, "Could not sign in.")));
    }

    /// Signs out of the saved ScreenScraper account off the GUI thread (#62).
    fn scrape_sign_out(self: Pin<&mut Self>) {
        self.run_scrape_task(false, true, |s| s.scrape_sign_out().map(|_| None));
    }

    /// Re-verifies the saved ScreenScraper account off the GUI thread (#62).
    fn scrape_check(self: Pin<&mut Self>) {
        self.run_scrape_task(true, false, |s| s.scrape_check().map(|val| scrape_account_note(&val, "Could not verify account.")));
    }

    /// Resumes queued games from a stopped scrape run off the GUI thread (#62).
    fn scrape_resume(self: Pin<&mut Self>) {
        let app = self.view.read.app.clone();
        self.run_scrape_task(false, false, move |s| s.scrape_resume(&app).map(|_| None));
    }

    /// Cancels an active scrape run off the GUI thread (#62).
    fn scrape_cancel(self: Pin<&mut Self>) {
        self.run_scrape_task(false, false, |s| s.scrape_cancel().map(|_| None));
    }

    /// The focused row's pane.
    fn pane(mut self: Pin<&mut Self>) {
        let pane = self.view.pane(self.focused.max(0) as usize);
        let sentence = QString::from(pane.sentence.as_str());
        if self.sentence != sentence {
            self.as_mut().rust_mut().sentence = sentence;
            self.as_mut().sentence_changed();
        }
        // The kit's Tones are numbered in the order tools::Tone lists them.
        let tones: Vec<i32> = pane.tones.iter().map(|t| *t as i32).collect();
        let sentence_tone = tones.first().copied().unwrap_or(0);
        if self.sentence_tone != sentence_tone {
            self.as_mut().rust_mut().sentence_tone = sentence_tone;
            self.as_mut().sentence_tone_changed();
        }
        // The lines' tones before the lines, so the delegates the new lines make read theirs.
        let line_tones: Vec<i32> = tones.iter().skip(1).copied().collect();
        let lines_changed = self.line_tones != line_tones;
        self.as_mut().rust_mut().line_tones = line_tones;
        let lines = list(pane.lines.into_iter());
        if self.pane_lines != lines || lines_changed {
            self.as_mut().rust_mut().pane_lines = lines;
            self.as_mut().pane_lines_changed();
        }
        let muted = list(pane.muted.into_iter());
        if self.pane_muted != muted {
            self.as_mut().rust_mut().pane_muted = muted;
            self.as_mut().pane_muted_changed();
        }
        let preview = pane.preview.number();
        if self.preview != preview {
            self.as_mut().rust_mut().preview = preview;
            self.as_mut().preview_changed();
        }
        let caption = QString::from(pane.caption.as_str());
        if self.caption != caption {
            self.as_mut().rust_mut().caption = caption;
            self.as_mut().caption_changed();
        }
        if self.meter != pane.meter {
            self.as_mut().rust_mut().meter = pane.meter;
            self.as_mut().meter_changed();
        }
        let samples = list(pane.samples.iter().map(|(s, _)| s.clone()));
        let scales: Vec<f64> = pane.samples.iter().map(|(_, scale)| *scale).collect();
        if self.samples != samples || self.sample_scales != scales {
            // The scales first, so the delegates the new samples make read theirs.
            self.as_mut().rust_mut().sample_scales = scales;
            self.as_mut().rust_mut().samples = samples;
            self.as_mut().samples_changed();
        }
    }

    pub(crate) fn open_shaders_board(mut self: Pin<&mut Self>, return_row: usize) {
        let board = crate::settings_shaders::ShadersBoard::open_immediate(&self.view, return_row);
        let gen = self.shaders_open_gen.wrapping_add(1);
        {
            let mut r = self.as_mut().rust_mut();
            r.shaders_open_gen = gen;
            r.shaders_state = Some(board);
        }
        self.as_mut().sync_shaders_view();
        self.as_mut().spawn_shaders_open(gen, true);
    }

    fn spawn_shaders_open(mut self: Pin<&mut Self>, open_gen: u64, include_scan: bool) {
        let library = self.platform_library.to_string();
        if library.is_empty() {
            return;
        }
        let Some(board) = self.shaders_state.clone() else { return };
        let view = self.view.clone();
        let meta_gen = self.shaders_meta_gen.wrapping_add(1);
        self.as_mut().rust_mut().shaders_meta_gen = meta_gen;
        let clients = self.clients.clone();
        let qt = self.qt_thread();
        self.clients.spawn(move || {
            let platform = clients.platform(&library).ok();
            let data = crate::settings_shaders::ShadersBoard::collect_open_data(platform.as_deref(), &view, &board, include_scan);
            let _ = qt.queue(move |mut model| {
                if model.shaders_open_gen != open_gen || (!include_scan && model.shaders_meta_gen != meta_gen) {
                    return;
                }
                let took = {
                    let mut r = model.as_mut().rust_mut();
                    if let Some(mut b) = r.shaders_state.take() {
                        b.apply_open_data(data, &r.view);
                        r.shaders_state = Some(b);
                        true
                    } else {
                        false
                    }
                };
                if took {
                    model.as_mut().sync_shaders_view();
                    if model.shaders_shown {
                        model.as_mut().sync_shaders_preview(false);
                    }
                }
            });
        });
    }

    fn maybe_fetch_shaders_meta(self: Pin<&mut Self>) {
        let open_gen = self.shaders_open_gen;
        if self.shaders_state.as_ref().is_some_and(|b| b.needs_visible_meta(&self.view)) {
            self.spawn_shaders_open(open_gen, false);
        }
    }

    /// Rewinds any subpage, moves to `section`, and returns the index of `row_id` if present.
    pub(crate) fn focus_section_row(mut self: Pin<&mut Self>, section: i32, row_id: &str) -> Option<usize> {
        while !self.trail().is_empty() && self.as_mut().back() {}
        self.as_mut().set_section(section);
        self.shown_rows().iter().position(|r| r.id == row_id)
    }

    /// Opens the Eclipse #20 Shaders board directly from Desktop Settings regardless of `themed_settings`.
    pub(crate) fn open_shaders_door(mut self: Pin<&mut Self>) {
        let return_row = self.as_mut().focus_section_row(1, "shaders").unwrap_or(0);
        self.open_shaders_board(return_row);
    }

    fn close_shaders_state(mut self: Pin<&mut Self>) {
        let had = self.shaders_state.is_some() || self.shaders_shown || self.shader_preview_feed != 0;
        {
            let mut r = self.as_mut().rust_mut();
            r.shaders_open_gen = r.shaders_open_gen.wrapping_add(1);
            r.shaders_state = None;
            r.shaders_shown = false;
            r.shader_preview.release();
            r.shader_preview_feed = 0;
            r.shader_preview_pending = false;
            r.shaders_board = QString::from("{}");
            r.shaders_help = QString::from("[]");
            r.shaders_presets_json = QString::from("[]");
            r.shaders_categories_json = QString::from("[]");
            r.shaders_sliders_json = QString::from("[]");
            r.shaders_meta_json.clear();
            r.shaders_meta = QVariant::default();
            r.shaders_counter = QString::default();
        }
        if had {
            self.as_mut().shaders_lists_changed();
            self.as_mut().shaders_scalar_changed();
            self.as_mut().shaders_changed();
            self.as_mut().shader_preview_changed();
        }
    }

    fn close_shaders_to_display(mut self: Pin<&mut Self>) {
        let return_row = self.shaders_state.as_ref().map(|b| b.return_row);
        self.as_mut().close_shaders_state();
        if let Some(row) = return_row {
            self.refocus(row as i32);
        }
    }

    fn sync_shaders_scalars(mut self: Pin<&mut Self>) {
        let Some(board) = self.shaders_state.as_ref() else { return };
        let (split, comparing, zone, preset, slider, confirm, src, cat, scope) = (
            board.split,
            board.comparing,
            board.focus_zone as i32,
            board.focused_preset as i32,
            board.focused_slider as i32,
            board.confirm_remove,
            board.source as i32,
            board.category as i32,
            board.scope as i32,
        );
        let changed = (self.shaders_split - split).abs() > 1e-6
            || self.shaders_comparing != comparing
            || self.shaders_focus_zone != zone
            || self.shaders_focused_preset != preset
            || self.shaders_focused_slider != slider
            || self.shaders_confirm_open != confirm
            || self.shaders_source_idx != src
            || self.shaders_category_idx != cat
            || self.shaders_scope_idx != scope;
        if changed {
            let mut r = self.as_mut().rust_mut();
            r.shaders_split = split;
            r.shaders_comparing = comparing;
            r.shaders_focus_zone = zone;
            r.shaders_focused_preset = preset;
            r.shaders_focused_slider = slider;
            r.shaders_confirm_open = confirm;
            r.shaders_source_idx = src;
            r.shaders_category_idx = cat;
            r.shaders_scope_idx = scope;
            self.as_mut().shaders_scalar_changed();
        }
    }

    fn sync_shaders_view(mut self: Pin<&mut Self>) {
        let (shown, parts, board_ref) = {
            let mut r = self.as_mut().rust_mut();
            let Some(board) = r.shaders_state.take() else {
                r.shaders_shown = false;
                r.shaders_board = QString::from("{}");
                r.shaders_help = QString::from("[]");
                return;
            };
            let shown = !board.return_params;
            let parts = board.to_parts_in_memory(&r.view, board.comparing, true);
            let scalar_snap = (
                board.split,
                board.comparing,
                board.focus_zone as i32,
                board.focused_preset as i32,
                board.focused_slider as i32,
                board.confirm_remove,
                board.source as i32,
                board.category as i32,
                board.scope as i32,
            );
            r.shaders_state = Some(board);
            (shown, parts, scalar_snap)
        };
        let help = parts.meta["footerHelp"].to_string();
        let meta_str = parts.meta.to_string();
        let board_str = parts.board.to_string();
        let counter_str = parts.board["counter"].as_str().unwrap_or("0 of 0").to_owned();

        let (presets_q, cats_q, sliders_q, board_q, help_q, counter_q) = (
            QString::from(parts.presets_json.as_str()),
            QString::from(parts.categories_json.as_str()),
            QString::from(parts.sliders_json.as_str()),
            QString::from(board_str.as_str()),
            QString::from(help.as_str()),
            QString::from(counter_str.as_str()),
        );
        let lists_changed = self.shaders_presets_json != presets_q || self.shaders_categories_json != cats_q || self.shaders_sliders_json != sliders_q;
        let scalars_changed = (self.shaders_split - board_ref.0).abs() > 1e-6
            || self.shaders_comparing != board_ref.1
            || self.shaders_focus_zone != board_ref.2
            || self.shaders_focused_preset != board_ref.3
            || self.shaders_focused_slider != board_ref.4
            || self.shaders_confirm_open != board_ref.5
            || self.shaders_source_idx != board_ref.6
            || self.shaders_category_idx != board_ref.7
            || self.shaders_scope_idx != board_ref.8
            || self.shaders_counter != counter_q;
        let meta_dirty = self.shaders_meta_json != meta_str;
        let view_changed = self.shaders_shown != shown || self.shaders_board != board_q || self.shaders_help != help_q || meta_dirty;

        if lists_changed {
            let mut r = self.as_mut().rust_mut();
            r.shaders_presets_json = presets_q;
            r.shaders_categories_json = cats_q;
            r.shaders_sliders_json = sliders_q;
            self.as_mut().shaders_lists_changed();
        }
        if scalars_changed {
            let mut r = self.as_mut().rust_mut();
            (r.shaders_split, r.shaders_comparing, r.shaders_focus_zone, r.shaders_focused_preset, r.shaders_focused_slider) =
                (board_ref.0, board_ref.1, board_ref.2, board_ref.3, board_ref.4);
            (r.shaders_confirm_open, r.shaders_source_idx, r.shaders_category_idx, r.shaders_scope_idx, r.shaders_counter) =
                (board_ref.5, board_ref.6, board_ref.7, board_ref.8, counter_q);
            self.as_mut().shaders_scalar_changed();
        }
        if view_changed {
            let next_meta = if meta_dirty { Some(json_to_variant(&meta_str)) } else { None };
            let mut r = self.as_mut().rust_mut();
            r.shaders_shown = shown;
            r.shaders_board = board_q;
            r.shaders_help = help_q;
            if let Some(m) = next_meta {
                r.shaders_meta_json = meta_str;
                r.shaders_meta = m;
            }
            self.as_mut().shaders_changed();
        }
    }

    fn sync_shaders_preview(mut self: Pin<&mut Self>, immediate: bool) {
        let library = self.platform_library.to_string();
        let target = {
            let r = self.as_mut().rust_mut();
            let Some(board) = r.shaders_state.as_ref() else { return };
            board.preview_target_in_memory(&r.view)
        };
        let platform = if immediate && !library.is_empty() { self.clients.platform(&library).ok() } else { None };
        let (scheduled, committed, feed_addr, src, pending) = {
            let mut r = self.as_mut().rust_mut();
            let scheduled = r.shader_preview.schedule(target);
            let committed = if immediate && (scheduled || r.shader_preview.has_pending()) {
                r.shader_preview.commit_with_platform(platform.as_deref())
            } else {
                false
            };
            let feed_addr = r.shader_preview.feed_address();
            let src = QString::from(r.shader_preview.unfiltered_url());
            let pending = r.shader_preview.has_pending();
            (scheduled, committed, feed_addr, src, pending)
        };
        let props_changed = self.shader_preview_feed != feed_addr || self.shader_preview_source != src || self.shader_preview_pending != pending;
        if props_changed {
            let mut r = self.as_mut().rust_mut();
            r.shader_preview_feed = feed_addr;
            r.shader_preview_source = src;
            r.shader_preview_pending = pending;
            self.as_mut().shader_preview_changed();
        }
        if !immediate && scheduled {
            self.as_mut().shaders_preview_debounce();
        }
        if committed {
            self.as_mut().shader_preview_ready();
        }
    }

    fn mutate_shaders(mut self: Pin<&mut Self>, immediate: bool, f: impl FnOnce(&mut crate::settings_shaders::ShadersBoard, &mut SettingsView, &str) -> crate::settings_shaders::ShadersOutcome) {
        let library = self.platform_library.to_string();
        let outcome = {
            let mut r = self.as_mut().rust_mut();
            let Some(mut board) = r.shaders_state.take() else { return };
            let out = f(&mut board, &mut r.view, &library);
            r.shaders_state = Some(board);
            out
        };
        self.apply_shaders_outcome(outcome, immediate);
    }

    fn apply_shaders_outcome(mut self: Pin<&mut Self>, outcome: crate::settings_shaders::ShadersOutcome, immediate: bool) {
        use crate::settings_shaders::ShadersOutcome;
        match outcome {
            ShadersOutcome::None => return,
            ShadersOutcome::Changed | ShadersOutcome::RemoveConfirmed => {}
            ShadersOutcome::Saved(out) => {
                self.as_mut().carry(out);
            }
            ShadersOutcome::Search { title, prompt, start } => {
                self.as_mut().carry(Outcome::Type { title, prompt, start, typed: crate::settings_controllers::Typed::SearchParameters });
                return;
            }
            ShadersOutcome::OpenSettingsPage { console, stored, return_row } => {
                let outcome = self.as_mut().rust_mut().view.open_shader_settings_page(&console, &stored, return_row);
                self.as_mut().sync_shaders_view();
                self.as_mut().outcome(outcome);
                return;
            }
        }
        self.as_mut().sync_shaders_view();
        self.as_mut().maybe_fetch_shaders_meta();
        self.sync_shaders_preview(immediate);
    }

    fn shaders_apply_search(self: Pin<&mut Self>, query: String) {
        self.mutate_shaders(false, |board, view, _| {
            board.search = query;
            board.focused_preset = 0;
            board.sync_preview_stored_in_memory(view);
            crate::settings_shaders::ShadersOutcome::Changed
        });
    }

    pub fn shaders_init(mut self: Pin<&mut Self>, host_feed: u64, paused_bytes: &QByteArray) {
        if self.shaders_state.is_none() {
            return;
        }
        let library = self.platform_library.to_string();
        {
            let mut r = self.as_mut().rust_mut();
            r.shader_preview.attach(host_feed, &library, paused_bytes.as_slice());
        }
        self.as_mut().sync_shaders_view();
        self.sync_shaders_preview(true);
    }

    pub fn shaders_release(mut self: Pin<&mut Self>) {
        {
            let mut r = self.as_mut().rust_mut();
            r.shader_preview.release();
            r.shader_preview_feed = 0;
            r.shader_preview_pending = false;
        }
        self.shader_preview_changed();
    }

    pub fn commit_shader_preview(mut self: Pin<&mut Self>) {
        if self.shaders_state.is_none() {
            return;
        }
        let library = self.platform_library.to_string();
        let platform = if !library.is_empty() { self.clients.platform(&library).ok() } else { None };
        let (committed, pending, problem) = {
            let mut r = self.as_mut().rust_mut();
            let committed = r.shader_preview.commit_with_platform(platform.as_deref());
            let pending = r.shader_preview.has_pending();
            let problem = r.shader_preview.problem().to_owned();
            (committed, pending, problem)
        };
        if self.shader_preview_pending != pending {
            self.as_mut().rust_mut().shader_preview_pending = pending;
            self.as_mut().shader_preview_changed();
        }
        if !problem.is_empty() {
            self.as_mut().said(problem);
        }
        if committed {
            self.shader_preview_ready();
        }
    }

    pub fn shaders_pad_button(self: Pin<&mut Self>, button: qobject::PadButton, down: bool, repeat: bool) -> bool {
        let Some(ui) = emusen_kit_qt::kinds::ui_button(button) else { return false };
        self.shaders_ui_button(ui, down, repeat)
    }

    pub fn shaders_key_nav(self: Pin<&mut Self>, code: i32) -> bool {
        use emusen_kit_core::input::UiButton;
        let ui = match code {
            0 => UiButton::Up,
            1 => UiButton::Down,
            2 => UiButton::Left,
            3 => UiButton::Right,
            4 => UiButton::Accept,
            5 => UiButton::Back,
            6 => UiButton::Search,
            7 => UiButton::PageUp,
            8 => UiButton::PageDown,
            10 => UiButton::First,
            11 => UiButton::Last,
            _ => return false,
        };
        self.shaders_ui_button(ui, true, false)
    }

    fn shaders_ui_button(self: Pin<&mut Self>, ui: emusen_kit_core::input::UiButton, down: bool, repeat: bool) -> bool {
        use emusen_kit_core::input::UiButton;
        if !self.shaders_shown || self.shaders_state.is_none() {
            return false;
        }
        if ui == UiButton::Screensaver && !repeat {
            self.shaders_hold_compare(down);
            return true;
        }
        if !down {
            return false;
        }
        let dir = match ui {
            UiButton::Left => Some((-1, 0)),
            UiButton::Right => Some((1, 0)),
            UiButton::Up => Some((0, -1)),
            UiButton::Down => Some((0, 1)),
            _ => None,
        };
        if let Some((dx, dy)) = dir {
            if let Some((slot, true)) = self.shaders_state.as_ref().map(|b| (b.focused_slider as i32, b.focus_zone == 6 && dx != 0)) {
                self.shaders_step_slider(slot, dx);
                return true;
            }
            self.mutate_shaders(false, |board, view, _| board.nav_in_memory(view, dx, dy));
            return true;
        }
        if ui == UiButton::PageUp || ui == UiButton::PageDown {
            let delta = if ui == UiButton::PageUp { -0.08 } else { 0.08 };
            self.mutate_shaders_scalar(|board| board.step_split(delta));
            return true;
        }
        if repeat {
            return false;
        }
        match ui {
            UiButton::Accept => {
                self.mutate_shaders(false, |board, view, library| board.accept(view, library));
                true
            }
            UiButton::Back => {
                self.close_shaders_to_display();
                true
            }
            UiButton::Search => {
                self.shaders_toggle_favourite(-1);
                true
            }
            UiButton::First | UiButton::Last => {
                let by = if ui == UiButton::First { -1 } else { 1 };
                self.mutate_shaders(false, |board, view, _| {
                    if board.step_console_in_memory(view, by) { crate::settings_shaders::ShadersOutcome::Changed } else { crate::settings_shaders::ShadersOutcome::None }
                });
                true
            }
            _ => false,
        }
    }

    fn mutate_shaders_scalar(mut self: Pin<&mut Self>, f: impl FnOnce(&mut crate::settings_shaders::ShadersBoard) -> bool) {
        let changed = {
            let mut r = self.as_mut().rust_mut();
            let Some(board) = r.shaders_state.as_mut() else { return };
            f(board)
        };
        if changed {
            self.as_mut().sync_shaders_scalars();
        }
    }

    pub fn shaders_source(self: Pin<&mut Self>, index: i32) {
        self.mutate_shaders(true, |board, view, _| {
            board.focus_zone = 0;
            board.set_source_in_memory(view, index.max(0) as usize);
            crate::settings_shaders::ShadersOutcome::Changed
        });
    }

    pub fn shaders_category(self: Pin<&mut Self>, index: i32) {
        self.mutate_shaders(true, |board, view, _| {
            board.focus_zone = 1;
            board.set_category_in_memory(view, index.max(0) as usize);
            crate::settings_shaders::ShadersOutcome::Changed
        });
    }

    pub fn shaders_focus_preset(self: Pin<&mut Self>, index: i32, activate: bool) {
        self.mutate_shaders(activate, |board, view, _| {
            let visible = board.visible_items(view).len();
            if visible == 0 {
                return crate::settings_shaders::ShadersOutcome::None;
            }
            board.focus_zone = 4;
            board.focused_preset = (index.max(0) as usize).min(visible - 1);
            board.sync_preview_stored_in_memory(view);
            if activate {
                let out = board.apply_current(view);
                if out == Outcome::Nothing { crate::settings_shaders::ShadersOutcome::Changed } else { crate::settings_shaders::ShadersOutcome::Saved(out) }
            } else {
                crate::settings_shaders::ShadersOutcome::Changed
            }
        });
    }

    pub fn shaders_scope(self: Pin<&mut Self>, index: i32) {
        self.mutate_shaders(true, |board, view, _| {
            board.focus_zone = 5;
            board.set_scope_in_memory(view, index.max(0) as usize);
            crate::settings_shaders::ShadersOutcome::Changed
        });
    }

    pub fn shaders_toggle_favourite(mut self: Pin<&mut Self>, index: i32) {
        let updated = {
            let mut r = self.as_mut().rust_mut();
            let Some(mut board) = r.shaders_state.take() else { return };
            if index >= 0 {
                let visible = board.visible_items(&r.view).len();
                if visible > 0 {
                    board.focus_zone = 4;
                    board.focused_preset = (index as usize).min(visible - 1);
                    board.sync_preview_stored_in_memory(&r.view);
                }
            }
            let favs = board.toggle_favourite_in_memory(&r.view);
            r.shaders_state = Some(board);
            favs
        };
        let Some(favs) = updated else { return };
        self.as_mut().sync_shaders_view();
        self.as_mut().maybe_fetch_shaders_meta();
        self.as_mut().sync_shaders_preview(false);
        let library = self.platform_library.to_string();
        if library.is_empty() {
            return;
        }
        let clients = self.clients.clone();
        self.clients.spawn(move || {
            if let Ok(platform) = clients.platform(&library) {
                crate::settings_shaders::save_favourites_on(&platform, &favs);
            }
        });
    }

    pub fn shaders_search(self: Pin<&mut Self>) {
        self.mutate_shaders(false, |board, view, library| {
            board.focus_zone = 3;
            board.accept(view, library)
        });
    }

    pub fn shaders_all_settings(self: Pin<&mut Self>) {
        self.mutate_shaders(false, |board, view, library| {
            board.focus_zone = 7;
            board.accept(view, library)
        });
    }

    pub fn shaders_ask_remove(self: Pin<&mut Self>) {
        self.mutate_shaders(false, |board, view, library| {
            board.focus_zone = 2;
            board.accept(view, library)
        });
    }

    pub fn shaders_confirm_remove(mut self: Pin<&mut Self>, confirm: bool) {
        if !confirm {
            self.mutate_shaders(false, |board, _, _| {
                board.confirm_remove = false;
                crate::settings_shaders::ShadersOutcome::Changed
            });
            return;
        }
        let library = self.platform_library.to_string();
        let (pack_dir, favs, graphics_dirty, graphics_doc) = {
            let mut r = self.as_mut().rust_mut();
            let Some(mut board) = r.shaders_state.take() else { return };
            let dir = board.pack_dir.clone();
            let dirty = board.finish_remove_pack(&mut r.view);
            let favs = board.favourites.clone();
            let doc = r.view.read.graphics.clone();
            r.shaders_state = Some(board);
            (dir, favs, dirty, doc)
        };
        self.as_mut().said(String::new());
        self.as_mut().sync_shaders_view();
        self.as_mut().sync_shaders_preview(true);
        let clients = self.clients.clone();
        let qt = self.qt_thread();
        self.clients.spawn(move || {
            let platform = if library.is_empty() { None } else { clients.platform(&library).ok() };
            let dir = pack_dir.or_else(|| platform.as_deref().and_then(|p| crate::settings_shaders::scan_pack_dir_on(p).0));
            let deleted = match dir {
                Some(d) => crate::settings_shaders::ShadersBoard::remove_pack_dir(&d),
                None => Ok(()),
            };
            if deleted.is_ok() {
                if let Some(p) = platform.as_deref() {
                    crate::settings_shaders::save_favourites_on(p, &favs);
                    if graphics_dirty {
                        if let Ok(file) = p.file(FILE_GRAPHICS) {
                            if let Some(name) = file.file_name().and_then(|n| n.to_str()) {
                                let _ = p.model_save(MODEL_GRAPHICS_CONFIG, None, name, &graphics_doc);
                            }
                        }
                    }
                }
            }
            let _ = qt.queue(move |mut model| {
                if let Err(words) = deleted {
                    model.as_mut().said(words);
                    model.as_mut().sync_shaders_view();
                } else {
                    model.as_mut().rust_mut().saved_since_open = true;
                    model.refresh();
                }
            });
        });
    }

    fn shaders_update_slider(mut self: Pin<&mut Self>, slot: i32, res: Option<(String, f32, Outcome)>) {
        let Some((id, val, outcome)) = res else { return };
        self.as_mut().rust_mut().shader_preview.set_live_value(&id, val);
        if let Some(board) = self.as_mut().rust_mut().shaders_state.as_mut() {
            board.focus_zone = 6;
            board.focused_slider = slot.max(0) as usize;
        }
        self.as_mut().sync_shaders_view();
        self.as_mut().shader_preview_ready();
        self.carry(outcome);
    }

    pub fn shaders_step_slider(mut self: Pin<&mut Self>, slot: i32, by: i32) {
        let res = {
            let mut r = self.as_mut().rust_mut();
            let Some(mut board) = r.shaders_state.take() else { return };
            let out = board.step_slider_in_memory(&mut r.view, slot.max(0) as usize, by);
            r.shaders_state = Some(board);
            out
        };
        self.shaders_update_slider(slot, res);
    }

    pub fn shaders_slide_slider(mut self: Pin<&mut Self>, slot: i32, norm: f64) {
        let res = {
            let mut r = self.as_mut().rust_mut();
            let Some(mut board) = r.shaders_state.take() else { return };
            let out = board.slide_slider_in_memory(&mut r.view, slot.max(0) as usize, norm);
            r.shaders_state = Some(board);
            out
        };
        self.shaders_update_slider(slot, res);
    }

    pub fn shaders_set_split(self: Pin<&mut Self>, norm: f64) {
        self.mutate_shaders_scalar(|board| board.set_split(norm));
    }

    pub fn shaders_scrub_split(self: Pin<&mut Self>, delta: f64) {
        self.mutate_shaders_scalar(|board| board.step_split(delta));
    }

    pub fn shaders_hold_compare(self: Pin<&mut Self>, down: bool) {
        self.mutate_shaders_scalar(|board| {
            if board.comparing == down {
                false
            } else {
                board.comparing = down;
                true
            }
        });
    }

    pub fn shaders_back(self: Pin<&mut Self>) {
        self.close_shaders_to_display();
    }
}

fn strip_pad_volatile(arr: Option<&serde_json::Value>) -> String {
    let Some(items) = arr.and_then(serde_json::Value::as_array) else {
        return "[]".to_owned();
    };
    let mut clean = Vec::with_capacity(items.len());
    for item in items {
        let mut copy = item.clone();
        if let Some(map) = copy.as_object_mut() {
            map.remove("focused");
            map.remove("held");
        }
        clean.push(copy);
    }
    serde_json::Value::Array(clean).to_string()
}

fn split_pad_board(pad_board: Option<&serde_json::Value>) -> (String, i32, i32, f64, f64, String, String, String, String) {
    let Some(v) = pad_board else {
        return (String::new(), 0, -1, 0.0, 0.0, "[]".to_owned(), "[]".to_owned(), "{}".to_owned(), String::new());
    };
    let svg = v["svg"].as_str().unwrap_or_default().to_owned();
    let focus = v["focus"].as_i64().unwrap_or(0) as i32;
    let header_focus = v["headerFocus"].as_i64().unwrap_or(-1) as i32;
    let left_y = v["leftScrollY"].as_f64().unwrap_or(0.0);
    let right_y = v["rightScrollY"].as_f64().unwrap_or(0.0);
    let left_rows = strip_pad_volatile(v.get("left"));
    let right_rows = strip_pad_volatile(v.get("right"));
    let mut meta = v.clone();
    if let Some(m) = meta.as_object_mut() {
        for volatile_key in ["svg", "focus", "headerFocus", "leftScrollY", "rightScrollY", "left", "right"] {
            m.remove(volatile_key);
        }
    }
    (svg, focus, header_focus, left_y, right_y, left_rows, right_rows, meta.to_string(), v.to_string())
}

/// Formats a non-signed-in ScreenScraper account response into a UI status note (#62).
fn scrape_account_note(val: &serde_json::Value, fallback: &str) -> Option<String> {
    if val["result"].as_str() == Some("signed_in") {
        return None;
    }
    Some(val["said"].as_str().filter(|s| !s.is_empty()).unwrap_or_else(|| val["result"].as_str().unwrap_or(fallback)).to_owned())
}
