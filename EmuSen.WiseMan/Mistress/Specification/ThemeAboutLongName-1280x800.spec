# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ThemeAboutLongName-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 220,24.12,840,693.88
title "About The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition"
case ThemeAboutLongName
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=220,169,840,529 by=panel limit=220,169,840,529 paints=#D4D4D4FF role=none
    1 DockPanel at=260,197,760,485 by=panel limit=220,169,840,529 role=none
      2 ButtonBar at=933,638,87,44 by=panel limit=220,169,840,529 paints=#D4D4D4FF role=toolbar
        3 Button at=933,638,87,44 by=clip drawn focus heard=CLOSE limit=933,638,87,44 op paints=#707070FF,#D4D4D4FF role=button text=Close
          4 TextBlock at=950,644,53,32 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=933,638,87,44 owner=3 paints=#EEEEF0FF role=text size=26 text=CLOSE
      5 ScrollViewer at=260,197,760,429 by=panel fades=b focus limit=220,169,840,529 name=AboutScroll overflow=129 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=260,197,760,429
        6 StackPanel at=264,209,752,542 by=scroll in=5 limit=260,197,760,429 paints=#D4D4D4FF role=none seen=264,209,752,417
          7 FieldRow at=264,209,752,62 by=scroll heard=Theme in=5 limit=260,197,760,429 paints=#D4D4D4FF role=group
            8 TextBlock at=264,209,752,29 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Theme in=5 layout=52.61,28.8 limit=264,209,752,62 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Theme
            9 TextBlock at=264,242,752,29 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition" in=5 layout=594.26,28.8 limit=264,209,752,62 name=AboutName paints=#D4D4D4FF role=text size=24 text="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition"
          10 FieldRow at=264,283,752,62 by=scroll heard=Author in=5 limit=260,197,760,429 paints=#D4D4D4FF role=group
            11 TextBlock at=264,283,752,29 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Author in=5 layout=51.46,28.8 limit=264,283,752,62 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Author
            12 TextBlock at=264,316,752,29 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="community-artwork-collective, the owner of its repository on GitHub" in=5 layout=527.26,28.8 limit=264,283,752,62 name=AboutAuthor paints=#D4D4D4FF role=text size=24 text="community-artwork-collective, the owner of its repository on GitHub"
          13 FieldRow at=264,357,752,119 by=scroll heard=Licence in=5 limit=260,197,760,429 paints=#D4D4D4FF role=group
            14 TextBlock at=264,357,752,29 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Licence in=5 layout=59.88,28.8 limit=264,357,752,119 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Licence
            15 HintText at=264,390,752,24 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="From the theme's own README or LICENSE file." in=5 layout=301.16,24 limit=264,357,752,119 paints=#8D8D91FF part role=text size=20 text="From the theme's own README or LICENSE file."
            16 StackPanel at=264,418,752,58 by=clip heard=Licence in=5 limit=264,357,752,119 role=none
              17 TextBlock at=264,418,752,58 box=752,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International, with the fonts under the SIL Open Font License 1.1." in=5 layout=746.78,57.6 limit=264,357,752,119 name=AboutLicence0 paints=#D4D4D4FF role=text size=24 text="Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International, with the fonts under the SIL Open Font License 1.1."
          18 FieldRow at=264,488,752,90 by=scroll heard=Credits in=5 limit=260,197,760,429 paints=#D4D4D4FF role=group
            19 TextBlock at=264,488,752,29 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Credits in=5 layout=54.41,28.8 limit=264,488,752,90 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Credits
            20 HintText at=264,521,752,24 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="From the theme's own README." in=5 layout=202.1,24 limit=264,488,752,90 paints=#8D8D91FF part role=text size=20 text="From the theme's own README."
            21 TextBlock at=264,549,752,29 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="The README names no credits." in=5 layout=238.66,28.8 limit=264,488,752,90 name=AboutCredits paints=#D4D4D4FF role=text size=24 text="The README names no credits."
          22 FieldRow at=264,590,752,91 by=scroll heard=Source in=5 limit=260,197,760,429 paints=#D4D4D4FF role=group seen=264,590,752,36
            23 TextBlock at=264,590,752,29 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Source in=5 layout=53.88,28.8 limit=264,590,752,91 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Source
            24 TextBlock at=264,623,752,58 box=752,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="https://github.com/community-artwork-collective/complete-illustrated-library-anniversary-deluxe-es-de, branch main, commit 1111111aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" in=5 layout=712.25,57.6 limit=264,590,752,91 name=AboutSource paints=#D4D4D4FF role=text seen=264,623,752,3 size=24 text="https://github.com/community-artwork-collective/complete-illustrated-library-anniversary-deluxe-es-de, branch main, commit 1111111aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
focus 3
edge 3 up same
edge 3 down same
edge 3 left same
edge 3 right same
press pageup focus="Button 'Close'" title="About The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition" top=ThemeAboutWindow
press pagedown focus="Button 'Close'" title="About The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition" top=ThemeAboutWindow
press first focus="Button 'Close'" title="About The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition" top=ThemeAboutWindow
press last focus="Button 'Close'" title="About The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition" top=ThemeAboutWindow
press back focus="Button 'Previous'" title="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition" top=ThemeDetailWindow
