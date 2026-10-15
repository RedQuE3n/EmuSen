# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-MainMenu-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case MainMenu
palette #000000FF #007ACCFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #6E6E6EFF #808080FF #9CDCFEFF #CD5C5CFF #D08770FF #D4D4D4FF #DAA520FF #DCDCDCFF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves this view's own selection, not the focus
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 MenuPanel at=0,0,1920,1200 by=panel heard="Main Menu" limit=0,0,1920,1200 name=PadMenuBig role=menu
      2 HintBar at=535,1114,850,71 by=panel heard="Menu Close Menu, A Select, B Close Menu, D-pad up and down Choose" limit=0,0,1920,1200 role=group
      3 LunaList at=330,538,1260,243 by=panel heard=Menu limit=0,0,1920,1200 name=PadMenuList paints=#FFFFFFFF role=list
        4 ScrollViewer at=330,538,1260,243 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=330,538,1260,243
          5 ListBoxItem at=330,538,1260,81 by=fixed-scroll drawn focus heard="Library  ›" in=4 limit=330,538,1260,243 op paints=#FFFFFFFF role=listitem text="Library  ›"
          6 ListBoxItem at=330,619,1260,81 by=fixed-scroll drawn focus heard="Settings  ›" in=4 limit=330,538,1260,243 op paints=#FFFFFFFF role=listitem text="Settings  ›"
          7 ListBoxItem at=330,700,1260,81 by=fixed-scroll drawn focus heard="EmuSen  ›" in=4 limit=330,538,1260,243 op paints=#FFFFFFFF role=listitem text="EmuSen  ›"
focus none
press pageup menu="open at 0: Library" top=none
press pagedown menu="open at 0: Library" top=none
press first menu="open at 0: Library" top=none
press last menu="open at 0: Library" top=none
press back top=none
