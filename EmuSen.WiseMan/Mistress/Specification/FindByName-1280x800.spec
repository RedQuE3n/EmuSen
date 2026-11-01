# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state FindByName-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 220,168,840,464
title "Find by Name"
case FindByName
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=220,268,840,344 by=panel limit=220,268,840,344 paints=#D4D4D4FF role=none
    1 StackPanel at=260,296,760,300 by=panel limit=220,268,840,344 paints=#D4D4D4FF role=none
      2 StackPanel at=260,296,760,232 by=panel limit=220,268,840,344 role=none
        3 TextBlock at=260,296,760,24 box=760,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Cobalt Harbor (Synthetic)" layout=160.24,24 limit=220,268,840,344 name=FindByNameTitle paints=#D4D4D4FF role=text size=20 text="Cobalt Harbor (Synthetic)"
        4 HintText at=260,326,760,96 box=760,96 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Searches ScreenScraper by name for this game's console, for a file its hashes did not find. Nothing is sent until you press Search; each search is one request, and one that finds nothing also counts against today's allowance of unrecognised games. Choosing a game keeps its text for this file and fetches its pictures, up to 4 more requests; the file itself is not renamed." layout=754.1,96 limit=220,268,840,344 name=FindByNameHint paints=#8D8D91FF role=text size=20 text="Searches ScreenScraper by name for this game's console, for a file its hashes did not find. Nothing is sent until you press Search; each search is one request, and one that finds nothing also counts against today's allowance of unrecognised games. Choosing a game keeps its text for this file and fetches its pictures, up to 4 more requests; the file itself is not renamed."
        5 Grid at=260,428,760,46 by=panel limit=220,268,840,344 role=none
          6 TextBox at=260,428,653,46 by=panel drawn focus heard="The game's name" limit=220,268,840,344 name=FindByNameText op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit text="Cobalt Harbor"
            7 ScrollViewer allow="a text box scrolls its own text" at=271,435,631,32 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
          8 Button at=921,429,99,44 by=panel drawn heard=SEARCH limit=220,268,840,344 name=FindByNameSearch off paints=#707070FF,#D4D4D4FF role=button text=Search
            9 TextBlock at=938,435,65,32 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SEARCH layout=64.9,31.2 limit=921,429,99,44 off owner=8 paints=#707070FF role=text size=26 text=SEARCH
        10 HintText at=260,480,760,48 box=760,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="ScreenScraper cannot be asked: this build carries no developer credentials and EmuSen's developer file is not on this computer." layout=733.42,48 limit=220,268,840,344 name=FindByNameStatus paints=#8D8D91FF role=text size=20 text="ScreenScraper cannot be asked: this build carries no developer credentials and EmuSen's developer file is not on this computer."
      11 ButtonBar at=921,552,99,44 by=panel limit=220,268,840,344 paints=#D4D4D4FF role=toolbar
        12 Button at=921,552,99,44 by=clip drawn focus heard=CANCEL limit=921,552,99,44 name=FindByNameCancel op paints=#707070FF,#D4D4D4FF role=button text=Cancel
          13 TextBlock at=938,558,65,32 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=64.74,31.2 limit=921,552,99,44 owner=12 paints=#D4D4D4FF role=text size=26 text=CANCEL
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
