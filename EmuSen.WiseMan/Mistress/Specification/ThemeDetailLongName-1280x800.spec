# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ThemeDetailLongName-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 128,24.12,1024,693.88
title "The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition"
case ThemeDetailLongName
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=128,169,1024,529 by=panel limit=128,169,1024,529 paints=#9C9CA0FF role=none
    1 DockPanel at=152,181,976,505 by=panel limit=128,169,1024,529 role=none
      2 Grid at=152,181,976,324 by=panel limit=128,169,1024,529 role=none
        3 StackPanel at=152,181,529,324 by=panel limit=128,169,1024,529 role=none
          4 FittedImage at=152,181,529,210 by=panel drawn limit=128,169,1024,529 name=ThemeDetailScreenshot role=image
          5 TextBlock at=152,397,529,58 box=529,58 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition, screenshot 1" layout=505.73,57.6 limit=128,169,1024,529 name=ThemeDetailCaption paints=#9C9CA0FF role=text size=24 text="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition, screenshot 1"
          6 StackPanel at=152,461,529,44 by=panel limit=128,169,1024,529 role=none
            7 Button at=152,461,121,44 by=panel drawn focus heard=PREVIOUS limit=128,169,1024,529 name=ThemeDetailPrevious op paints=#55555AFF,#9C9CA0FF role=button text=Previous
              8 TextBlock at=169,467,87,32 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PREVIOUS layout=86.22,31.2 limit=152,461,121,44 owner=7 paints=#EEEEF0FF role=text size=26 text=PREVIOUS
            9 TextBlock at=285,469,42,29 box=42,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="1 of 2" layout=42,28.8 limit=128,169,1024,529 name=ThemeDetailPosition paints=#9C9CA0FF role=text size=24 text="1 of 2"
            10 Button at=339,461,81,44 by=panel drawn focus heard=NEXT limit=128,169,1024,529 name=ThemeDetailNext op paints=#55555AFF,#9C9CA0FF role=button text=Next
              11 TextBlock at=356,467,47,32 box=47,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NEXT layout=46.25,31.2 limit=339,461,81,44 owner=10 paints=#9C9CA0FF role=text size=26 text=NEXT
        12 ScrollViewer at=705,181,423,300 by=panel fades=b focus limit=128,169,1024,529 name=ThemeDetailFacts overflow=380 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=705,181,423,300
          13 StackPanel at=705,181,415,680 by=scroll in=12 limit=705,181,423,300 paints=#9C9CA0FF role=none seen=705,181,415,300
            14 TextBlock at=705,181,416,144 box=416,144 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="by Alexandra Margaretha van der Westhuizen-Oyelaran, with the Community Artwork Collective   ·   github.com/community-artwork-collective/complete-illustrated-library-anniversary-deluxe-es-de" in=12 layout=412.49,144 limit=705,181,423,300 name=ThemeDetailAuthor paints=#9C9CA0FF role=text size=24 text="by Alexandra Margaretha van der Westhuizen-Oyelaran, with the Community Artwork Collective   ·   github.com/community-artwork-collective/complete-illustrated-library-anniversary-deluxe-es-de"
            15 FieldRow at=705,333,415,62 by=scroll heard="Last Updated" in=12 limit=705,181,423,300 paints=#9C9CA0FF role=group
              16 TextBlock at=705,333,415,29 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Last Updated" in=12 layout=107.69,28.8 limit=705,333,415,62 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Last Updated"
              17 TextBlock at=705,366,415,29 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="2026-09-20, commit 1111111, on branch main" in=12 layout=347.83,28.8 limit=705,333,415,62 name=ThemeDetailUpdated paints=#9C9CA0FF role=text size=24 text="2026-09-20, commit 1111111, on branch main"
            18 FieldRow at=705,403,415,149 by=scroll heard=Here in=12 limit=705,181,423,300 paints=#9C9CA0FF role=group seen=705,403,415,78
              19 TextBlock at=705,403,415,29 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Here in=12 layout=37.78,28.8 limit=705,403,415,149 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Here
              20 TextBlock name=ThemeDetailState paints=#9C9CA0FF role=text volatile="its words name the run's own folders"
      21 StackPanel at=152,517,976,169 by=panel limit=128,169,1024,529 role=none
        22 FieldRow at=152,517,976,90 by=panel heard=Licence limit=128,169,1024,529 name=ThemeDetailLicenceRow paints=#9C9CA0FF role=group
          23 TextBlock at=152,517,976,29 box=976,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Licence layout=63.5,28.8 limit=152,517,976,90 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Licence
          24 HintText at=152,550,976,24 box=976,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="From the theme's own README, else the licence file its host names. The theme is its author's work, under its own terms." layout=808.76,24 limit=152,517,976,90 paints=#86868AFF part role=text size=20 text="From the theme's own README, else the licence file its host names. The theme is its author's work, under its own terms."
          25 StackPanel at=152,578,976,29 by=clip heard=Licence limit=152,517,976,90 role=none
            26 TextBlock at=152,578,976,29 box=976,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Reading the licence line from the theme's repository…" layout=439.63,28.8 limit=152,517,976,90 name=ThemeDetailLicence0 paints=#9C9CA0FF role=text size=24 text="Reading the licence line from the theme's repository…"
        27 WrapPanel at=152,615,976,44 by=panel limit=128,169,1024,529 role=none
          28 Button at=152,615,101,44 by=panel drawn heard=UPDATE limit=128,169,1024,529 name=ThemeDetailUpdate off paints=#55555AFF,#9C9CA0FF role=button text=Update
            29 TextBlock at=169,621,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=66.3,31.2 limit=152,615,101,44 off owner=28 paints=#55555AFF role=text size=26 text=UPDATE
          30 Button at=261,615,69,44 by=panel drawn focus heard=USE limit=128,169,1024,529 name=ThemeDetailUse op paints=#55555AFF,#9C9CA0FF role=button text=Use
            31 TextBlock at=278,621,35,32 box=35,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=USE layout=34.16,31.2 limit=261,615,69,44 owner=30 paints=#9C9CA0FF role=text size=26 text=USE
          32 Button at=338,615,105,44 by=panel drawn focus heard=REMOVE limit=128,169,1024,529 name=ThemeDetailRemove op paints=#55555AFF,#9C9CA0FF role=button text=Remove
            33 TextBlock at=355,621,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=70.56,31.2 limit=338,615,105,44 owner=32 paints=#9C9CA0FF role=text size=26 text=REMOVE
          34 Button at=451,615,92,44 by=panel drawn focus heard=ABOUT limit=128,169,1024,529 name=ThemeDetailAbout op paints=#55555AFF,#9C9CA0FF role=button text=About
            35 TextBlock at=468,621,58,32 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ABOUT layout=57.56,31.2 limit=451,615,92,44 owner=34 paints=#9C9CA0FF role=text size=26 text=ABOUT
          36 Button at=551,615,90,44 by=panel drawn focus heard=CLOSE limit=128,169,1024,529 name=ThemeDetailClose op paints=#55555AFF,#9C9CA0FF role=button text=Close
            37 TextBlock at=568,621,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=551,615,90,44 owner=36 paints=#9C9CA0FF role=text size=26 text=CLOSE
        38 TextBlock at=152,667,976,29 by=panel drawn limit=128,169,1024,529 name=ThemeDetailStatus paints=#9C9CA0FF role=text
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
edge 30 left same
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
