
<h1 align="center">
aurora
</h1>
<p align="center">
A music player.
</p>

Indev by [@mavenried](https://github.com/mavenried).

## Todo
- [x] better animations for the slider
- [x] rotate queue on next instead of changing index (ended up switching to a VecDeque)
- [x] playlists page
- [x] fix that weird stutter when adding songs.
- [x] add ability to replace queue with playlists (ie. Load a playlist as a queue)
- [ ] hot reloadable themes (json configs?)
- [ ] add ability to add songs to playlists from the search menu
- [ ] add ability to create and delete playlists
- [ ] FIX THE UNGODLY AMOUNT OF RAM USAGE! (PRIORITY)
  - [ ] eliminate the double caching. (both slint and the 'business logic' cache images)
- [ ] add batching support for commands like album art
