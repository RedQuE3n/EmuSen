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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=128,169,1024,529 by=panel limit=128,169,1024,529 paints=#D4D4D4FF role=none
    1 DockPanel at=152,181,976,505 by=panel limit=128,169,1024,529 role=none
      2 Grid at=152,181,976,324 by=panel limit=128,169,1024,529 role=none
        3 StackPanel at=152,181,529,324 by=panel limit=128,169,1024,529 role=none
          4 FittedImage at=152,181,529,210 by=panel drawn limit=128,169,1024,529 name=ThemeDetailScreenshot role=image
          5 TextBlock at=152,397,529,58 box=529,58 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition, screenshot 1" layout=480.58,57.6 limit=128,169,1024,529 name=ThemeDetailCaption paints=#D4D4D4FF role=text size=24 text="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition, screenshot 1"
          6 StackPanel at=152,461,529,44 by=panel limit=128,169,1024,529 role=none
            7 Button at=152,461,116,44 by=panel drawn focus heard=PREVIOUS limit=128,169,1024,529 name=ThemeDetailPrevious op paints=#707070FF,#D4D4D4FF role=button text=Previous
              8 TextBlock at=169,467,82,32 box=82,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PREVIOUS layout=81.04,31.2 limit=152,461,116,44 owner=7 paints=#EEEEF0FF role=text size=26 text=PREVIOUS
            9 TextBlock at=280,469,40,29 box=40,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="1 of 2" layout=39.91,28.8 limit=128,169,1024,529 name=ThemeDetailPosition paints=#D4D4D4FF role=text size=24 text="1 of 2"
            10 Button at=332,461,77,44 by=panel drawn focus heard=NEXT limit=128,169,1024,529 name=ThemeDetailNext op paints=#707070FF,#D4D4D4FF role=button text=Next
              11 TextBlock at=349,467,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NEXT layout=42.61,31.2 limit=332,461,77,44 owner=10 paints=#D4D4D4FF role=text size=26 text=NEXT
        12 ScrollViewer at=705,181,423,300 by=panel fades=b focus limit=128,169,1024,529 name=ThemeDetailFacts overflow=323 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=705,181,423,300
          13 StackPanel at=705,181,415,623 by=scroll in=12 limit=705,181,423,300 paints=#D4D4D4FF role=none seen=705,181,415,300
            14 TextBlock at=705,181,416,116 box=416,116 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="by Alexandra Margaretha van der Westhuizen-Oyelaran, with the Community Artwork Collective   ·   github.com/community-artwork-collective/complete-illustrated-library-anniversary-deluxe-es-de" in=12 layout=415.08,115.2 limit=705,181,423,300 name=ThemeDetailAuthor paints=#D4D4D4FF role=text size=24 text="by Alexandra Margaretha van der Westhuizen-Oyelaran, with the Community Artwork Collective   ·   github.com/community-artwork-collective/complete-illustrated-library-anniversary-deluxe-es-de"
            15 FieldRow at=705,305,415,62 by=scroll heard="Last Updated" in=12 limit=705,181,423,300 paints=#D4D4D4FF role=group
              16 TextBlock at=705,305,415,29 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Last Updated" in=12 layout=103.54,28.8 limit=705,305,415,62 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Last Updated"
              17 TextBlock at=705,338,415,29 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="2026-09-20, commit 1111111, on branch main" in=12 layout=337.44,28.8 limit=705,305,415,62 name=ThemeDetailUpdated paints=#D4D4D4FF role=text size=24 text="2026-09-20, commit 1111111, on branch main"
            18 FieldRow at=705,375,415,120 by=scroll heard=Here in=12 limit=705,181,423,300 paints=#D4D4D4FF role=group seen=705,375,415,106
              19 TextBlock at=705,375,415,29 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Here in=12 layout=35.9,28.8 limit=705,375,415,120 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Here
              20 TextBlock name=ThemeDetailState paints=#D4D4D4FF role=text volatile="its words name the run's own folders"
      21 StackPanel at=152,517,976,169 by=panel limit=128,169,1024,529 role=none
        22 FieldRow at=152,517,976,90 by=panel heard=Licence limit=128,169,1024,529 name=ThemeDetailLicenceRow paints=#D4D4D4FF role=group
          23 TextBlock at=152,517,976,29 box=976,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Licence layout=59.88,28.8 limit=152,517,976,90 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Licence
          24 HintText at=152,550,976,24 box=976,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="From the theme's own README, else the licence file its host names. The theme is its author's work, under its own terms." layout=772.36,24 limit=152,517,976,90 paints=#8D8D91FF part role=text size=20 text="From the theme's own README, else the licence file its host names. The theme is its author's work, under its own terms."
          25 StackPanel at=152,578,976,29 by=clip heard=Licence limit=152,517,976,90 role=none
            26 TextBlock at=152,578,976,29 box=976,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Reading the licence line from the theme's repository…" layout=419.38,28.8 limit=152,517,976,90 name=ThemeDetailLicence0 paints=#D4D4D4FF role=text size=24 text="Reading the licence line from the theme's repository…"
        27 WrapPanel at=152,615,976,44 by=panel limit=128,169,1024,529 role=none
          28 Button at=152,615,98,44 by=panel drawn heard=UPDATE limit=128,169,1024,529 name=ThemeDetailUpdate off paints=#707070FF,#D4D4D4FF role=button text=Update
            29 TextBlock at=169,621,64,32 box=64,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=152,615,98,44 off owner=28 paints=#707070FF role=text size=26 text=UPDATE
          30 Button at=258,615,67,44 by=panel drawn focus heard=USE limit=128,169,1024,529 name=ThemeDetailUse op paints=#707070FF,#D4D4D4FF role=button text=Use
            31 TextBlock at=275,621,33,32 box=33,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=USE layout=32.32,31.2 limit=258,615,67,44 owner=30 paints=#D4D4D4FF role=text size=26 text=USE
          32 Button at=333,615,101,44 by=panel drawn focus heard=REMOVE limit=128,169,1024,529 name=ThemeDetailRemove op paints=#707070FF,#D4D4D4FF role=button text=Remove
            33 TextBlock at=350,621,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=333,615,101,44 owner=32 paints=#D4D4D4FF role=text size=26 text=REMOVE
          34 Button at=442,615,90,44 by=panel drawn focus heard=ABOUT limit=128,169,1024,529 name=ThemeDetailAbout op paints=#707070FF,#D4D4D4FF role=button text=About
            35 TextBlock at=459,621,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ABOUT layout=55.12,31.2 limit=442,615,90,44 owner=34 paints=#D4D4D4FF role=text size=26 text=ABOUT
          36 Button at=540,615,87,44 by=panel drawn focus heard=CLOSE limit=128,169,1024,529 name=ThemeDetailClose op paints=#707070FF,#D4D4D4FF role=button text=Close
            37 TextBlock at=557,621,53,32 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=540,615,87,44 owner=36 paints=#D4D4D4FF role=text size=26 text=CLOSE
        38 TextBlock at=152,667,976,29 by=panel drawn limit=128,169,1024,529 name=ThemeDetailStatus paints=#D4D4D4FF role=text
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
