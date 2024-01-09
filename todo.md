# To Do
1. ~~Add MSPT meter: For draw and logic threads: "{lock} + {processing} / {max}" - in red if lock + processing > max~~
2. ~~Water and Lava flow downhill~~
3. ~~Water and Lava have a "height" value that, along with the elevation, determines where they flow~~
4. ~~When water and lava meet, one "height" of each gets turned into one elevation~~
5. ~~Remove Seasons and add l: Box<dyn LevelState>, which shall be stored on the logic thread stack. Each level will provide its own implementation of LevelState, which will update the board every tick and provide shop information. Unlike GameState and ViewState, LevelState will be a fully fledged object with methods and can mutate itself, but the logic thread only cares that it implements LevelState.~~
6. Fix lava colors
7. Seed has a height value and dist value. If dist < 10, then height has 1% chance to increment, else decrements. Then, for each 5 height, has a 1% to spawn food (which can be red, light green, or dark green now) or coin. Empty can become seed by the same dist formula
8. Seed height affects water flow. It can keep water from flowing, but if it does flow, it takes out the seed.
9. If seed touches lava, it immediately becomes lava with the same height
10. Seed "dist" value has a max speed of 1 per tick
11. Food and powerups no longer spawn naturally. Player spawns with 5 waters, 3 seeds, and 10 coins
12. Add shop where you can spend Gold for all powerups
13. Add Shovel powerup: lets you pick up one piece of dirt and place it back down later
14. Invincibility is an on-demand powerup
15. Explosion also decreases elevation