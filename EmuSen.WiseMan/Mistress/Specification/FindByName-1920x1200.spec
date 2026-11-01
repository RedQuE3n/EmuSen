# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state FindByName-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 330,252,1260,696
title "Find by Name"
case FindByName
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=330,402,1260,516 by=panel limit=330,402,1260,516 paints=#D4D4D4FF role=none
    1 StackPanel at=390,444,1140,450 by=panel limit=330,402,1260,516 paints=#D4D4D4FF role=none
      2 StackPanel at=390,444,1140,348 by=panel limit=330,402,1260,516 role=none
        3 TextBlock at=390,444,1140,36 box=760,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Cobalt Harbor (Synthetic)" layout=160.24,24 limit=330,402,1260,516 name=FindByNameTitle paints=#D4D4D4FF role=text size=20 text="Cobalt Harbor (Synthetic)"
        4 HintText at=390,489,1140,144 box=760,96 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Searches ScreenScraper by name for this game's console, for a file its hashes did not find. Nothing is sent until you press Search; each search is one request, and one that finds nothing also counts against today's allowance of unrecognised games. Choosing a game keeps its text for this file and fetches its pictures, up to 4 more requests; the file itself is not renamed." layout=754.1,96 limit=330,402,1260,516 name=FindByNameHint paints=#8D8D91FF role=text size=20 text="Searches ScreenScraper by name for this game's console, for a file its hashes did not find. Nothing is sent until you press Search; each search is one request, and one that finds nothing also counts against today's allowance of unrecognised games. Choosing a game keeps its text for this file and fetches its pictures, up to 4 more requests; the file itself is not renamed."
        5 Grid at=390,642,1140,69 by=panel limit=330,402,1260,516 role=none
          6 TextBox at=390,642,979.5,69 by=panel drawn focus heard="The game's name" limit=330,402,1260,516 name=FindByNameText op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit text="Cobalt Harbor"
            7 ScrollViewer allow="a text box scrolls its own text" at=406.5,652.5,946.5,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
          8 Button at=1381.5,643.5,148.5,66 by=panel drawn heard=SEARCH limit=330,402,1260,516 name=FindByNameSearch off paints=#707070FF,#D4D4D4FF role=button text=Search
            9 TextBlock at=1407,652.5,97.5,48 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SEARCH layout=64.9,31.2 limit=1381.5,643.5,148.5,66 off owner=8 paints=#707070FF role=text size=26 text=SEARCH
        10 HintText at=390,720,1140,72 box=760,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="ScreenScraper cannot be asked: this build carries no developer credentials and EmuSen's developer file is not on this computer." layout=733.42,48 limit=330,402,1260,516 name=FindByNameStatus paints=#8D8D91FF role=text size=20 text="ScreenScraper cannot be asked: this build carries no developer credentials and EmuSen's developer file is not on this computer."
      11 ButtonBar at=1381.5,828,148.5,66 by=panel limit=330,402,1260,516 paints=#D4D4D4FF role=toolbar
        12 Button at=1381.5,828,148.5,66 by=clip drawn focus heard=CANCEL limit=1381.5,828,148.5,66 name=FindByNameCancel op paints=#707070FF,#D4D4D4FF role=button text=Cancel
          13 TextBlock at=1407,837,97.5,48 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=64.74,31.2 limit=1381.5,828,148.5,66 owner=12 paints=#D4D4D4FF role=text size=26 text=CANCEL
focus 6
edge 6 up same
edge 6 down 12
edge 6 left same
edge 6 right 12
edge 12 up 6
edge 12 down same
edge 12 left 6
edge 12 right same
press pageup focus="TextBox FindByNameText (The game's name)" title="Find by Name" top=FindByNameWindow
press pagedown focus="TextBox FindByNameText (The game's name)" title="Find by Name" top=FindByNameWindow
press first focus="TextBox FindByNameText (The game's name)" title="Find by Name" top=FindByNameWindow
press last focus="TextBox FindByNameText (The game's name)" title="Find by Name" top=FindByNameWindow
press back top=none
