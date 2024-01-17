# To Do
1. ~~Fix volcano level so that ridges do not diverge~~
2. Food and powerups no longer spawn naturally. Player spawns with 300 coins.
3. Instead of shop, give the player 3 powerups to choose from, randomly chosen (but unique) and with their respective prices. 
4. Plants grow food and earn coins (but not visibly). If you eat food, you get longer. The coins are spent in the shop. At the end of the level, your length is converted to coins to calculate score. Length is more valuable than coins.
5. Add Shovel powerup: lets you pick up one piece of dirt and place it back down later.
6. Invincibility is an on-demand powerup
7. Explosion only decreases elevation and does nothing else
8. It is a loss condition that the last seed is gone.
9. Maybe we add a tutorial at some point?

## Powerups
1. Water
2. Explosive
3. Shovel
4. Seed
5. Invincibility

## Pricing
The price of each item is gonna be multiplied by the price multiplier (PM), which starts at 10 and has a random chance of incrementing each frame (say, 0.002). When they buy something, the shop resets. The first two times they shop it is water and seed. After that, they can choose one of 3 random powerups for a price of {price of item} * PM (at the time the shop is loaded). I'll start with the prices all at 10x and adjust them later for game balance. That way, as long as the player places their first forest/garden within a reasonable amount of time from the start, they'll have enough coins to place it. I might show the PM on the screen.

The player buys powerups in two steps: first, they select it by pressing the right key, and second, they press space to buy and activate the powerup.