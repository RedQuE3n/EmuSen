// The kit's hand-written C++, counted against EmuSen_Qt.md §1.4's budget; the reasons are in EmuSen_Kit.md §7.3.
#pragma once

// Delivers a key press and release to the focused window, so a declined pad press reaches a stock control.
void send_key(int key, int modifiers);
