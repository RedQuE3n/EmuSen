# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsShaderParameters-1920x1200
size 1920 1200
kind menu
root SettingsLayout#SettingsScreen
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case SettingsShaderParameters
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#1B1B1EFF role=none
        3 WholeRow at=96,60,1728,66 by=panel heard=Sections limit=0,0,1920,1200 name=SettingsSections role=none
          4 ToggleButton at=96,60,156,66 by=panel drawn focus heard=Display limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=DISPLAY
            5 AccessText at=120,72,108,37.5 box=72,25 by=clip caps=13.999 drawn face=Barlow layout=71.9,24 limit=96,60,156,66 owner=4 paints=#D4D4D4FF role=none size=20 text=DISPLAY
          6 ToggleButton at=264,60,172.5,66 by=panel drawn focus heard=Shaders limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=SHADERS
            7 AccessText at=288,72,124.5,37.5 box=83,25 by=clip caps=13.999 drawn face=Barlow layout=82.2,24 limit=264,60,172.5,66 owner=6 paints=#D4D4D4FF role=none size=20 text=SHADERS
          8 ToggleButton at=448.5,60,279,66 by=panel drawn focus heard="Super Nintendo" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="SUPER NINTENDO"
            9 AccessText at=472.5,72,231,37.5 box=154,25 by=clip caps=13.999 drawn face=Barlow layout=153.46,24 limit=448.5,60,279,66 owner=8 paints=#D4D4D4FF role=none size=20 text="SUPER NINTENDO"
          10 ToggleButton at=739.5,60,223.5,66 by=panel drawn focus heard=Parameters limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text=PARAMETERS
            11 AccessText at=763.5,72,175.5,37.5 box=117,25 by=clip caps=13.999 drawn face=Barlow layout=116.52,24 limit=739.5,60,223.5,66 owner=10 paints=#EEEEF0FF role=none size=20 text=PARAMETERS
        12 StackPanel at=96,162,1056,810 by=panel heard=Parameters limit=0,0,1920,1200 name=SettingsRows role=none
          13 MenuRow at=96,162,1056,81 by=panel heard="Search parameters, …" limit=0,0,1920,1200 name=SettingsRow.paramsearch role=listitem
          14 MenuRow at=96,243,1056,81 by=panel heard="Reset all" limit=0,0,1920,1200 name=SettingsRow.paramreset role=listitem
          15 MenuRow at=96,324,1056,81 by=panel heard="Quality, Balanced" limit=0,0,1920,1200 name=SettingsRow.p:quality role=listitem
          16 MenuRow at=96,405,1056,81 by=panel heard="Signal, Composite" limit=0,0,1920,1200 name=SettingsRow.p:signal role=listitem
          17 MenuRow at=96,486,1056,81 by=panel heard="Television standard, The game's" limit=0,0,1920,1200 name=SettingsRow.p:standard role=listitem
          18 MenuRow at=96,567,1056,81 by=panel heard="PAL colour decoding, Delay line" limit=0,0,1920,1200 name=SettingsRow.p:palDecoder role=listitem
          19 MenuRow at=96,648,1056,81 by=panel heard="Screen, Consumer TV, 20-inch" limit=0,0,1920,1200 name=SettingsRow.p:screen role=listitem
          20 MenuRow at=96,729,1056,81 by=panel heard="Colour, North America" limit=0,0,1920,1200 name=SettingsRow.p:colour role=listitem
          21 MenuRow at=96,810,1056,81 by=panel heard="Curvature, 1" limit=0,0,1920,1200 name=SettingsRow.p:curvature role=listitem
          22 MenuRow at=96,891,1056,81 by=panel heard="Picture brightness, Bright" limit=0,0,1920,1200 name=SettingsRow.p:level role=listitem
        23 Border at=1200,162,624,867 by=panel limit=0,0,1920,1200 name=SettingsPane paints=#26262BFF role=none
          24 StackPanel at=1236,198,552,795 by=clip limit=1200,162,624,867 role=none
            25 TextBlock at=1236,198,552,40.5 box=368,27 by=clip caps=15.3989 drawn face=Barlow heard="Words in a parameter's name." layout=277,26.4 limit=1200,162,624,867 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="Words in a parameter's name."
        26 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=SettingsHelpBar paints=#1B1B1EFF,#707070FF role=none
          27 HintBar at=96,1074,1728,45 by=panel heard="Change, Value, Back, Home" limit=0,0,1920,1200 role=group
focus none
press down focus="SettingsLayout SettingsScreen" settings="display Display › Shaders › Super Nintendo › Parameters paramreset" top=none
press up focus="SettingsLayout SettingsScreen" settings="display Display › Shaders › Super Nintendo › Parameters paramsearch" top=none
press back focus="SettingsLayout SettingsScreen" settings="display Display › Shaders › Super Nintendo params" top=none
