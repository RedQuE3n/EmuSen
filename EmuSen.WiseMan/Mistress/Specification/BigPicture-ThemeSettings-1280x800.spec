# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-ThemeSettings-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case ThemeSettings
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#FFFFFFFF role=none
    1 Grid at=0,0,1280,800 by=clip limit=0,0,1280,800 role=none
      2 MenuPanel at=0,0,1280,800 by=clip heard="Theme Settings" limit=0,0,1280,800 name=BigMenu role=menu
        3 HintBar at=416,742,448,48 by=clip heard="A Select, B Back, D-pad left and right Change, D-pad up and down Choose" limit=0,0,1280,800 role=group
        4 ScrollViewer at=220,292,840,162 by=clip limit=0,0,1280,800 overflow=0 paints=#FFFFFFFF role=scrollviewer viewport=220,292,840,162
          5 StackPanel at=220,292,840,162 by=fixed-scroll in=4 limit=220,292,840,162 name=BigMenuRows role=none
            6 Dropdown at=220,292,840,54 by=fixed-scroll drawn focus heard=Transitions in=4 limit=220,292,840,162 name=TransitionsDropdown op paints=#707070FF,#D4D4D4FF role=combobox sideways
            7 Button at=220,346,840,54 by=fixed-scroll drawn focus heard=Themes in=4 limit=220,292,840,162 name=BigMenuPage1 op paints=#707070FF,#D4D4D4FF role=button text=Themes
            8 Button at=220,400,840,54 by=fixed-scroll drawn focus heard=Interface in=4 limit=220,292,840,162 name=BigMenuPage2 op paints=#707070FF,#D4D4D4FF role=button text=Interface
        9 StackPanel at=598,466,84,46 by=clip limit=0,0,1280,800 role=none
          10 Button at=598,466,84,46 by=clip drawn focus heard=Back limit=0,0,1280,800 name=BigMenuBack op paints=#707070FF,#D4D4D4FF role=button text=Back
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
press pageup focus="Dropdown QuickSystemSelect (Quick System Select)" tab=Interface top=ThemeSettingsWindow
press pagedown focus="Dropdown TransitionsDropdown (Transitions)" tab=Options top=ThemeSettingsWindow
press first focus="Dropdown TransitionsDropdown (Transitions)" tab=Options top=ThemeSettingsWindow
press last focus="Dropdown TransitionsDropdown (Transitions)" tab=Options top=ThemeSettingsWindow
press back top=none
