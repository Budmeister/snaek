use super::{
    types::{
        CellObject,
        CellFloor,
        PowerupType,
        Coord,
        ShopState,
        ShopItem,
        NUM_SHOP_ITEMS,
        Board,
        SB_WIDTH,
        SB_HEIGHT,
    },
    art::{
        Fill,
        BoardArt,
        write_letter,
    },
};


pub enum ShopItemFill {
    Remove,
    Clear,
    FromItem,
}

pub static SCORE_BANNER_VERT: &[u8] = include_bytes!("../../res/levels/score_banner_vertical.bin");

const SHOP_X: usize = 2;
const SHOP_Y: usize = 3;
const SHOP_ITEM_H: usize = 15;
const PM_X: usize = SHOP_X + 6;
const PM_Y: usize = SHOP_Y + NUM_SHOP_ITEMS * SHOP_ITEM_H + 2;
const COINS_X: usize = SHOP_X + 6;
const COINS_Y: usize = PM_Y + 7;
pub trait ScoreboardArt: BoardArt {
    fn shop(&mut self, shop: &ShopState) {
        for item_num in 0..NUM_SHOP_ITEMS {
            self.shop_item_display(shop, item_num, if item_num == shop.selected { ShopItemFill::FromItem } else { ShopItemFill::Clear });
        }
    }
    fn shop_remove(&mut self, shop: &ShopState) {
        for item_num in 0..NUM_SHOP_ITEMS {
            self.shop_item_display(shop, item_num, ShopItemFill::Remove);
        }
    }
    fn pm(&mut self, shop: &ShopState) {
        let pm = format!("{}", shop.price_multiplier);
        self.text(&pm, (PM_X, PM_Y), CellObject::None, ());
    }
    fn pm_remove(&mut self, shop: &ShopState) {
        let pm = format!("{}", shop.price_multiplier);
        self.text(&pm, (PM_X, PM_Y), CellObject::Wall, ());
    }
    fn coins(&mut self, coins: usize) {
        let coins = format!("{}", coins);
        self.text(&coins, (COINS_X, COINS_Y), CellObject::None, ());
    }
    fn coins_remove(&mut self, coins: usize) {
        let coins = format!("{}", coins);
        self.text(&coins, (COINS_X, COINS_Y), CellObject::Wall, ());
    }
    fn shop_item_display(&mut self, shop: &ShopState, item_num: usize, fill: ShopItemFill) {
        if item_num > NUM_SHOP_ITEMS {
            return;
        }

        let y = SHOP_Y + SHOP_ITEM_H * item_num;
        let xy = (SHOP_X, y).into();
        let ShopItem { kind, mut price } = shop.powerups[item_num];
        price *= shop.price_multiplier;
        match fill {
            ShopItemFill::Remove => self._shop_item_display(kind, price, xy, (CellFloor::Empty, CellObject::Wall)),
            ShopItemFill::Clear => self._shop_item_display(kind, price, xy, (CellFloor::Empty, CellObject::None)),
            ShopItemFill::FromItem => self._shop_item_display(kind, price, xy, (kind, CellObject::None)),
        }
    }
    fn _shop_item_display(&mut self, kind: PowerupType, price: usize, Coord { x, y }: Coord, fill: impl Fill) {
        // Box
        const EDGE_WIDTH: usize = 7;
        const PRICE_X: usize = EDGE_WIDTH + 5;

        self.line((x + 1, y), (x + EDGE_WIDTH, y), fill);
        self.line((x + EDGE_WIDTH + 1, y + 1), (x + EDGE_WIDTH + 1, y + EDGE_WIDTH), fill);
        self.line((x + 1, y + EDGE_WIDTH + 1), (x + EDGE_WIDTH, y + EDGE_WIDTH + 1), fill);
        self.line((x, y + 1), (x, y + EDGE_WIDTH), fill);

        write_letter(&POWERUP_GRIDS[kind as usize], x + 2, y + 2, self, fill, ());

        let price = format!("{}", price);
        self.text(&price, (x + PRICE_X, y + 2), fill, ());
    }
}
impl ScoreboardArt for Board<SB_WIDTH, SB_HEIGHT> {}

pub const P_WIDTH: usize = 5;
pub const P_HEIGHT: usize = 5;
pub type PowerupGrid = [[bool; P_WIDTH]; P_HEIGHT];

static POWERUP_GRIDS: [PowerupGrid; 5] = {
    const X: bool = true;
    const O: bool = false;
    [
        // Water
        [
            [X, O, O, O, X],
            [X, O, O, O, X],
            [X, O, X, O, X],
            [X, O, X, O, X],
            [O, X, O, X, O],
        ],
        // Explosive
        [
            [X, X, X, X, X],
            [X, O, O, O, O],
            [X, X, X, O, O],
            [X, O, O, O, O],
            [X, X, X, X, X],
        ],
        // Shovel
        [
            [X, O, O, O, X],
            [X, O, O, O, X],
            [X, X, X, X, X],
            [X, O, O, O, X],
            [X, O, O, O, X],
        ],
        // Seed
        [
            [O, X, X, X, X],
            [X, O, O, O, O],
            [O, X, X, X, O],
            [O, O, O, O, X],
            [X, X, X, X, O],
        ],
        // Invincibility
        [
            [X, X, X, X, X],
            [O, O, X, O, O],
            [O, O, X, O, O],
            [O, O, X, O, O],
            [X, X, X, X, X],
        ],
    ]
};
