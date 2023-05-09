use gltf::image::{Data, Format};

pub fn convert_texture(data: &Data) -> impl ExactSizeIterator<Item = [u8; 4]> + '_ {
    let pixel_conversion = match &data.format {
        Format::R8G8B8 => convert_r8g8b8,
        Format::R16G16B16 => convert_r16g16b16,
        Format::R8G8 => convert_r8g8,
        Format::R16G16 => convert_r16g16,
        Format::R8G8B8A8 => convert_r8g8b8a8,
        Format::R16G16B16A16 => convert_r16g16b16a16,
        Format::R8 => convert_r8,
        Format::R16 => convert_r16,
        Format::R32G32B32FLOAT => convert_r32g32b32,
        Format::R32G32B32A32FLOAT => convert_r32g32b32a32,
    };
    let chunk_size = match data.format {
        Format::R8G8B8 => 3,
        Format::R16G16B16 => 6,
        Format::R8G8 => 2,
        Format::R16G16 => 4,
        Format::R8G8B8A8 => 4,
        Format::R16G16B16A16 => 8,
        Format::R8 => 1,
        Format::R16 => 2,
        Format::R32G32B32FLOAT => 12,
        Format::R32G32B32A32FLOAT => 16,
    };
    data.pixels.chunks(chunk_size).map(pixel_conversion)
}

fn convert_r8g8b8(pixel: &[u8]) -> [u8; 4] {
    [pixel[2], pixel[1], pixel[0], u8::MAX]
}

fn convert_r16g16b16(pixel: &[u8]) -> [u8; 4] {
    [pixel[4], pixel[2], pixel[0], u8::MAX]
}

fn convert_r8g8(pixel: &[u8]) -> [u8; 4] {
    [0, pixel[1], pixel[0], u8::MAX]
}

fn convert_r16g16(pixel: &[u8]) -> [u8; 4] {
    [0, pixel[2], pixel[0], u8::MAX]
}

fn convert_r8g8b8a8(pixel: &[u8]) -> [u8; 4] {
    [pixel[2], pixel[1], pixel[0], pixel[3]]
}

fn convert_r16g16b16a16(pixel: &[u8]) -> [u8; 4] {
    [pixel[4], pixel[2], pixel[0], pixel[6]]
}

fn convert_r8(pixel: &[u8]) -> [u8; 4] {
    [0, 0, pixel[0], u8::MAX]
}

fn convert_r16(pixel: &[u8]) -> [u8; 4] {
    [0, 0, pixel[0], u8::MAX]
}

fn convert_r32g32b32(pixel: &[u8]) -> [u8; 4] {
    [
        convert_float_bits(&pixel[8..12]),
        convert_float_bits(&pixel[4..8]),
        convert_float_bits(&pixel[0..4]),
        u8::MAX,
    ]
}

fn convert_r32g32b32a32(pixel: &[u8]) -> [u8; 4] {
    [
        convert_float_bits(&pixel[8..12]),
        convert_float_bits(&pixel[4..8]),
        convert_float_bits(&pixel[0..4]),
        convert_float_bits(&pixel[12..16]),
    ]
}

fn convert_float_bits(channel: &[u8]) -> u8 {
    (255.0
        * f32::from_bits(
            ((channel[0] as u32) << 24)
                + ((channel[1] as u32) << 16)
                + ((channel[2] as u32) << 8)
                + (channel[3] as u32),
        )) as u8
}
