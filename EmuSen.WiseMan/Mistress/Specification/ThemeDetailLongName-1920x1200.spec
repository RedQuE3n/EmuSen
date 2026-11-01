# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ThemeDetailLongName-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 192,37.18,1536,1040.82
title "The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition"
case ThemeDetailLongName
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=192,255,1536,795 by=panel limit=192,255,1536,794 paints=#D4D4D4FF role=none seen=192,255,1536,794
    1 DockPanel at=228,273,1464,759 by=panel limit=192,255,1536,794 role=none
      2 Grid at=228,273,1464,486 by=panel limit=192,255,1536,794 role=none
        3 StackPanel at=228,273,793.5,486 by=panel limit=192,255,1536,794 role=none
          4 FittedImage at=228,273,793.5,315 by=panel drawn limit=192,255,1536,794 name=ThemeDetailScreenshot role=image
          5 TextBlock at=228,597,793.5,87 box=529,58 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition, screenshot 1" layout=480.58,57.6 limit=192,255,1536,794 name=ThemeDetailCaption paints=#D4D4D4FF role=text size=24 text="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition, screenshot 1"
          6 StackPanel at=228,693,793.5,66 by=panel limit=192,255,1536,794 role=none
            7 Button at=228,693,174,66 by=panel drawn focus heard=PREVIOUS limit=192,255,1536,794 name=ThemeDetailPrevious op paints=#707070FF,#D4D4D4FF role=button text=Previous
              8 TextBlock at=253.5,702,123,48 box=82,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PREVIOUS layout=81.04,31.2 limit=228,693,174,66 owner=7 paints=#EEEEF0FF role=text size=26 text=PREVIOUS
            9 TextBlock at=420,705,60,43.5 box=40,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="1 of 2" layout=39.91,28.8 limit=192,255,1536,794 name=ThemeDetailPosition paints=#D4D4D4FF role=text size=24 text="1 of 2"
            10 Button at=498,693,115.5,66 by=panel drawn focus heard=NEXT limit=192,255,1536,794 name=ThemeDetailNext op paints=#707070FF,#D4D4D4FF role=button text=Next
              11 TextBlock at=523.5,702,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NEXT layout=42.61,31.2 limit=498,693,115.5,66 owner=10 paints=#D4D4D4FF role=text size=26 text=NEXT
        12 ScrollViewer at=1057.5,273,634.5,450 by=panel fades=b focus limit=192,255,1536,794 name=ThemeDetailFacts overflow=484.5 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=1057.5,273,634.5,450
          13 StackPanel at=1057.5,273,622.5,934.5 by=scroll in=12 limit=1057.5,273,634.5,450 paints=#D4D4D4FF role=none seen=1057.5,273,622.5,450
            14 TextBlock at=1057.5,273,624,174 box=416,116 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="by Alexandra Margaretha van der Westhuizen-Oyelaran, with the Community Artwork Collective   ·   github.com/community-artwork-collective/complete-illustrated-library-anniversary-deluxe-es-de" in=12 layout=415.08,115.2 limit=1057.5,273,634.5,450 name=ThemeDetailAuthor paints=#D4D4D4FF role=text size=24 text="by Alexandra Margaretha van der Westhuizen-Oyelaran, with the Community Artwork Collective   ·   github.com/community-artwork-collective/complete-illustrated-library-anniversary-deluxe-es-de"
            15 FieldRow at=1057.5,459,622.5,93 by=scroll heard="Last Updated" in=12 limit=1057.5,273,634.5,450 paints=#D4D4D4FF role=group
              16 TextBlock at=1057.5,459,622.5,43.5 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Last Updated" in=12 layout=103.54,28.8 limit=1057.5,459,622.5,93 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Last Updated"
              17 TextBlock at=1057.5,508.5,622.5,43.5 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="2026-09-20, commit 1111111, on branch main" in=12 layout=337.44,28.8 limit=1057.5,459,622.5,93 name=ThemeDetailUpdated paints=#D4D4D4FF role=text size=24 text="2026-09-20, commit 1111111, on branch main"
            18 FieldRow at=1057.5,564,622.5,180 by=scroll heard=Here in=12 limit=1057.5,273,634.5,450 paints=#D4D4D4FF role=group seen=1057.5,564,622.5,159
              19 TextBlock at=1057.5,564,622.5,43.5 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Here in=12 layout=35.9,28.8 limit=1057.5,564,622.5,180 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Here
              20 TextBlock name=ThemeDetailState paints=#D4D4D4FF role=text volatile="its words name the run's own folders"
      21 StackPanel at=228,778.5,1464,253.5 by=panel limit=192,255,1536,794 role=none
        22 FieldRow at=228,778.5,1464,135 by=panel heard=Licence limit=192,255,1536,794 name=ThemeDetailLicenceRow paints=#D4D4D4FF role=group
          23 TextBlock at=228,778.5,1464,43.5 box=976,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Licence layout=59.88,28.8 limit=228,778.5,1464,135 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Licence
          24 HintText at=228,828,1464,36 box=976,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="From the theme's own README, else the licence file its host names. The theme is its author's work, under its own terms." layout=772.36,24 limit=228,778.5,1464,135 paints=#8D8D91FF part role=text size=20 text="From the theme's own README, else the licence file its host names. The theme is its author's work, under its own terms."
          25 StackPanel at=228,870,1464,43.5 by=clip heard=Licence limit=228,778.5,1464,135 role=none
            26 TextBlock at=228,870,1464,43.5 box=976,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Reading the licence line from the theme's repository…" layout=419.38,28.8 limit=228,778.5,1464,135 name=ThemeDetailLicence0 paints=#D4D4D4FF role=text size=24 text="Reading the licence line from the theme's repository…"
        27 WrapPanel at=228,925.5,1464,66 by=panel limit=192,255,1536,794 role=none
          28 Button at=228,925.5,147,66 by=panel drawn heard=UPDATE limit=192,255,1536,794 name=ThemeDetailUpdate off paints=#707070FF,#D4D4D4FF role=button text=Update
            29 TextBlock at=253.5,934.5,96,48 box=64,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=228,925.5,147,66 off owner=28 paints=#707070FF role=text size=26 text=UPDATE
          30 Button at=387,925.5,100.5,66 by=panel drawn focus heard=USE limit=192,255,1536,794 name=ThemeDetailUse op paints=#707070FF,#D4D4D4FF role=button text=Use
            31 TextBlock at=412.5,934.5,49.5,48 box=33,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=USE layout=32.32,31.2 limit=387,925.5,100.5,66 owner=30 paints=#D4D4D4FF role=text size=26 text=USE
          32 Button at=499.5,925.5,151.5,66 by=panel drawn focus heard=REMOVE limit=192,255,1536,794 name=ThemeDetailRemove op paints=#707070FF,#D4D4D4FF role=button text=Remove
            33 TextBlock at=525,934.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=499.5,925.5,151.5,66 owner=32 paints=#D4D4D4FF role=text size=26 text=REMOVE
          34 Button at=663,925.5,135,66 by=panel drawn focus heard=ABOUT limit=192,255,1536,794 name=ThemeDetailAbout op paints=#707070FF,#D4D4D4FF role=button text=About
            35 TextBlock at=688.5,934.5,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ABOUT layout=55.12,31.2 limit=663,925.5,135,66 owner=34 paints=#D4D4D4FF role=text size=26 text=ABOUT
          36 Button at=810,925.5,130.5,66 by=panel drawn focus heard=CLOSE limit=192,255,1536,794 name=ThemeDetailClose op paints=#707070FF,#D4D4D4FF role=button text=Close
            37 TextBlock at=835.5,934.5,79.5,48 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=810,925.5,130.5,66 owner=36 paints=#D4D4D4FF role=text size=26 text=CLOSE
        38 TextBlock at=228,1003.5,1464,43.5 by=panel drawn limit=192,255,1536,794 name=ThemeDetailStatus paints=#D4D4D4FF role=text
focus 7
edge 7 up same
edge 7 down 32
edge 7 left same
edge 7 right 10
edge 10 up same
edge 10 down 32
edge 10 left 7
edge 10 right 34
edge 30 up 10
edge 30 down same
edge 30 left 7
edge 30 right 32
edge 32 up 10
edge 32 down same
edge 32 left 30
edge 32 right 34
edge 34 up 10
edge 34 down same
edge 34 left 32
edge 34 right 36
edge 36 up 10
edge 36 down same
edge 36 left 34
edge 36 right same
press pageup focus="Button 'Previous'" title="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition" top=ThemeDetailWindow
press pagedown focus="Button 'Previous'" title="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition" top=ThemeDetailWindow
press first focus="Button 'Previous'" title="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition" top=ThemeDetailWindow
press last focus="Button 'Previous'" title="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition" top=ThemeDetailWindow
press back focus="Button 'Refresh'" title="ES-DE Themes" top=ThemeBrowserWindow
