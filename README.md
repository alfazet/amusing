# Amusing

A Musing client with a beautiful TUI.

## Usage
Amusing's interface is split into three screens: the cover art screen, the queue screen and the library screen.
They are accessible (by default) with keys <kbd>1</kbd>, <kbd>2</kbd> and <kbd>3</kbd> respectively.

### Cover art screen
The "starting screen" of Amusing, where you can admire the current tracks's cover art.[^1]
![musing cover art screen](https://github.com/alfazet/amusing/blob/main/images/cover.png)

### Queue screen
Here queued up songs are displayed. You can play a selected song "out of order" (default: <kbd>Enter</kbd>),
delete a song from the queue (default: <kbd>D</kbd>) or clear the entire queue (default: <kbd>Delete</kbd>).
![musing queue screen](https://github.com/alfazet/amusing/blob/main/images/queue.png)

### Library screen
This screen allows you to browse through your music library. The library is divided into two sections: on the left you select a group of songs (by default songs are grouped by their album artist and then the album, but you can configure it to your liking) and on the right you can see all the titles of all the individual songs that belong to the selected group.
![musing library screen](https://github.com/alfazet/amusing/blob/main/images/library.png)

[^1]: Image rendering is provided by the [ratatui-image](https://github.com/benjajaja/ratatui-image/) crate.
