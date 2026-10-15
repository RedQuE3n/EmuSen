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
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=192,255,1536,795 by=panel limit=192,255,1536,794 paints=#9C9CA0FF role=none seen=192,255,1536,794
    1 DockPanel at=228,273,1464,759 by=panel limit=192,255,1536,794 role=none
      2 Grid at=228,273,1464,486 by=panel limit=192,255,1536,794 role=none
        3 StackPanel at=228,273,793.5,486 by=panel limit=192,255,1536,794 role=none
          4 FittedImage at=228,273,793.5,315 by=panel drawn limit=192,255,1536,794 name=ThemeDetailScreenshot role=image
          5 TextBlock at=228,597,793.5,87 box=529,58 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition, screenshot 1" layout=505.73,57.6 limit=192,255,1536,794 name=ThemeDetailCaption paints=#9C9CA0FF role=text size=24 text="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition, screenshot 1"
          6 StackPanel at=228,693,793.5,66 by=panel limit=192,255,1536,794 role=none
            7 Button at=228,693,181.5,66 by=panel drawn focus heard=PREVIOUS limit=192,255,1536,794 name=ThemeDetailPrevious op paints=#55555AFF,#9C9CA0FF role=button text=Previous
              8 TextBlock at=253.5,702,130.5,48 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PREVIOUS layout=86.22,31.2 limit=228,693,181.5,66 owner=7 paints=#EEEEF0FF role=text size=26 text=PREVIOUS
            9 TextBlock at=427.5,705,63,43.5 box=42,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="1 of 2" layout=42,28.8 limit=192,255,1536,794 name=ThemeDetailPosition paints=#9C9CA0FF role=text size=24 text="1 of 2"
            10 Button at=508.5,693,121.5,66 by=panel drawn focus heard=NEXT limit=192,255,1536,794 name=ThemeDetailNext op paints=#55555AFF,#9C9CA0FF role=button text=Next
              11 TextBlock at=534,702,70.5,48 box=47,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NEXT layout=46.25,31.2 limit=508.5,693,121.5,66 owner=10 paints=#9C9CA0FF role=text size=26 text=NEXT
        12 ScrollViewer at=1057.5,273,634.5,450 by=panel fades=b focus limit=192,255,1536,794 name=ThemeDetailFacts overflow=570 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=1057.5,273,634.5,450
          13 StackPanel at=1057.5,273,622.5,1020 by=scroll in=12 limit=1057.5,273,634.5,450 paints=#9C9CA0FF role=none seen=1057.5,273,622.5,450
            14 TextBlock at=1057.5,273,624,216 box=416,144 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="by Alexandra Margaretha van der Westhuizen-Oyelaran, with the Community Artwork Collective   ·   github.com/community-artwork-collective/complete-illustrated-library-anniversary-deluxe-es-de" in=12 layout=412.49,144 limit=1057.5,273,634.5,450 name=ThemeDetailAuthor paints=#9C9CA0FF role=text size=24 text="by Alexandra Margaretha van der Westhuizen-Oyelaran, with the Community Artwork Collective   ·   github.com/community-artwork-collective/complete-illustrated-library-anniversary-deluxe-es-de"
            15 FieldRow at=1057.5,501,622.5,93 by=scroll heard="Last Updated" in=12 limit=1057.5,273,634.5,450 paints=#9C9CA0FF role=group
              16 TextBlock at=1057.5,501,622.5,43.5 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Last Updated" in=12 layout=107.69,28.8 limit=1057.5,501,622.5,93 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Last Updated"
              17 TextBlock at=1057.5,550.5,622.5,43.5 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="2026-09-20, commit 1111111, on branch main" in=12 layout=347.83,28.8 limit=1057.5,501,622.5,93 name=ThemeDetailUpdated paints=#9C9CA0FF role=text size=24 text="2026-09-20, commit 1111111, on branch main"
            18 FieldRow at=1057.5,606,622.5,223.5 by=scroll heard=Here in=12 limit=1057.5,273,634.5,450 paints=#9C9CA0FF role=group seen=1057.5,606,622.5,117
              19 TextBlock at=1057.5,606,622.5,43.5 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Here in=12 layout=37.78,28.8 limit=1057.5,606,622.5,223.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Here
              20 TextBlock name=ThemeDetailState paints=#9C9CA0FF role=text volatile="its words name the run's own folders"
      21 StackPanel at=228,778.5,1464,253.5 by=panel limit=192,255,1536,794 role=none
        22 FieldRow at=228,778.5,1464,135 by=panel heard=Licence limit=192,255,1536,794 name=ThemeDetailLicenceRow paints=#9C9CA0FF role=group
          23 TextBlock at=228,778.5,1464,43.5 box=976,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Licence layout=63.5,28.8 limit=228,778.5,1464,135 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Licence
          24 HintText at=228,828,1464,36 box=976,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="From the theme's own README, else the licence file its host names. The theme is its author's work, under its own terms." layout=808.76,24 limit=228,778.5,1464,135 paints=#86868AFF part role=text size=20 text="From the theme's own README, else the licence file its host names. The theme is its author's work, under its own terms."
          25 StackPanel at=228,870,1464,43.5 by=clip heard=Licence limit=228,778.5,1464,135 role=none
            26 TextBlock at=228,870,1464,43.5 box=976,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Reading the licence line from the theme's repository…" layout=439.63,28.8 limit=228,778.5,1464,135 name=ThemeDetailLicence0 paints=#9C9CA0FF role=text size=24 text="Reading the licence line from the theme's repository…"
        27 WrapPanel at=228,925.5,1464,66 by=panel limit=192,255,1536,794 role=none
          28 Button at=228,925.5,151.5,66 by=panel drawn heard=UPDATE limit=192,255,1536,794 name=ThemeDetailUpdate off paints=#55555AFF,#9C9CA0FF role=button text=Update
            29 TextBlock at=253.5,934.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=66.3,31.2 limit=228,925.5,151.5,66 off owner=28 paints=#55555AFF role=text size=26 text=UPDATE
          30 Button at=391.5,925.5,103.5,66 by=panel drawn focus heard=USE limit=192,255,1536,794 name=ThemeDetailUse op paints=#55555AFF,#9C9CA0FF role=button text=Use
            31 TextBlock at=417,934.5,52.5,48 box=35,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=USE layout=34.16,31.2 limit=391.5,925.5,103.5,66 owner=30 paints=#9C9CA0FF role=text size=26 text=USE
          32 Button at=507,925.5,157.5,66 by=panel drawn focus heard=REMOVE limit=192,255,1536,794 name=ThemeDetailRemove op paints=#55555AFF,#9C9CA0FF role=button text=Remove
            33 TextBlock at=532.5,934.5,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=70.56,31.2 limit=507,925.5,157.5,66 owner=32 paints=#9C9CA0FF role=text size=26 text=REMOVE
          34 Button at=676.5,925.5,138,66 by=panel drawn focus heard=ABOUT limit=192,255,1536,794 name=ThemeDetailAbout op paints=#55555AFF,#9C9CA0FF role=button text=About
            35 TextBlock at=702,934.5,87,48 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ABOUT layout=57.56,31.2 limit=676.5,925.5,138,66 owner=34 paints=#9C9CA0FF role=text size=26 text=ABOUT
          36 Button at=826.5,925.5,135,66 by=panel drawn focus heard=CLOSE limit=192,255,1536,794 name=ThemeDetailClose op paints=#55555AFF,#9C9CA0FF role=button text=Close
            37 TextBlock at=852,934.5,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=826.5,925.5,135,66 owner=36 paints=#9C9CA0FF role=text size=26 text=CLOSE
        38 TextBlock at=228,1003.5,1464,43.5 by=panel drawn limit=192,255,1536,794 name=ThemeDetailStatus paints=#9C9CA0FF role=text
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
