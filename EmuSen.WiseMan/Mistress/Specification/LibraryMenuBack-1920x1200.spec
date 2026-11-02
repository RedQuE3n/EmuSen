# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state LibraryMenuBack-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1161
case LibraryMenuBack
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1920,1161 by=panel limit=0,0,1920,1161 role=none
    1 MenuPanel at=0,0,1920,1161 by=panel heard="Main Menu" limit=0,0,1920,1161 name=PadMenuBig role=menu
      2 HintBar at=550,1075,821,71 by=panel heard="Menu Close Menu, A Select, B Close Menu, D-pad up and down Choose" limit=0,0,1920,1161 role=group
      3 LunaList at=350,398,1220,486 by=panel heard=Menu limit=0,0,1920,1161 name=PadMenuList paints=#FFFFFFFF role=list
        4 ScrollViewer at=350,398,1220,486 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=350,398,1220,486
          5 ListBoxItem at=350,398,1220,81 by=fixed-scroll drawn focus heard="Back to Cobalt Harbor (Synthetic)" in=4 limit=350,398,1220,486 op paints=#FFFFFFFF role=listitem text="Back to Cobalt Harbor (Synthetic)"
          6 ListBoxItem at=350,479,1220,81 by=fixed-scroll drawn focus heard="Game Options..." in=4 limit=350,398,1220,486 op paints=#FFFFFFFF role=listitem text="Game Options..."
          7 ListBoxItem at=350,560,1220,81 by=fixed-scroll drawn focus heard="Scrape This Game..." in=4 limit=350,398,1220,486 op paints=#FFFFFFFF role=listitem text="Scrape This Game..."
          8 ListBoxItem at=350,641,1220,81 by=fixed-scroll drawn focus heard=Library in=4 limit=350,398,1220,486 op paints=#FFFFFFFF role=listitem text="Library  ›"
          9 ListBoxItem at=350,722,1220,81 by=fixed-scroll drawn focus heard=Settings in=4 limit=350,398,1220,486 op paints=#FFFFFFFF role=listitem text="Settings  ›"
          10 ListBoxItem at=350,803,1220,81 by=fixed-scroll drawn focus heard=EmuSen in=4 limit=350,398,1220,486 op paints=#FFFFFFFF role=listitem text="EmuSen  ›"
      11 PauseCardPane at=1640,422,210,232 by=panel limit=0,0,1920,1161 name=PauseCardPane role=none
        12 StackPanel at=1640,422,210,232 by=panel limit=0,0,1920,1161 role=none
          13 Image at=1640,422,210,183.75 by=panel drawn limit=0,0,1920,1161 name=PanePicture role=image
          14 TextBlock at=1640,618,210,36 box=210,36 by=panel caps=13.999 drawn face=Barlow heard=Paused layout=93.99,36 limit=0,0,1920,1161 name=PaneCaption paints=#8D8D91FF role=text size=20 text=Paused
focus none
press down menu="open at 1: Game Options..." top=none
press up menu="open at 0: Back to Cobalt Harbor (Synthetic)" top=none
press back top=none
