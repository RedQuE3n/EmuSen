# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsMemoryCards-1280x800
size 1280 800
kind menu
root SettingsLayout#SettingsScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case SettingsMemoryCards
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      2 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#1B1B1EFF role=none
        3 WholeRow at=64,40,1152,44 by=panel heard=Sections limit=0,0,1280,800 name=SettingsSections role=tab
          4 SectionTab at=64,40,125,44 by=panel drawn focus heard=Consoles limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=CONSOLES
            5 AccessText at=80,48,93,25 box=93,25 by=clip caps=13.999 drawn face=Barlow layout=92.68,24 limit=64,40,125,44 owner=4 paints=#D4D4D4FF role=none size=20 text=CONSOLES
          6 SectionTab at=197,40,146,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=PLAYSTATION
            7 AccessText at=213,48,114,25 box=114,25 by=clip caps=13.999 drawn face=Barlow layout=113.56,24 limit=197,40,146,44 owner=6 paints=#D4D4D4FF role=none size=20 text=PLAYSTATION
          8 SectionTab at=351,40,169,44 by=panel drawn focus heard="Memory cards" limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text="MEMORY CARDS"
            9 AccessText at=367,48,137,25 box=137,25 by=clip caps=13.999 drawn face=Barlow layout=136.02,24 limit=351,40,169,44 owner=8 paints=#EEEEF0FF role=none size=20 text="MEMORY CARDS"
        10 SelectionStack at=64,108,704,324 by=panel heard="Memory cards" limit=0,0,1280,800 name=SettingsRows role=list
          11 MenuRow at=64,108,704,54 by=panel focus heard="Mode, Shared" limit=0,0,1280,800 name=SettingsRow.mode:psx.card role=listitem
          12 MenuRow at=64,162,704,54 by=panel focus heard="Slot 1, Card 1" limit=0,0,1280,800 name=SettingsRow.slot:psx.card:0 role=listitem
          13 MenuRow at=64,216,704,54 by=panel focus heard="Slot 2, Card 2" limit=0,0,1280,800 name=SettingsRow.slot:psx.card:1 role=listitem
          14 MenuRow at=64,270,704,54 by=panel focus heard="Cards, 0 cards" limit=0,0,1280,800 name=SettingsRow.cards:psx.card role=listitem
          15 MenuRow at=64,324,704,54 by=panel focus heard="Saves on the cards" limit=0,0,1280,800 name=SettingsRow.saves:psx.card role=listitem
          16 MenuRow at=64,378,704,54 by=panel focus heard="New card, …" limit=0,0,1280,800 name=SettingsRow.new:psx.card role=listitem
        17 Border at=800,108,416,578 by=panel limit=0,0,1280,800 name=SettingsPane paints=#26262BFF role=none
          18 StackPanel at=824,132,368,530 by=clip limit=800,108,416,578 role=none
            19 TextBlock at=824,132,368,106 box=368,106 by=clip caps=15.3989 drawn face=Barlow heard="Shared: every PlayStation game sees the same cards when they are shared, as the console did; per game gives each game its own." layout=365.64,105.6 limit=800,108,416,578 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="Shared: every PlayStation game sees the same cards when they are shared, as the console did; per game gives each game its own."
            20 TextBlock at=824,246,368,27 box=368,27 by=clip caps=15.3989 drawn face=Barlow heard="0 cards in the drawer" layout=195.23,26.4 limit=800,108,416,578 name=SettingsLine paints=#8D8D91FF role=text size=22 text="0 cards in the drawer"
        21 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=SettingsHelpBar paints=#1B1B1EFF,#707070FF role=none
          22 HintBar at=64,716,1152,30 by=panel heard="Change, Value, Back, Home" limit=0,0,1280,800 role=group
focus none
press down focus="MenuRow SettingsRow.slot:psx.card:0" settings="consoles Consoles › PlayStation › Memory cards slot:psx.card:0" top=none
press up focus="MenuRow SettingsRow.mode:psx.card" settings="consoles Consoles › PlayStation › Memory cards mode:psx.card" top=none
press back focus="MenuRow SettingsRow.cards" settings="consoles Consoles › PlayStation cards" top=none
