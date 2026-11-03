# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-ThemeSettings-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 330,272,1260,656
title "Theme Settings"
case ThemeSettings
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=330,422,1260,477 by=panel limit=330,422,1260,476 paints=#D4D4D4FF role=none seen=330,422,1260,476
    1 DockPanel at=390,464,1140,411 by=panel limit=330,422,1260,476 role=none
      2 ButtonBar at=1399.5,809,130.5,66 by=panel limit=330,422,1260,476 paints=#D4D4D4FF role=toolbar
        3 Button at=1399.5,809,130.5,66 by=clip drawn focus heard=CLOSE limit=1399.5,809,130.5,66 op paints=#707070FF,#D4D4D4FF role=button text=Close
          4 TextBlock at=1425,818,79.5,48 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=1399.5,809,130.5,66 owner=3 paints=#D4D4D4FF role=text size=26 text=CLOSE
      5 Tabs at=390,464,1140,327 by=panel limit=330,422,1260,476 name=ThemeSettingsTabs paints=#D4D4D4FF role=tab
        6 TabItem at=390,464,159,72 by=clip drawn focus heard=OPTIONS limit=390,464,1140,327 op paints=#EEEEF0FF role=tabitem sideways text=Options
          7 TextBlock at=408,473,123,54 box=82,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=OPTIONS layout=81.36,36 limit=390,464,159,72 owner=6 paints=#EEEEF0FF role=text size=30 text=OPTIONS
        8 TabItem at=549,464,150,72 by=clip drawn focus heard=THEMES limit=390,464,1140,327 paints=#8D8D91FF role=tabitem sideways text=Themes
          9 TextBlock at=567,473,114,54 box=76,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=THEMES layout=75.33,36 limit=549,464,150,72 owner=8 paints=#8D8D91FF role=text size=30 text=THEMES
        10 TabItem at=699,464,190.5,72 by=clip drawn focus heard=INTERFACE limit=390,464,1140,327 paints=#8D8D91FF role=tabitem sideways text=Interface
          11 TextBlock at=717,473,154.5,54 box=103,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=INTERFACE layout=102.39,36 limit=699,464,190.5,72 owner=10 paints=#8D8D91FF role=text size=30 text=INTERFACE
        12 ScrollViewer at=408,539,1104,252 by=clip limit=390,539,1140,252 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=408,539,1104,252
          13 StackPanel at=414,557,1092,228 by=fixed-scroll in=12 limit=408,539,1104,252 role=none
            14 SectionHeader paints=#D4D4D4FF role=text volatile="its words name the run's own folders"
            15 FieldRow at=414,626,1092,157.5 by=fixed-scroll heard=Transitions in=12 limit=408,539,1104,252 paints=#D4D4D4FF role=group
              16 TextBlock at=414,626,1092,43.5 box=728,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Transitions in=12 layout=85.08,28.8 limit=414,626,1092,157.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Transitions
              17 HintText at=414,675.5,1092,36 box=728,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="How the view changes between the systems and a game list." in=12 layout=392.8,24 limit=414,626,1092,157.5 paints=#8D8D91FF part role=text size=20 text="How the view changes between the systems and a game list."
              18 Dropdown at=414,717.5,1092,66 by=clip drawn focus heard=Transitions in=12 limit=414,626,1092,157.5 name=TransitionsDropdown op paints=#0E0E10FF,#707070FF,#D4D4D4FF,#EEEEF0FF role=combobox sideways
                19 TextBlock at=432,725,1026,48 box=684,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=Automatic in=12 layout=84.97,31.2 limit=432,725,1026,48 owner=18 paints=#EEEEF0FF role=text size=26 text=Automatic
                20 ViewboxContainer at=1473,745.6,18,9.81 by=clip in=12 limit=1473,745.6,18,9.81 nohit role=none
focus 18
edge 3 up 18
edge 3 down same
edge 3 left 18
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
