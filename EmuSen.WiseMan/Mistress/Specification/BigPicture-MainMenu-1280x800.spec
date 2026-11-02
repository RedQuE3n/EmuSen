# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-MainMenu-1280x800
size 1280 800
kind menu
root Border#PadMenuPanel
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case MainMenu
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves this view's own selection, not the focus
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 MenuPanel at=0,0,1280,800 by=panel heard="Main Menu" limit=0,0,1280,800 name=PadMenuBig role=menu
      2 HintBar at=366,742,547,48 by=panel heard="Menu Close Menu, A Select, B Close Menu, D-pad up and down Choose" limit=0,0,1280,800 role=group
      3 LunaList at=220,359,840,162 by=panel heard=Menu limit=0,0,1280,800 name=PadMenuList paints=#FFFFFFFF role=list
        4 ScrollViewer at=220,359,840,162 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=220,359,840,162
          5 ListBoxItem at=220,359,840,54 by=fixed-scroll drawn focus heard=Library in=4 limit=220,359,840,162 op paints=#FFFFFFFF role=listitem text="Library  ›"
          6 ListBoxItem at=220,413,840,54 by=fixed-scroll drawn focus heard=Settings in=4 limit=220,359,840,162 op paints=#FFFFFFFF role=listitem text="Settings  ›"
          7 ListBoxItem at=220,467,840,54 by=fixed-scroll drawn focus heard=EmuSen in=4 limit=220,359,840,162 op paints=#FFFFFFFF role=listitem text="EmuSen  ›"
focus none
press pageup menu="open at 0: Library" top=none
press pagedown menu="open at 0: Library" top=none
press first menu="open at 0: Library" top=none
press last menu="open at 0: Library" top=none
press back top=none
