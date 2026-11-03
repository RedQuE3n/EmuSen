# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsBindings-1920x1200
size 1920 1200
kind menu
root SettingsLayout#SettingsScreen
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case SettingsBindings
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#1B1B1EFF role=none
        3 WholeRow at=96,60,1728,66 by=panel heard=Sections limit=0,0,1920,1200 name=SettingsSections role=tab
          4 SectionTab at=96,60,238.5,66 by=panel drawn focus heard=Controllers limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=CONTROLLERS
            5 AccessText at=120,72,190.5,37.5 box=127,25 by=clip caps=13.999 drawn face=Barlow layout=126.94,24 limit=96,60,238.5,66 owner=4 paints=#D4D4D4FF role=none size=20 text=CONTROLLERS
          6 SectionTab at=346.5,60,352.5,66 by=panel drawn focus heard="Controller bindings" limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text="CONTROLLER BINDINGS"
            7 AccessText at=370.5,72,304.5,37.5 box=203,25 by=clip caps=13.999 drawn face=Barlow layout=202.66,24 limit=346.5,60,352.5,66 owner=6 paints=#EEEEF0FF role=none size=20 text="CONTROLLER BINDINGS"
        8 Border at=96,162,1056,867 by=panel heard="Controller bindings" limit=0,0,1920,1200 name=SettingsContent paints=#26262BFF role=none
          9 StackPanel at=132,198,984,795 by=clip limit=96,162,1056,867 role=none
            10 WholeRow at=132,198,984,66 by=clip limit=96,162,1056,867 name=SettingsContentTabs paints=#D4D4D4FF role=tab
              11 SectionTab at=132,198,165,66 by=clip drawn focus heard="Player 1" limit=96,162,1056,867 op paints=#D4D4D4FF role=tabitem text="PLAYER 1"
                12 AccessText at=156,210,117,37.5 box=78,25 by=clip caps=13.999 drawn face=Barlow layout=77.74,24 limit=132,198,165,66 owner=11 paints=#D4D4D4FF role=none size=20 text="PLAYER 1"
              13 SectionTab at=309,198,279,66 by=clip drawn focus heard="Super Nintendo" limit=96,162,1056,867 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text="SUPER NINTENDO"
                14 AccessText at=333,210,231,37.5 box=154,25 by=clip caps=13.999 drawn face=Barlow layout=153.46,24 limit=309,198,279,66 owner=13 paints=#EEEEF0FF role=none size=20 text="SUPER NINTENDO"
              15 SectionTab at=600,198,159,66 by=clip drawn focus heard="Sega CD" limit=96,162,1056,867 op paints=#D4D4D4FF role=tabitem text="SEGA CD"
                16 AccessText at=624,210,111,37.5 box=74,25 by=clip caps=13.999 drawn face=Barlow layout=73.12,24 limit=600,198,159,66 owner=15 paints=#D4D4D4FF role=none size=20 text="SEGA CD"
              17 SectionTab at=771,198,219,66 by=clip drawn focus heard=PlayStation limit=96,162,1056,867 op paints=#D4D4D4FF role=tabitem text=PLAYSTATION
                18 AccessText at=795,210,171,37.5 box=114,25 by=clip caps=13.999 drawn face=Barlow layout=113.56,24 limit=771,198,219,66 owner=17 paints=#D4D4D4FF role=none size=20 text=PLAYSTATION
            19 Border at=132,264,984,570 by=clip limit=96,162,1056,867 name=SettingsContentBody role=none
              20 ControllerDiagram at=132,264,984,570 by=clip heard="SNES controller" limit=132,264,984,570 name=BindingsDiagram role=group
            21 TextBlock at=132,852,984,79.5 box=656,53 by=clip caps=15.3989 drawn face=Barlow heard="Left and Right move between the drawing's regions; A rebinds the focused one." layout=603.64,52.8 limit=96,162,1056,867 name=SettingsContentCaption paints=#8D8D91FF role=text size=22 text="Left and Right move between the drawing's regions; A rebinds the focused one."
        22 Border at=1200,162,624,867 by=panel limit=0,0,1920,1200 name=SettingsPane paints=#26262BFF role=none
          23 StackPanel at=1236,198,552,795 by=clip limit=1200,162,624,867 role=none
            24 TextBlock at=1236,198,552,58.5 box=368,39 by=clip caps=22.3984 drawn face=Barlow heard=Y layout=17.63,38.4 limit=1200,162,624,867 name=SettingsLine paints=#D4D4D4FF role=text size=32 text=Y
            25 TextBlock at=1236,268.5,552,40.5 box=368,27 by=clip caps=15.3989 drawn face=Barlow heard="Super Nintendo · Y" layout=170.28,26.4 limit=1200,162,624,867 name=SettingsLine paints=#8D8D91FF role=text size=22 text="Super Nintendo · Y"
            26 TextBlock at=1236,321,552,40.5 box=368,27 by=clip caps=15.3989 drawn face=Barlow heard="Bound to the pad's West button." layout=294.58,26.4 limit=1200,162,624,867 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="Bound to the pad's West button."
            27 TextBlock at=1236,373.5,552,120 box=368,80 by=clip caps=15.3989 drawn face=Barlow heard="A listens for a press and rebinds it; Y resets this button; the profile saves at once." layout=345.38,79.2 limit=1200,162,624,867 name=SettingsLine paints=#8D8D91FF role=text size=22 text="A listens for a press and rebinds it; Y resets this button; the profile saves at once."
        28 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=SettingsHelpBar paints=#1B1B1EFF,#707070FF role=none
          29 HintBar at=96,1074,1728,45 by=panel heard="Rebind, Region, Reset, Console, Player, Test, Back" limit=0,0,1920,1200 role=group
focus none
press down focus="SettingsLayout SettingsScreen" settings="controllers Controllers › Controller bindings none" top=none
press up focus="SettingsLayout SettingsScreen" settings="controllers Controllers › Controller bindings none" top=none
press back focus="MenuRow SettingsRow.bindings" settings="controllers  bindings" top=none
