# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ThemeSettings-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 220,181.5,840,437
title "Theme Settings"
case ThemeSettings
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=220,282,840,317 by=panel limit=220,282,840,317 paints=#D4D4D4FF role=none
    1 DockPanel at=260,310,760,273 by=panel limit=220,282,840,317 role=none
      2 ButtonBar at=933,539,87,44 by=panel limit=220,282,840,317 paints=#D4D4D4FF role=toolbar
        3 Button at=933,539,87,44 by=clip drawn focus heard=CLOSE limit=933,539,87,44 op paints=#707070FF,#D4D4D4FF role=button text=Close
          4 TextBlock at=950,545,53,32 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=933,539,87,44 owner=3 paints=#D4D4D4FF role=text size=26 text=CLOSE
      5 Tabs at=260,310,760,217 by=panel limit=220,282,840,317 name=ThemeSettingsTabs paints=#D4D4D4FF role=tab
        6 TabItem at=260,310,106,48 by=clip drawn focus heard=OPTIONS limit=260,310,760,217 op paints=#EEEEF0FF role=tabitem sideways text=Options
          7 TextBlock at=272,316,82,36 box=82,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=OPTIONS layout=81.36,36 limit=260,310,106,48 owner=6 paints=#EEEEF0FF role=text size=30 text=OPTIONS
        8 TabItem at=366,310,100,48 by=clip drawn focus heard=THEMES limit=260,310,760,217 paints=#8D8D91FF role=tabitem sideways text=Themes
          9 TextBlock at=378,316,76,36 box=76,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=THEMES layout=75.33,36 limit=366,310,100,48 owner=8 paints=#8D8D91FF role=text size=30 text=THEMES
        10 TabItem at=466,310,127,48 by=clip drawn focus heard=INTERFACE limit=260,310,760,217 paints=#8D8D91FF role=tabitem sideways text=Interface
          11 TextBlock at=478,316,103,36 box=103,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=INTERFACE layout=102.39,36 limit=466,310,127,48 owner=10 paints=#8D8D91FF role=text size=30 text=INTERFACE
        12 ScrollViewer at=272,360,736,167 by=clip limit=260,360,760,167 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=272,360,736,167
          13 StackPanel at=276,372,728,151 by=fixed-scroll in=12 limit=272,360,736,167 role=none
            14 SectionHeader paints=#D4D4D4FF role=text volatile="its words name the run's own folders"
            15 FieldRow at=276,418,728,105 by=fixed-scroll heard=Transitions in=12 limit=272,360,736,167 paints=#D4D4D4FF role=group
              16 TextBlock at=276,418,728,29 box=728,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Transitions in=12 layout=85.08,28.8 limit=276,418,728,105 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Transitions
              17 HintText at=276,451,728,24 box=728,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="How the view changes between the systems and a game list." in=12 layout=392.8,24 limit=276,418,728,105 paints=#8D8D91FF part role=text size=20 text="How the view changes between the systems and a game list."
              18 Dropdown at=276,479,728,44 by=clip drawn focus heard=Transitions in=12 limit=276,418,728,105 name=TransitionsDropdown op paints=#0E0E10FF,#707070FF,#D4D4D4FF,#EEEEF0FF role=combobox sideways
                19 TextBlock at=288,484,684,32 box=684,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=Automatic in=12 layout=84.97,31.2 limit=288,484,684,32 owner=18 paints=#EEEEF0FF role=text size=26 text=Automatic
                20 ViewboxContainer at=982,497.73,12,6.54 by=clip in=12 limit=982,497.73,12,6.54 nohit role=none
focus 18
edge 3 up 18
edge 3 down same
edge 3 left 6
edge 3 right same
edge 6 up same
edge 6 down 18
edge 18 up 6
edge 18 down 3
press pageup focus="Dropdown QuickSystemSelect" tab=Interface title="Theme Settings" top=ThemeSettingsWindow
press pagedown focus="Dropdown TransitionsDropdown" tab=Options title="Theme Settings" top=ThemeSettingsWindow
press first focus="Dropdown TransitionsDropdown" tab=Options title="Theme Settings" top=ThemeSettingsWindow
press last focus="Dropdown TransitionsDropdown" tab=Options title="Theme Settings" top=ThemeSettingsWindow
press back top=none
