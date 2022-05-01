/* CHIP-8 font data */
// 8 width x 5 height letters 0-F
// Index = hex number * 5
// Length: 80
pub static FONT_DATA: [u8; 80] = [
    96, 144, 144, 144,  96,  96,  32,  32,  32, 112, 192,  32,
    96, 128,  96, 192,  32, 224,  32, 192, 144, 144, 112,  16,
    16, 112,  64, 112,  16,  96, 112, 128, 224, 144,  96, 112,
    16,  32,  32,  64,  96, 144,  96, 144,  96,  48,  80, 112,
    16,  96,  96, 160, 224, 160, 160, 192, 160, 224, 160, 192,
   112, 128, 128, 128, 112,  96,  80,  80,  80,  96, 112, 128,
   240, 128, 112, 112,  64, 112,  64,  64
];