// The kit's hand-written C++, counted against EmuSen_Qt.md §1.4's budget; the reasons are in EmuSen_Kit.md §7.3 and §7.24.
#pragma once

class QQuickItem;

// Delivers a key press and release to the focused window, so a declined pad press reaches a stock control; true if an item took the press.
bool send_key(int key, int modifiers);

// Offers a key press and release to each ancestor of an item in turn until one accepts the press; true if one did.
bool pass_key(QQuickItem* item, int key, int modifiers);

// Makes the delegate holding the item its ListView's or GridView's current item, unless it is already; nothing outside one.
void make_current(QQuickItem* item);

// Starts the pointer watch of every Qt Quick window that has none, as a frame controller is made, before any hover in it.
void watch_windows();

// True while a real move, press or touch of the pointer is being delivered to the item's window, not for a hover Qt raises as items move under a resting pointer; the window's first call starts the watch and answers true.
bool pointer_moving(QQuickItem* item);
