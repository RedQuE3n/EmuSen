// The kit's hand-written C++, counted against EmuSen_Qt.md §1.4's budget; the reasons are in EmuSen_Kit.md §7.3 and §7.24.
#pragma once

class QQuickItem;

// Delivers a key press and release to the focused window, so a declined pad press reaches a stock control; true if an item took the press.
bool send_key(int key, int modifiers);

// Offers a key press and release to each ancestor of an item in turn until one accepts the press; true if one did.
bool pass_key(QQuickItem* item, int key, int modifiers);
