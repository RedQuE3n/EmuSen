# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsMemoryCards-1920x1200
size 1920 1200
kind menu
root SettingsLayout#SettingsScreen
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case SettingsMemoryCards
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#1B1B1EFF role=none
        3 WholeRow at=96,60,1728,66 by=panel heard=Sections limit=0,0,1920,1200 name=SettingsSections role=none
          4 ToggleButton at=96,60,187.5,66 by=panel drawn focus heard=Consoles limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=CONSOLES
            5 AccessText at=120,72,139.5,37.5 box=93,25 by=clip caps=13.999 drawn face=Barlow layout=92.68,24 limit=96,60,187.5,66 owner=4 paints=#D4D4D4FF role=none size=20 text=CONSOLES
          6 ToggleButton at=295.5,60,219,66 by=panel drawn focus heard=PlayStation limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=PLAYSTATION
            7 AccessText at=319.5,72,171,37.5 box=114,25 by=clip caps=13.999 drawn face=Barlow layout=113.56,24 limit=295.5,60,219,66 owner=6 paints=#D4D4D4FF role=none size=20 text=PLAYSTATION
          8 ToggleButton at=526.5,60,253.5,66 by=panel drawn focus heard="Memory cards" limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text="MEMORY CARDS"
            9 AccessText at=550.5,72,205.5,37.5 box=137,25 by=clip caps=13.999 drawn face=Barlow layout=136.02,24 limit=526.5,60,253.5,66 owner=8 paints=#EEEEF0FF role=none size=20 text="MEMORY CARDS"
        10 StackPanel at=96,162,1056,486 by=panel heard="Memory cards" limit=0,0,1920,1200 name=SettingsRows role=none
          11 MenuRow at=96,162,1056,81 by=panel heard="Mode, Shared" limit=0,0,1920,1200 name=SettingsRow.mode:psx.card role=listitem
          12 MenuRow at=96,243,1056,81 by=panel heard="Slot 1, Card 1" limit=0,0,1920,1200 name=SettingsRow.slot:psx.card:0 role=listitem
          13 MenuRow at=96,324,1056,81 by=panel heard="Slot 2, Card 2" limit=0,0,1920,1200 name=SettingsRow.slot:psx.card:1 role=listitem
          14 MenuRow at=96,405,1056,81 by=panel heard="Cards, 0 cards" limit=0,0,1920,1200 name=SettingsRow.cards:psx.card role=listitem
          15 MenuRow at=96,486,1056,81 by=panel heard="Saves on the cards" limit=0,0,1920,1200 name=SettingsRow.saves:psx.card role=listitem
          16 MenuRow at=96,567,1056,81 by=panel heard="New card, …" limit=0,0,1920,1200 name=SettingsRow.new:psx.card role=listitem
        17 Border at=1200,162,624,867 by=panel limit=0,0,1920,1200 name=SettingsPane paints=#26262BFF role=none
          18 StackPanel at=1236,198,552,795 by=clip limit=1200,162,624,867 role=none
            19 TextBlock at=1236,198,552,159 box=368,106 by=clip caps=15.3989 drawn face=Barlow heard="Shared: every PlayStation game sees the same cards when they are shared, as the console did; per game gives each game its own." layout=365.64,105.6 limit=1200,162,624,867 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="Shared: every PlayStation game sees the same cards when they are shared, as the console did; per game gives each game its own."
            20 TextBlock at=1236,369,552,40.5 box=368,27 by=clip caps=15.3989 drawn face=Barlow heard="0 cards in the drawer" layout=195.23,26.4 limit=1200,162,624,867 name=SettingsLine paints=#8D8D91FF role=text size=22 text="0 cards in the drawer"
        21 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=SettingsHelpBar paints=#1B1B1EFF,#707070FF role=none
          22 HintBar at=96,1074,1728,45 by=panel heard="Change, Value, Back, Home" limit=0,0,1920,1200 role=group
focus none
press down focus="SettingsLayout SettingsScreen" settings="consoles Consoles › PlayStation › Memory cards slot:psx.card:0" top=none
press up focus="SettingsLayout SettingsScreen" settings="consoles Consoles › PlayStation › Memory cards mode:psx.card" top=none
press back focus="SettingsLayout SettingsScreen" settings="consoles Consoles › PlayStation cards" top=none
