# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-ThemeSettings-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case ThemeSettings
palette #000000FF #007ACCFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #6E6E6EFF #808080FF #9CDCFEFF #CD5C5CFF #D08770FF #D4D4D4FF #DAA520FF #DCDCDCFF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#FFFFFFFF role=none
    1 Grid at=0,0,1920,1200 by=clip limit=0,0,1920,1200 role=none
      2 MenuPanel at=0,0,1920,1200 by=clip heard="Theme Settings" limit=0,0,1920,1200 name=BigMenu role=menu
        3 HintBar at=614,1114,691,71 by=clip heard="A Select, B Back, D-pad left and right Change, D-pad up and down Choose" limit=0,0,1920,1200 role=group
        4 ScrollViewer at=330,439,1260,243 by=clip limit=0,0,1920,1200 overflow=0 paints=#FFFFFFFF role=scrollviewer viewport=330,439,1260,243
          5 StackPanel at=330,439,1260,243 by=fixed-scroll in=4 limit=330,439,1260,243 name=BigMenuRows role=none
            6 Dropdown at=330,439,1260,81 by=fixed-scroll drawn focus in=4 limit=330,439,1260,243 name=TransitionsDropdown op paints=#6E6E6EFF,#D4D4D4FF role=combobox sideways
            7 Button at=330,520,1260,81 by=fixed-scroll drawn focus heard=Themes in=4 limit=330,439,1260,243 name=BigMenuPage1 op paints=#6E6E6EFF,#D4D4D4FF role=button text=Themes
            8 Button at=330,601,1260,81 by=fixed-scroll drawn focus heard=Interface in=4 limit=330,439,1260,243 name=BigMenuPage2 op paints=#6E6E6EFF,#D4D4D4FF role=button text=Interface
        9 StackPanel at=896,700,128,69 by=clip limit=0,0,1920,1200 role=none
          10 Button at=896,700,128,69 by=clip drawn focus heard=Back limit=0,0,1920,1200 name=BigMenuBack op paints=#55555AFF,#6E6E6EFF,#D4D4D4FF role=button text=Back
focus 6
edge 6 up same
edge 6 down 7
edge 7 up 6
edge 7 down 8
edge 7 left same
edge 7 right same
edge 8 up 7
edge 8 down 10
edge 8 left same
edge 8 right same
edge 10 up 8
edge 10 down same
edge 10 left same
edge 10 right same
press pageup focus="Dropdown QuickSystemSelect" tab=Interface top=ThemeSettingsWindow
press pagedown focus="Dropdown TransitionsDropdown" tab=Options top=ThemeSettingsWindow
press first focus="Dropdown TransitionsDropdown" tab=Options top=ThemeSettingsWindow
press last focus="Dropdown TransitionsDropdown" tab=Options top=ThemeSettingsWindow
press back top=none
