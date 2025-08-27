use pimage::*;
use std::{
    io::{Error, Result},
    slice::Iter,
};

const EOF_ERR: &str = "End of file before end bytes.";

//
//
//
//
// TODO: DataChunk -> from_iter
//       Then all_pixels
//
//
//
//

#[allow(dead_code)]
/// A DataChunk is a chunk of data saved in a QOI file.
#[derive(PartialEq)]
enum DataChunk {
    /// 4 bytes:
    ///   tag: b11111110,
    ///   8 bit red, green, blue
    Rgb(u8, u8, u8),
    /// 5 bytes,
    ///   tag: b11111111,
    ///   8 bit red, green, blue, alpha
    Rgba(u8, u8, u8, u8),
    /// 1 byte,
    ///   tag: b00,
    ///   6 bits: index into the color array (0..63)
    Index(u8),
    /// 1 byte,
    ///   tag: b01,
    ///   2 bits: red difference from previous pixel (-2..1)
    ///   2 bits: green difference from previous pixel (-2..1)
    ///   2 bits: blue difference from previous pixel (-2..1)
    Diff(u8, u8, u8),
    /// 2 bytes,
    ///   tag: b10,
    ///   6 bits: green from previous (-32..31)
    ///   4 bits: red from previous, minus green diff (-8..7)
    ///   4 bits: blue from previous, minus green diff (-8..7)
    /// See qoi-specifiction
    Luma(u8, u8, u8),
    /// 1 byte,
    ///   tag: b11,
    ///   6 bits: length of repeating previous pixel (1..62)
    ///   !!! CAN'T BE 63 OR 64 !!!
    ///   that would make it b11111110 and b11111111, which are tags for other DataChunk.
    Run(u8),
}

fn next_byte(iter: &mut Iter<'_, u8>, string: &str) -> Result<u8> {
    if let Some(tmp) = iter.next() {
        Ok(*tmp)
    } else {
        error(string)
    }
}

#[allow(dead_code, unused_variables)]
impl DataChunk {
    fn is_0(&self) -> bool {
        *self == DataChunk::Index(0)
    }
    fn is_1(&self) -> bool {
        *self == DataChunk::Index(1)
    }
    fn should_update_array(&self) -> bool {
        !matches!(self, DataChunk::Index(_) | DataChunk::Run(_))
    }
    fn from_iter(iter: &mut Iter<'_, u8>) -> Result<DataChunk> {
        let first = next_byte(iter, EOF_ERR)?;
        if first == 0b11111110 {
            let r = next_byte(iter, EOF_ERR)?;
            let g = next_byte(iter, EOF_ERR)?;
            let b = next_byte(iter, EOF_ERR)?;
            return Ok(DataChunk::Rgb(r, g, b));
        }
        if first == 0b11111111 {
            let r = next_byte(iter, EOF_ERR)?;
            let g = next_byte(iter, EOF_ERR)?;
            let b = next_byte(iter, EOF_ERR)?;
            let a = next_byte(iter, EOF_ERR)?;
            return Ok(DataChunk::Rgba(r, g, b, a));
        }
        if (first & 0b11000000) == 0b00000000 {
            return Ok(DataChunk::Index(first & 0b00111111));
        }
        if (first & 0b11000000) == 0b01000000 {
            return Ok(DataChunk::Diff(
                (first & 0b00110000) >> 4,
                (first & 0b00001100) >> 2,
                first & 0b00000011,
            ));
        }
        if (first & 0b11000000) == 0b10000000 {
            let second = next_byte(iter, EOF_ERR)?;
            return Ok(DataChunk::Luma(
                first & 0b00111111,
                (second & 0b11110000) >> 4,
                second & 0b00001111,
            ));
        }
        if (first & 0b11000000) == 0b11000000 {
            return Ok(DataChunk::Run(first & 0b00111111));
        }
        error("Not a possible QOI encoding.")
    }
    fn to_color(&self, previous: &Color, array: &[Color; 64]) -> Vec<Color> {
        match self {
            /*
            DataChunk::Rgb(_, _, _) => vec![Color::RED],
            DataChunk::Rgba(_, _, _, _) => vec![Color::GREEN],
            DataChunk::Index(_) => vec![Color::BLUE],
            DataChunk::Diff(_, _, _) => vec![Color::YELLOW],
            DataChunk::Luma(_, _, _) => vec![Color::PINK],
            DataChunk::Run(count) => vec![Color::CYAN; *count as usize + 1],
            */
            DataChunk::Rgb(r, g, b) => vec![Color::new_alpha(*r, *g, *b, previous.a)],
            DataChunk::Rgba(r, g, b, a) => vec![Color::new_alpha(*r, *g, *b, *a)],
            DataChunk::Index(index) => vec![array[*index as usize]],
            DataChunk::Diff(dr, dg, db) => vec![Color::new_alpha(
                previous.r.wrapping_add(*dr).wrapping_sub(2),
                previous.g.wrapping_add(*dg).wrapping_sub(2),
                previous.b.wrapping_add(*db).wrapping_sub(2),
                previous.a,
            )],
            DataChunk::Luma(dg, dr_dg, db_dg) => vec![Color::new_alpha(
                previous
                    .r
                    .wrapping_add(*dr_dg)
                    .wrapping_add(*dg)
                    .wrapping_sub(40),
                previous.g.wrapping_add(*dg).wrapping_sub(32),
                previous
                    .b
                    .wrapping_add(*db_dg)
                    .wrapping_add(*dg)
                    .wrapping_sub(40),
                previous.a,
            )],
            DataChunk::Run(count) => vec![*previous; *count as usize + 1],
        }
    }

    fn all_pixels(iter: &mut Iter<'_, u8>) -> Result<Vec<Color>> {
        let mut previous = Color::BLACK;
        let mut array = [Color::TRANS; 64];
        let mut pixels = vec![];
        let mut zero_count = 0;
        loop {
            let next_data = DataChunk::from_iter(iter)?;
            if next_data.is_0() {
                zero_count += 1;
            } else if zero_count >= 7 && next_data.is_1() {
                break;
            } else {
                zero_count = 0;
            }
            let next_color = next_data.to_color(&previous, &array);
            pixels.push(next_color);
            previous = *pixels.last().unwrap().last().unwrap();
            if next_data.should_update_array() {
                array[index_position(&previous)] = previous;
            }
        }
        let mut result: Vec<Color> = pixels.iter().flatten().copied().collect();
        result.truncate(result.len() - 7);
        Ok(result)
    }
}

fn index_position(color: &Color) -> usize {
    let r = color.r as usize;
    let g = color.g as usize;
    let b = color.b as usize;
    let a = color.a as usize;
    (3 * r + 5 * g + 7 * b + 11 * a) % 64
}

#[allow(dead_code)]
#[derive(Default, Debug)]
struct Header {
    /// Magic number "qoif"
    magic: [char; 4],
    /// Image width in pixels
    width: u32,
    /// Image height in pixels
    height: u32,
    /// 3 => RGB, 4 => RGBA
    channels: u8,
    /// 0 => sRGB with linear alpha
    /// 1 => All channels linear
    colorspace: u8,
}

impl Header {
    fn from_iter(iter: &mut Iter<'_, u8>) -> Result<Header> {
        let mut header = Header::default();
        for index in 0..4 {
            if let Some(character) = iter.next() {
                header.magic[index] = *character as char;
            } else {
                return error("Not enough data in file, can't find magic number.");
            }
        }
        if header.magic != ['q', 'o', 'i', 'f'] {
            return error("Wrong magic number at start of file, not a QOI file.");
        }
        header.width = u32_from_iter(iter)?;
        header.height = u32_from_iter(iter)?;
        if let Some(channels) = iter.next() {
            header.channels = *channels;
        } else {
            return error("Not enought data in file, can't find channels");
        }
        if let Some(color) = iter.next() {
            header.colorspace = *color;
        } else {
            return error("Not enought data in file, can't find color space");
        }

        Ok(header)
    }
}

fn u32_from_iter(iter: &mut Iter<'_, u8>) -> Result<u32> {
    let mut tmp = [0, 0, 0, 0];
    for val in tmp.iter_mut() {
        if let Some(num) = iter.next() {
            *val = *num;
        } else {
            return error("Not enough data to determine width.");
        }
    }
    Ok(u32::from_be_bytes(tmp))
}

#[allow(dead_code, unused_variables)]
/// Loads a QOI (compressed raster image) and returns it as a Pimage.
pub fn load_qoi(path_name: &str) -> Result<Pimage> {
    let file = std::fs::read(path_name)?;
    let mut file_iter = file.iter();
    let header = Header::from_iter(&mut file_iter)?;
    let pixels = DataChunk::all_pixels(&mut file_iter)?;
    let mut pimage = Pimage::new(header.width as usize, header.height as usize, Color::BLACK);
    if pixels.len() > pimage.width() * pimage.height() {
        return error("Too many pixels.");
    }
    if pixels.len() < pimage.width() * pimage.height() {
        return error("Not enough pixels.");
    }
    for y in 0..pimage.height() {
        for x in 0..pimage.width() {
            let image_error = pimage.set(x, y, *pixels.get(x + y * pimage.width()).unwrap());
            if let Err(string) = image_error {
                return error(string);
            }
        }
    }

    Ok(pimage)
}

/// Writes a Pimage to a QOI file.
pub fn write_qoi(_path_name: &str, _pimage: &Pimage) -> Result<()> {
    error("not yet done")
}

fn error<T>(string: &str) -> Result<T> {
    Err(Error::other(string))
}
