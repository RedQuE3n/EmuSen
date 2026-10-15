# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ThemeAboutLongName-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 330,37.18,1260,1040.82
title "About The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition"
case ThemeAboutLongName
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=330,255,1260,795 by=panel limit=330,255,1260,794 paints=#9C9CA0FF role=none seen=330,255,1260,794
    1 DockPanel at=390,297,1140,729 by=panel limit=330,255,1260,794 role=none
      2 ButtonBar at=1395,960,135,66 by=panel limit=330,255,1260,794 paints=#9C9CA0FF role=toolbar
        3 Button at=1395,960,135,66 by=clip drawn focus heard=CLOSE limit=1395,960,135,66 op paints=#55555AFF,#9C9CA0FF role=button text=Close
          4 TextBlock at=1420.5,969,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1395,960,135,66 owner=3 paints=#EEEEF0FF role=text size=26 text=CLOSE
      5 ScrollViewer at=390,297,1140,645 by=panel fades=b focus limit=330,255,1260,794 name=AboutScroll overflow=192 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=390,297,1140,645
        6 StackPanel at=396,315,1128,813 by=scroll in=5 limit=390,297,1140,645 paints=#9C9CA0FF role=none seen=396,315,1128,627
          7 FieldRow at=396,315,1128,93 by=scroll heard=Theme in=5 limit=390,297,1140,645 paints=#9C9CA0FF role=group
            8 TextBlock at=396,315,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Theme in=5 layout=55.42,28.8 limit=396,315,1128,93 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Theme
            9 TextBlock at=396,364.5,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition" in=5 layout=626.16,28.8 limit=396,315,1128,93 name=AboutName paints=#9C9CA0FF role=text size=24 text="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition"
          10 FieldRow at=396,426,1128,93 by=scroll heard=Author in=5 limit=390,297,1140,645 paints=#9C9CA0FF role=group
            11 TextBlock at=396,426,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Author in=5 layout=53.42,28.8 limit=396,426,1128,93 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Author
            12 TextBlock at=396,475.5,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="community-artwork-collective, the owner of its repository on GitHub" in=5 layout=555.1,28.8 limit=396,426,1128,93 name=AboutAuthor paints=#9C9CA0FF role=text size=24 text="community-artwork-collective, the owner of its repository on GitHub"
          13 FieldRow at=396,537,1128,178.5 by=scroll heard=Licence in=5 limit=390,297,1140,645 paints=#9C9CA0FF role=group
            14 TextBlock at=396,537,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Licence in=5 layout=63.5,28.8 limit=396,537,1128,178.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Licence
            15 HintText at=396,586.5,1128,36 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="From the theme's own README or LICENSE file." in=5 layout=314.44,24 limit=396,537,1128,178.5 paints=#86868AFF part role=text size=20 text="From the theme's own README or LICENSE file."
            16 StackPanel at=396,628.5,1128,87 by=clip heard=Licence in=5 limit=396,537,1128,178.5 role=none
              17 TextBlock at=396,628.5,1128,87 box=752,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International, with the fonts under the SIL Open Font License 1.1." in=5 layout=733.54,57.6 limit=396,537,1128,178.5 name=AboutLicence0 paints=#9C9CA0FF role=text size=24 text="Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International, with the fonts under the SIL Open Font License 1.1."
          18 FieldRow at=396,733.5,1128,135 by=scroll heard=Credits in=5 limit=390,297,1140,645 paints=#9C9CA0FF role=group
            19 TextBlock at=396,733.5,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Credits in=5 layout=57.91,28.8 limit=396,733.5,1128,135 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Credits
            20 HintText at=396,783,1128,36 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="From the theme's own README." in=5 layout=210.66,24 limit=396,733.5,1128,135 paints=#86868AFF part role=text size=20 text="From the theme's own README."
            21 TextBlock at=396,825,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="The README names no credits." in=5 layout=250.49,28.8 limit=396,733.5,1128,135 name=AboutCredits paints=#9C9CA0FF role=text size=24 text="The README names no credits."
          22 FieldRow at=396,886.5,1128,136.5 by=scroll heard=Source in=5 limit=390,297,1140,645 paints=#9C9CA0FF role=group seen=396,886.5,1128,55.5
            23 TextBlock at=396,886.5,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Source in=5 layout=56.45,28.8 limit=396,886.5,1128,136.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Source
            24 TextBlock at=396,936,1128,87 box=752,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="https://github.com/community-artwork-collective/complete-illustrated-library-anniversary-deluxe-es-de, branch main, commit 1111111aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" in=5 layout=751.25,57.6 limit=396,886.5,1128,136.5 name=AboutSource paints=#9C9CA0FF role=text seen=396,936,1128,6 size=24 text="https://github.com/community-artwork-collective/complete-illustrated-library-anniversary-deluxe-es-de, branch main, commit 1111111aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
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
