# `ti`, the tiny terminal renderer

`ti` is a sprite-based 2d graphic renderer, using Unicode's [braille codepoints][braille] and [ANSI escapes][ansi escapes] to emulate
a responsive pixel screen in your terminal. `ti` has a purposefully simple interface, reminiscent of the [behavior of old consoles][hardware sprites].

With `ti`, you can draw sprites or individual pixels to the screen using various blitting modes and a simple 256-color palette. For a full set of features as well as examples, see the [documentation][documentation].

[braille]: https://en.wikipedia.org/wiki/Braille_Patterns
[ansi escapes]: https://en.wikipedia.org/wiki/ANSI_escape_code
[hardware sprites]: https://en.wikipedia.org/wiki/Sprite_(computer_graphics)#Systems_with_hardware_sprites

## Font support

Your terminal's font may not support braille characters, or you may not have fixed-width fonts that support it, or your font may not distinguish between
lit and unlit pixels sufficiently. In that case, this repository also includes the `MonoBrailleCC0.otf` font. The font is a variant of
[Braille CC0][braille cc0], licensed under [Creative Commons Zero v1.0 Universal][cc0]. In terminals that support fallback fonts (such as
[Visual Studio Code][vscode]), this can be used to ensure that `ti` renders cleanly.

[braille cc0]: https://www.fontspace.com/braille-cc0-font-f96789
[cc0]: https://creativecommons.org/share-your-work/public-domain/cc0/
[vscode]: https://code.visualstudio.com/

## Next steps

- [ ] Convert true colors to palette colors
- [ ] Operations to read sprites from image files in more advanced ways
- [ ] A simple rendering loop
- [ ] Read input
- [ ] When drawing a standalone frame, ensure there's enough space
- [ ] Better braille font, and a guide on it maybe
- [ ] Examples in docstrings

[documentation]: https://example.com
