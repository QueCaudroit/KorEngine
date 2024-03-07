use gltf::image::{Data, Format};

pub fn convert_texture(data: &Data) -> impl ExactSizeIterator<Item = u8> + '_ {
    let (pixel_conversion, chunk_size): (fn(_) -> _, _) = match &data.format {
        Format::R8G8B8 => (convert_int, 3),
        Format::R16G16B16 => (convert_int, 6),
        Format::R8G8 => (convert_int, 2),
        Format::R16G16 => (convert_int, 4),
        Format::R8G8B8A8 => (convert_int, 4),
        Format::R16G16B16A16 => (convert_int, 8),
        Format::R8 => (convert_int, 1),
        Format::R16 => (convert_int, 2),
        Format::R32G32B32FLOAT => (convert_float, 12),
        Format::R32G32B32A32FLOAT => (convert_float, 16),
    };
    data.pixels.chunks(chunk_size).map(pixel_conversion)
}

fn convert_int(pixel: &[u8]) -> u8 {
    pixel[0]
}

fn convert_float(pixel: &[u8]) -> u8 {
    (255.0 * f32::from_be_bytes([pixel[0], pixel[1], pixel[2], pixel[3]])) as u8
}
