# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ThemeDetail-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 128,82.5,1024,635
title "Plain Shelf"
case ThemeDetail
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=128,182,1024,515 by=panel limit=128,182,1024,515 paints=#D4D4D4FF role=none
    1 DockPanel at=152,194,976,491 by=panel limit=128,182,1024,515 role=none
      2 Grid at=152,194,976,300 by=panel limit=128,182,1024,515 role=none
        3 StackPanel at=152,194,529,300 by=panel limit=128,182,1024,515 role=none
          4 FittedImage at=152,194,529,210 by=panel drawn limit=128,182,1024,515 name=ThemeDetailScreenshot role=image
          5 TextBlock at=152,410,529,29 box=529,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Plain Shelf, screenshot 1" layout=185.81,28.8 limit=128,182,1024,515 name=ThemeDetailCaption paints=#D4D4D4FF role=text size=24 text="Plain Shelf, screenshot 1"
          6 StackPanel at=152,445,529,44 by=panel limit=128,182,1024,515 role=none
            7 Button at=152,445,116,44 by=panel drawn focus heard=PREVIOUS limit=128,182,1024,515 name=ThemeDetailPrevious op paints=#707070FF,#D4D4D4FF role=button text=Previous
              8 TextBlock at=169,451,82,32 box=82,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PREVIOUS layout=81.04,31.2 limit=152,445,116,44 owner=7 paints=#EEEEF0FF role=text size=26 text=PREVIOUS
            9 TextBlock at=280,453,40,29 box=40,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="1 of 2" layout=39.91,28.8 limit=128,182,1024,515 name=ThemeDetailPosition paints=#D4D4D4FF role=text size=24 text="1 of 2"
            10 Button at=332,445,77,44 by=panel drawn focus heard=NEXT limit=128,182,1024,515 name=ThemeDetailNext op paints=#707070FF,#D4D4D4FF role=button text=Next
              11 TextBlock at=349,451,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NEXT layout=42.61,31.2 limit=332,445,77,44 owner=10 paints=#D4D4D4FF role=text size=26 text=NEXT
        12 ScrollViewer at=705,194,423,300 by=panel fades=b focus limit=128,182,1024,515 name=ThemeDetailFacts overflow=108 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=705,194,423,300
          13 StackPanel at=705,194,415,408 by=scroll in=12 limit=705,194,423,300 paints=#D4D4D4FF role=none seen=705,194,415,300
            14 TextBlock at=705,194,415,29 box=415,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="by Bo Sample   ·   github.com/bo/plain-shelf-es-de" in=12 layout=393.96,28.8 limit=705,194,423,300 name=ThemeDetailAuthor paints=#D4D4D4FF role=text size=24 text="by Bo Sample   ·   github.com/bo/plain-shelf-es-de"
            15 FieldRow at=705,231,415,62 by=scroll heard="Last Updated" in=12 limit=705,194,423,300 paints=#D4D4D4FF role=group
              16 TextBlock at=705,231,415,29 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Last Updated" in=12 layout=103.54,28.8 limit=705,231,415,62 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Last Updated"
              17 TextBlock at=705,264,415,29 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="2026-09-20, commit 1111111, on branch master" in=12 layout=353.69,28.8 limit=705,231,415,62 name=ThemeDetailUpdated paints=#D4D4D4FF role=text size=24 text="2026-09-20, commit 1111111, on branch master"
            18 FieldRow at=705,301,415,91 by=scroll heard=Here in=12 limit=705,194,423,300 paints=#D4D4D4FF role=group
              19 TextBlock at=705,301,415,29 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Here in=12 layout=35.9,28.8 limit=705,301,415,91 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Here
              20 TextBlock name=ThemeDetailState paints=#D4D4D4FF role=text volatile="its words name the run's own folders"
            21 FieldRow at=705,400,415,62 by=scroll heard="Variants (2)" in=12 limit=705,194,423,300 paints=#D4D4D4FF role=group
              22 TextBlock at=705,400,415,29 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Variants (2)" in=12 layout=87.26,28.8 limit=705,400,415,62 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Variants (2)"
              23 TextBlock at=705,433,415,29 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="List, Grid" in=12 layout=67.22,28.8 limit=705,400,415,62 name=ThemeDetailVariants paints=#D4D4D4FF role=text size=24 text="List, Grid"
            24 FieldRow at=705,470,415,62 by=scroll heard="Colour Schemes (0)" in=12 limit=705,194,423,300 paints=#D4D4D4FF role=group seen=705,470,415,24
              25 TextBlock at=705,470,415,29 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Colour Schemes (0)" in=12 layout=150.17,28.8 limit=705,470,415,62 name=PART_Label paints=#D4D4D4FF part role=text seen=705,470,415,24 size=24 text="Colour Schemes (0)"
      26 StackPanel at=152,506,976,179 by=panel limit=128,182,1024,515 role=none
        27 FieldRow at=152,506,976,90 by=panel heard=Licence limit=128,182,1024,515 name=ThemeDetailLicenceRow paints=#D4D4D4FF role=group
          28 TextBlock at=152,506,976,29 box=976,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Licence layout=59.88,28.8 limit=152,506,976,90 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Licence
          29 HintText at=152,539,976,24 box=976,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="From the theme's own README, else the licence file its host names. The theme is its author's work, under its own terms." layout=772.36,24 limit=152,506,976,90 paints=#8D8D91FF part role=text size=20 text="From the theme's own README, else the licence file its host names. The theme is its author's work, under its own terms."
          30 StackPanel at=152,567,976,29 by=clip heard=Licence limit=152,506,976,90 role=none
            31 TextBlock at=152,567,976,29 box=976,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Reading the licence line from the theme's repository…" layout=419.38,28.8 limit=152,506,976,90 name=ThemeDetailLicence0 paints=#D4D4D4FF role=text size=24 text="Reading the licence line from the theme's repository…"
        32 WrapPanel at=152,604,976,44 by=panel limit=128,182,1024,515 role=none
          33 Button at=152,604,98,44 by=panel drawn heard=UPDATE limit=128,182,1024,515 name=ThemeDetailUpdate off paints=#707070FF,#D4D4D4FF role=button text=Update
            34 TextBlock at=169,610,64,32 box=64,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=152,604,98,44 off owner=33 paints=#707070FF role=text size=26 text=UPDATE
          35 Button at=258,604,67,44 by=panel drawn focus heard=USE limit=128,182,1024,515 name=ThemeDetailUse op paints=#707070FF,#D4D4D4FF role=button text=Use
            36 TextBlock at=275,610,33,32 box=33,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=USE layout=32.32,31.2 limit=258,604,67,44 owner=35 paints=#D4D4D4FF role=text size=26 text=USE
          37 Button at=333,604,101,44 by=panel drawn focus heard=REMOVE limit=128,182,1024,515 name=ThemeDetailRemove op paints=#707070FF,#D4D4D4FF role=button text=Remove
            38 TextBlock at=350,610,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=333,604,101,44 owner=37 paints=#D4D4D4FF role=text size=26 text=REMOVE
          39 Button at=442,604,90,44 by=panel drawn focus heard=ABOUT limit=128,182,1024,515 name=ThemeDetailAbout op paints=#707070FF,#D4D4D4FF role=button text=About
            40 TextBlock at=459,610,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ABOUT layout=55.12,31.2 limit=442,604,90,44 owner=39 paints=#D4D4D4FF role=text size=26 text=ABOUT
          41 Button at=540,604,87,44 by=panel drawn focus heard=CLOSE limit=128,182,1024,515 name=ThemeDetailClose op paints=#707070FF,#D4D4D4FF role=button text=Close
            42 TextBlock at=557,610,53,32 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=540,604,87,44 owner=41 paints=#D4D4D4FF role=text size=26 text=CLOSE
        43 TextBlock at=152,656,976,29 by=panel drawn limit=128,182,1024,515 name=ThemeDetailStatus paints=#D4D4D4FF role=text
focus 7
edge 7 up same
edge 7 down 37
edge 7 left same
edge 7 right 10
edge 10 up same
edge 10 down 37
edge 10 left 7
edge 10 right 39
edge 35 up 10
edge 35 down same
edge 35 left same
edge 35 right 37
edge 37 up 10
edge 37 down same
edge 37 left 35
edge 37 right 39
edge 39 up 10
edge 39 down same
edge 39 left 37
edge 39 right 41
edge 41 up 10
edge 41 down same
edge 41 left 39
edge 41 right same
press pageup focus="Button 'Previous'" title="Plain Shelf" top=ThemeDetailWindow
press pagedown focus="Button 'Previous'" title="Plain Shelf" top=ThemeDetailWindow
press first focus="Button 'Previous'" title="Plain Shelf" top=ThemeDetailWindow
press last focus="Button 'Previous'" title="Plain Shelf" top=ThemeDetailWindow
press back focus="Button 'Refresh'" title="ES-DE Themes" top=ThemeBrowserWindow
