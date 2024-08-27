use num_enum::TryFromPrimitive;

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct Stack {
    pos: (u16, u16),
    stack_type: StackType,
    amount: u8,
    unknown0: i8, // always -2
    unknown1: u8,
}

#[derive(Copy, Clone, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum StackType {
    Agave = 1,
    Ammo,
    Armor,
    Axe,
    BattleAxe,
    Blowgun,
    Board,
    Bow,
    Bread,
    Coal,
    Fish,
    Flour,
    Goat,
    GoldBar,
    GoldOre,
    Grain,
    Gunpowder,
    Hammer,
    Honey,
    IronBar,
    IronOre,
    Log,
    Mead,
    Meat,
    Pickaxe,
    Pig,
    FishingRod,
    Saw,
    Scythe,
    Sheep,
    Shovel,
    Stone,
    Sulfur,
    Sword,
    Tequila,
    Water,
    Wine,
    BackpackCatapult,
    Goose,
    ExplosiveArrow,
    SunflowerOil,
    Sunflower,
}
