# gfx.lib

## Dateiformat
- Binary
- Little Endian

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

