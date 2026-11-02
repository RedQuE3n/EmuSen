# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-GameOptions-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case GameOptions
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#FFFFFFFF role=none
    1 MenuPanel at=0,0,1920,1200 by=clip heard="Gamelist Options" limit=0,0,1920,1200 name=GameOptionsMenu role=menu
      2 HintBar at=386,1114,1147,71 by=clip heard="View Close (Cancel), A Select, B Close (Apply), D-pad left and right Change Value, D-pad up and down Choose" limit=0,0,1920,1200 role=group
      3 ScrollViewer at=330,417,1260,486 by=clip limit=0,0,1920,1200 overflow=0 paints=#FFFFFFFF role=scrollviewer viewport=330,417,1260,486
        4 StackPanel at=330,417,1260,486 by=fixed-scroll in=3 limit=330,417,1260,486 role=none
          5 Dropdown at=330,417,1260,81 by=fixed-scroll drawn focus heard="Jump To..." in=3 limit=330,417,1260,486 name=GamelistJumpTo op paints=#707070FF,#D4D4D4FF role=combobox sideways
          6 Dropdown at=330,498,1260,81 by=fixed-scroll drawn focus heard="Sort Games By" in=3 limit=330,417,1260,486 name=GamelistSortBy op paints=#707070FF,#D4D4D4FF role=combobox sideways
          7 Button at=330,579,1260,81 by=fixed-scroll drawn focus heard="Filter Gamelist" in=3 limit=330,417,1260,486 name=GamelistFilterButton op paints=#707070FF,#D4D4D4FF role=button text="Filter Gamelist"
          8 Button at=330,660,1260,81 by=fixed-scroll drawn focus heard=Search... in=3 limit=330,417,1260,486 name=GameOption_Search op paints=#707070FF,#D4D4D4FF role=button text=Search...
          9 Button at=330,741,1260,81 by=fixed-scroll drawn focus heard="Game Page" in=3 limit=330,417,1260,486 name=GameOption_GamePage op paints=#707070FF,#D4D4D4FF role=button text="Game Page"
          10 StackPanel at=802,834,316,69 by=fixed-scroll in=3 limit=330,417,1260,486 role=none
            11 Button at=802,834,141,69 by=fixed-scroll drawn focus heard=Apply in=3 limit=330,417,1260,486 name=GameOptionsClose op paints=#707070FF,#D4D4D4FF role=button text=Apply
            12 Button at=951,834,167,69 by=fixed-scroll drawn focus heard=Cancel in=3 limit=330,417,1260,486 name=GameOptionsCancel op paints=#707070FF,#D4D4D4FF role=button text=Cancel
focus 5
edge 5 up same
edge 5 down 6
edge 6 up 5
edge 6 down 7
edge 7 up 6
edge 7 down 8
edge 7 left same
edge 7 right same
edge 8 up 7
edge 8 down 9
edge 8 left same
edge 8 right same
edge 9 up 8
edge 9 down 11
edge 9 left same
edge 9 right same
edge 11 up 9
edge 11 down same
edge 11 left same
edge 11 right 12
edge 12 up 9
edge 12 down same
edge 12 left 11
edge 12 right same
press pageup focus="Dropdown GamelistJumpTo (Jump To...)" top=GamelistOptionsWindow
press pagedown focus="Dropdown GamelistJumpTo (Jump To...)" top=GamelistOptionsWindow
press first focus="Dropdown GamelistJumpTo (Jump To...)" top=GamelistOptionsWindow
press last focus="Dropdown GamelistJumpTo (Jump To...)" top=GamelistOptionsWindow
press back top=none
