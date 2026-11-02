# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomeCardTools-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case HomeCardTools
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 AdaptiveLayout at=0,0,1920,1200 by=panel heard=Home limit=0,0,1920,1200 name=PadCard role=pane
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
        3 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1920,1200 by=panel heard=Home limit=0,0,1920,1200 name=AdaptiveCard role=menu
            5 HintBar at=96,1104,763,45 by=panel heard="A Choose, B Back, Y Favourite, Menu Close" limit=0,0,1920,1200 role=group
            6 Border at=96,234,636,567 by=panel limit=0,0,1920,1200 name=AdaptiveCardRows role=none
              7 Border at=96,234,636,567 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                8 LunaList at=96,234,636,567 by=panel heard=Menu limit=0,0,1920,1200 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=96,234,636,567 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,234,636,567
                    10 ListBoxItem at=96,234,636,81 by=fixed-scroll drawn focus heard=Continue in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Continue
                    11 ListBoxItem at=96,315,636,81 by=fixed-scroll drawn focus heard="Selected game" in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Cobalt Harbor (Synthetic)"
                    12 ListBoxItem at=96,396,636,81 by=fixed-scroll drawn focus heard=Search in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Search
                    13 ListBoxItem at=96,477,636,81 by=fixed-scroll drawn focus heard="Start Console" in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Start Console  ›"
                    14 ListBoxItem at=96,558,636,81 by=fixed-scroll drawn focus heard=Tools in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Tools  ›"
                    15 ListBoxItem at=96,639,636,81 by=fixed-scroll drawn focus heard=Settings in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Settings  ›"
                    16 ListBoxItem at=96,720,636,81 by=fixed-scroll drawn focus heard=Power in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Power  ›"
            17 Border at=864,234,960,795 by=panel limit=0,0,1920,1200 name=AdaptiveCardPane paints=#26262BFF role=none
              18 Border at=900,270,888,723 by=panel limit=0,0,1920,1200 name=AdaptiveDetail role=none
                19 PauseCardPane at=900,270,888,370 by=panel limit=0,0,1920,1200 name=PauseCardPane role=none
                  20 StackPanel at=900,270,888,370 by=panel limit=0,0,1920,1200 name=PaneLines role=none
                    21 StackPanel at=900,270,888,79 by=panel limit=0,0,1920,1200 role=none
                      22 TextBlock at=900,270,888,40 box=888,40 by=panel caps=15.3989 drawn face=Barlow heard=Scraper layout=109.89,39.6 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=22 text=Scraper
                      23 TextBlock at=900,313,888,36 box=888,36 by=panel caps=13.999 drawn face=Barlow heard="41 games missing media" layout=305.16,36 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="41 games missing media"
                    24 StackPanel at=900,367,888,79 by=panel limit=0,0,1920,1200 role=none
                      25 TextBlock at=900,367,888,40 box=888,40 by=panel caps=15.3989 drawn face=Barlow heard=Packages layout=134.64,39.6 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=22 text=Packages
                      26 TextBlock at=900,410,888,36 box=888,36 by=panel caps=13.999 drawn face=Barlow heard="Makes, checks and zips game packages" layout=497.85,36 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="Makes, checks and zips game packages"
                    27 StackPanel at=900,464,888,79 by=panel limit=0,0,1920,1200 role=none
                      28 TextBlock at=900,464,888,40 box=888,40 by=panel caps=15.3989 drawn face=Barlow heard="Library Manager" layout=226.81,39.6 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=22 text="Library Manager"
                      29 TextBlock at=900,507,888,36 box=888,36 by=panel caps=13.999 drawn face=Barlow heard="Finds duplicate and damaged dumps" layout=467.1,36 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="Finds duplicate and damaged dumps"
                    30 StackPanel at=900,561,888,79 by=panel limit=0,0,1920,1200 role=none
                      31 TextBlock at=900,561,888,40 box=888,40 by=panel caps=15.3989 drawn face=Barlow heard=Cores layout=79.73,39.6 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=22 text=Cores
                      32 TextBlock at=900,604,888,36 box=888,36 by=panel caps=13.999 drawn face=Barlow heard="1 core waiting for approval" layout=330.06,36 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="1 core waiting for approval"
focus none
press down menu="open at 5: Settings" top=none
press up menu="open at 4: Tools" top=none
press back top=none
