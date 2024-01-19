use gltf::image::{Data, Format};

pub fn convert_texture(data: &Data) -> impl ExactSizeIterator<Item = [u8; 4]> + '_ {
    let (pixel_conversion, chunk_size): (fn(_) -> _, _) = match &data.format {
        Format::R8G8B8 => (convert_r8g8b8, 3),
        Format::R16G16B16 => (convert_r16g16b16, 6),
        Format::R8G8 => (convert_r8g8, 2),
        Format::R16G16 => (convert_r16g16, 4),
        Format::R8G8B8A8 => (convert_r8g8b8a8, 4),
        Format::R16G16B16A16 => (convert_r16g16b16a16, 8),
        Format::R8 => (convert_r8, 1),
        Format::R16 => (convert_r16, 2),
        Format::R32G32B32FLOAT => (convert_r32g32b32, 12),
        Format::R32G32B32A32FLOAT => (convert_r32g32b32a32, 16),
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
        (255.0 * f32::from_be_bytes([pixel[8], pixel[9], pixel[10], pixel[11]])) as u8,
        (255.0 * f32::from_be_bytes([pixel[4], pixel[5], pixel[6], pixel[7]])) as u8,
        (255.0 * f32::from_be_bytes([pixel[0], pixel[1], pixel[2], pixel[3]])) as u8,
        u8::MAX,
    ]
}

fn convert_r32g32b32a32(pixel: &[u8]) -> [u8; 4] {
    [
        (255.0 * f32::from_be_bytes([pixel[8], pixel[9], pixel[10], pixel[11]])) as u8,
        (255.0 * f32::from_be_bytes([pixel[4], pixel[5], pixel[6], pixel[7]])) as u8,
        (255.0 * f32::from_be_bytes([pixel[0], pixel[1], pixel[2], pixel[3]])) as u8,
        (255.0 * f32::from_be_bytes([pixel[12], pixel[13], pixel[14], pixel[15]])) as u8,
    ]
}
