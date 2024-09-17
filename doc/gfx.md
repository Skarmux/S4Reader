# gfx.lib

## About

Settlers 4 uses a virtual filesystem, present in .lib files. Stored is a library header, containing
information about the amounts of stored data and then a list of file information headers with offset
and other metadata. The file contents are stored after all the informational segments inside the lib
file.

## Format
- Binary
- Little Endian

The `gfx.lib` file at install root contains embedded graphical assets that are not compressed.

|Lenght|Description|
|--|--|
|?|Header|
|4|Length of ?|
|4|?|
|4|Paths Segment Length (bytes)|
|4|Number of Paths|
|4|Files Segment Length (bytes)|
|4|Number of Files|
|`Paths Segment Lenght`|Stream of `\0` terminated c-strings representing file paths.|
|`File Segment Length`||
|...|...|
|4|Header Size|

## Graphics (.gfx)

Plain image data.

### 8 bit (256 Colors)
### 16 bit (65536 Colors)
### palette

#### Header
|Offset|Description|
|--|--|
|0|Image Type|
|4|Data Offset|
|8|Width|
|12|Height|
|16|Chunk Height|
|20|Flag 1|
|24|Flag 2|
|28|Row Count|

### Header
|Offset|Description|
|--|--|
|0|imageType|
|4|dataOffset|
|8|headType|
|12|imgType|
|16|width|
|20|height|
|24|Anchor point X|
|28|Anghor point Y|
|32|flag1|
|36|flag2|

### Gfx/24.gfx | Trojan Siedler

|Offset|Description|
|28|Platzhalter|
|14752|Freier Siedler (Ost) Frame 1|
|25552|Freier Siedler (Ost) Frame 13|
|...|...|

## Frames (.gil)

Indexing `.gfx` files.

List of animation frame offsets within a .gfx file.

## Direction (.dil)

Indexing `.gil` files.

Many assets have multiple orientations, especially when moving on the map.

### Gfx/24.dil | Trojan Siedler

|||

## Job (.jil)

Indexing `.dil` files.

Job related in context of buildings. Working in/at the building or construction of the building itself.

|Gfx/14.jil|Trojan Buildings|
|--|--|
|0|Placeholder|
|1|Holzfäller|
|2|Förster|
|3|Sägewerk|
|4|Steinmetz|
|...|...|

|Gfx/24.jil|Trojan Settlers|
|--|--|
|0|Placeholder|
|1|Freier Siedler|
|...|...|

# Index Lists

Index files are nothing more than a stream of uncompressed 32 bit indices.
The file name has to match a corresponding graphics file where the indices point into.

## .jil

- For animations and overlapping textures.
- jil may stand for: "job index list" file or "jump index list"
- it indicates the different jobs in a .dil file
  jil (job)    --> .dil (direction)--> gil (frames) --> gfx
- May contain build stages and animation of buildings
- Items, Orientation and Frames

## .pil

- pil may stand for: "palette index list" file"
- it is a list of file indexes used to read a .pa5 or .pa6 file

## .gil

- gil may stand for: "graphics index list" file
- it contains the offsets of animation frames in a .gfx file

## .dil

- dil may stand for: "direction index list" file
- it indicates the different object directions in a gil file
- jil (job)    --> .dil (direction)--> gil (frames) --> gfx

# Palette

## .pa6

- Stands for palette collection

# Texture Atlas

## .ghX .glX

- Texture Atlas for ground
- h (huge?) is twice the resolution of l (large?)

## Assets within `gfx.lib` in order of appearance

|Path|Description|
|--|--|
|Gfx/0.gil||
|Gfx/0.pa5||
|Gfx/0.pa6||
|Gfx/0.pil||
|Gfx/10.dil||
|Gfx/10.gil||
|Gfx/10.jil||
|Gfx/10.p24||
|Gfx/10.p25||
|Gfx/10.p26||
|Gfx/10.p44||
|Gfx/10.p45||
|Gfx/10.p46||
|Gfx/10.pi2||
|Gfx/10.pi4||
|Gfx/10.sil||
|Gfx/11.dil||
|Gfx/11.gil||
|Gfx/11.jil||
|Gfx/11.p24||
|Gfx/11.p25||
|Gfx/11.p26||
|Gfx/11.p44||
|Gfx/11.p45||
|Gfx/11.p46||
|Gfx/11.pi2||
|Gfx/11.pi4||
|Gfx/11.sil||
|Gfx/12.dil||
|Gfx/12.gil||
|Gfx/12.jil||
|Gfx/12.p24||
|Gfx/12.p25||
|Gfx/12.p26||
|Gfx/12.p44||
|Gfx/12.p45||
|Gfx/12.p46||
|Gfx/12.pi2||
|Gfx/12.pi4||
|Gfx/12.sil||
|Gfx/13.dil||
|Gfx/13.gil||
|Gfx/13.jil||
|Gfx/13.p24||
|Gfx/13.p25||
|Gfx/13.p26||
|Gfx/13.p44||
|Gfx/13.p45||
|Gfx/13.p46||
|Gfx/13.pi2||
|Gfx/13.pi4||
|Gfx/13.sil||
|Gfx/19.gil||
|Gfx/19.pa5||
|Gfx/19.pa6||
|Gfx/19.pil||
|Gfx/20.dil||
|Gfx/20.gil||
|Gfx/20.jil||
|Gfx/20.p24||
|Gfx/20.p25||
|Gfx/20.p26||
|Gfx/20.p44||
|Gfx/20.p45||
|Gfx/20.p46||
|Gfx/20.pi2||
|Gfx/20.pi4||
|Gfx/20.sil||
|Gfx/21.dil||
|Gfx/21.gil||
|Gfx/21.jil||
|Gfx/21.p24||
|Gfx/21.p25||
|Gfx/21.p26||
|Gfx/21.p44||
|Gfx/21.p45||
|Gfx/21.p46||
|Gfx/21.pi2||
|Gfx/21.pi4||
|Gfx/21.sil||
|Gfx/22.dil||
|Gfx/22.gil||
|Gfx/22.jil||
|Gfx/22.p24||
|Gfx/22.p25||
|Gfx/22.p26||
|Gfx/22.p44||
|Gfx/22.p45||
|Gfx/22.p46||
|Gfx/22.pi2||
|Gfx/22.pi4||
|Gfx/22.sil||
|Gfx/23.dil||
|Gfx/23.gil||
|Gfx/23.jil||
|Gfx/23.p24||
|Gfx/23.p25||
|Gfx/23.p26||
|Gfx/23.p44||
|Gfx/23.p45||
|Gfx/23.p46||
|Gfx/23.pi2||
|Gfx/23.pi4||
|Gfx/23.sil||
|Gfx/28.gil||
|Gfx/28.pa5||
|Gfx/28.pa6||
|Gfx/28.pil||
|Gfx/29.gil||
|Gfx/29.pa5||
|Gfx/29.pa6||
|Gfx/29.pil||
|Gfx/3.dil||
|Gfx/3.gil||
|Gfx/3.jil||
|Gfx/3.p24||
|Gfx/3.p25||
|Gfx/3.p26||
|Gfx/3.p44||
|Gfx/3.p45||
|Gfx/3.p46||
|Gfx/3.pi2||
|Gfx/3.pi4||
|Gfx/3.sil||
|Gfx/30.dil||
|Gfx/30.gil||
|Gfx/30.jil||
|Gfx/30.p24||
|Gfx/30.p25||
|Gfx/30.p26||
|Gfx/30.p44||
|Gfx/30.p45||
|Gfx/30.p46||
|Gfx/30.pi2||
|Gfx/30.pi4||
|Gfx/30.sil||
|Gfx/31.dil||
|Gfx/31.gil||
|Gfx/31.jil||
|Gfx/31.p24||
|Gfx/31.p25||
|Gfx/31.p26||
|Gfx/31.p44||
|Gfx/31.p45||
|Gfx/31.p46||
|Gfx/31.pi2||
|Gfx/31.pi4||
|Gfx/31.sil||
|Gfx/32.dil||
|Gfx/32.gil||
|Gfx/32.jil||
|Gfx/32.p24||
|Gfx/32.p25||
|Gfx/32.p26||
|Gfx/32.p44||
|Gfx/32.p45||
|Gfx/32.p46||
|Gfx/32.pi2||
|Gfx/32.pi4||
|Gfx/32.sil||
|Gfx/33.jil||
|Gfx/33.pi2||
|Gfx/33.pi4||
|Gfx/33.sil||
|Gfx/34.dil||
|Gfx/34.gil||
|Gfx/34.jil||
|Gfx/34.p24||
|Gfx/34.p25||
|Gfx/34.p26||
|Gfx/34.p44||
|Gfx/34.p45||
|Gfx/34.p46||
|Gfx/34.pi2||
|Gfx/34.pi4||
|Gfx/34.sil||
|Gfx/36.dil||
|Gfx/36.gil||
|Gfx/36.jil||
|Gfx/36.p24||
|Gfx/36.p25||
|Gfx/36.p26||
|Gfx/36.p44||
|Gfx/36.p45||
|Gfx/36.p46||
|Gfx/36.pi2||
|Gfx/36.pi4||
|Gfx/36.sil||
|Gfx/37.dil||
|Gfx/37.gil||
|Gfx/37.jil||
|Gfx/37.p24||
|Gfx/37.p25||
|Gfx/37.p26||
|Gfx/37.p44||
|Gfx/37.p45||
|Gfx/37.p46||
|Gfx/37.pi2||
|Gfx/37.pi4||
|Gfx/37.sil||
|Gfx/39.gil||
|Gfx/39.pa5||
|Gfx/39.pa6||
|Gfx/39.pil||
|Gfx/4.dil||
|Gfx/4.gil||
|Gfx/4.jil||
|Gfx/4.p24||
|Gfx/4.p25||
|Gfx/4.p26||
|Gfx/4.p44||
|Gfx/4.p45||
|Gfx/4.p46||
|Gfx/4.pi2||
|Gfx/4.pi4||
|Gfx/4.sil||
|Gfx/40.gil||
|Gfx/40.pa5||
|Gfx/40.pa6||
|Gfx/40.pil||
|Gfx/5.dil||
|Gfx/5.gil||
|Gfx/5.jil||
|Gfx/5.p24||
|Gfx/5.p25||
|Gfx/5.p26||
|Gfx/5.p44||
|Gfx/5.p45||
|Gfx/5.p46||
|Gfx/5.pi2||
|Gfx/5.pi4||
|Gfx/5.sil||
|Gfx/6.dil||
|Gfx/6.gil||

