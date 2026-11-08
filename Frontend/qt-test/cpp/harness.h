// The harness's hand-written C++ (EmuSen_Qt.md §8.3), counted against §1.4's budget: QTest's verbs and the item tree as JSON. Reasons in EmuSen_Kit.md §7.9.
#pragma once
#include <QtCore/QObject>
#include <QtCore/QString>
#include <QtQml/QQmlApplicationEngine>
#include <cstdint>

bool harness_show(QObject& window, int width, int height);
void harness_resize(QObject& window, int width, int height);
QString harness_tree(QObject& window);
void harness_click(QObject& window, double x, double y, int button);
void harness_key(QObject& window, int key, int modifiers);
void harness_wheel(QObject& window, double x, double y, int delta);
void harness_wait(int ms);
void harness_keep_messages();
QString harness_messages();
std::uint64_t harness_focus(QObject& window);
bool harness_focus_item(QObject& window, std::uint64_t id);
double harness_get(QObject& window, std::uint64_t id, const QString& property);
bool harness_put(QObject& window, std::uint64_t id, const QString& property, double value);
std::uint64_t harness_scoped(QObject& window, std::uint64_t scope);
void harness_give_focus(QObject& window, std::uint64_t id);
bool harness_grab(QObject& window, const QString& path);
QString harness_property(QObject& root, const QString& object_name, const QString& property);
QObject* harness_singleton(QQmlApplicationEngine& engine, const QString& module, const QString& name);
int harness_roots(QQmlApplicationEngine& engine);
bool harness_set_property(QObject& root, const QString& object_name, const QString& property, const QString& value);
bool harness_watch(QObject& root, const QString& object_name, const QString& property);
int harness_notified(QObject& root, const QString& object_name, const QString& property);
QString harness_colours_at(QObject& window, const QString& points);
void harness_hide_words(QObject& window, bool hidden);
std::uint64_t harness_frames(QObject& window);
QString harness_running(QObject& window);
int harness_difference(QObject& window, const QString& reference, int tolerance, int reach, const QString& loose, const QString& marks);
