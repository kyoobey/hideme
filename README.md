
# hideme
a small cross-platform utility to hide portions of screen

# help text
```console
A small utility to hide portions of your screen
usage: hideme [arguments]
Arguments:
	-h|--help		Print this help message
	-c|--color <r,g,b,a>	The color value of the window
	-v|--loglevel <n>	Changes the log level 0 (lowest) to 3 (highest)
	-b|--background	Flag for setting the background to desktop wallper (use with conjunction with color arguments to get a tinted background)
```

# elaborate these flags and options for me please

## `-c`
specifies the color in rgba format (0 to 1), if the last number is 0 the window should be translucent (working on it :p)
examples:
```
-c 0,0,1,0
```
```
-c 0,0,1,1
```

## `-v`
specifies the log level, 0,1,2,3 for error,warn,info,debug respectively
example:
```
-v 3
```
```
-v 1
```

## `-b`
a flag to render the desktop wallpaper instead of the solid color
