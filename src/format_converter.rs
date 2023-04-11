#[allow(non_snake_case)]
pub fn convert_R8G8B8(data: &Vec<u8>) -> Vec<u8> {
    let size = data.len() / 3;
    let mut result = Vec::with_capacity(size * 4);
    for i in 0..size {
        result.push(data[3 * i + 2]);
        result.push(data[3 * i + 1]);
        result.push(data[3 * i]);
        result.push(u8::MAX);
    }
    result
}

#[allow(non_snake_case)]
pub fn convert_R16G16B16(data: &Vec<u8>) -> Vec<u8> {
    let size = data.len() / 6;
    let mut result = Vec::with_capacity(size * 4);
    for i in 0..size {
        result.push(data[6 * i + 4]);
        result.push(data[6 * i + 2]);
        result.push(data[6 * i]);
        result.push(u8::MAX);
    }
    result
}

#[allow(non_snake_case)]
pub fn convert_R8G8(data: &Vec<u8>) -> Vec<u8> {
    let size = data.len() / 2;
    let mut result = Vec::with_capacity(size * 4);
    for i in 0..size {
        result.push(0);
        result.push(data[2 * i + 1]);
        result.push(data[2 * i]);
        result.push(u8::MAX);
    }
    result
}

#[allow(non_snake_case)]
pub fn convert_R16G16(data: &Vec<u8>) -> Vec<u8> {
    let size = data.len() / 3;
    let mut result = Vec::with_capacity(size * 4);
    for i in 0..size {
        result.push(data[0]);
        result.push(data[4 * i + 2]);
        result.push(data[4 * i]);
        result.push(u8::MAX);
    }
    result
}

#[allow(non_snake_case)]
pub fn convert_R8G8B8A8(data: &Vec<u8>) -> Vec<u8> {
    let size = data.len() / 4;
    let mut result = Vec::with_capacity(size * 4);
    for i in 0..size {
        result.push(data[4 * i + 2]);
        result.push(data[4 * i + 1]);
        result.push(data[4 * i]);
        result.push(data[4 * i + 3]);
    }
    result
}

#[allow(non_snake_case)]
pub fn convert_R16G16B16A16(data: &Vec<u8>) -> Vec<u8> {
    let size = data.len() / 8;
    let mut result = Vec::with_capacity(size * 4);
    for i in 0..size {
        result.push(data[8 * i + 4]);
        result.push(data[8 * i + 2]);
        result.push(data[8 * i]);
        result.push(data[8 * i + 6]);
    }
    result
}

#[allow(non_snake_case)]
pub fn convert_R8(data: &Vec<u8>) -> Vec<u8> {
    let size = data.len();
    let mut result = Vec::with_capacity(size * 4);
    for i in 0..size {
        result.push(0);
        result.push(0);
        result.push(data[i]);
        result.push(u8::MAX);
    }
    result
}

#[allow(non_snake_case)]
pub fn convert_R16(data: &Vec<u8>) -> Vec<u8> {
    let size = data.len() / 2;
    let mut result = Vec::with_capacity(size * 4);
    for i in 0..size {
        result.push(0);
        result.push(0);
        result.push(data[2 * i]);
        result.push(u8::MAX);
    }
    result
}

#[allow(non_snake_case)]
pub fn convert_R32G32B32(data: &Vec<u8>) -> Vec<u8> {
    let size = data.len() / 12;
    let mut result = Vec::with_capacity(size * 4);
    for i in 0..size {
        let red = f32::from_bits(
            ((data[12 * i] as u32) << 24)
                + ((data[12 * i + 1] as u32) << 16)
                + ((data[12 * i + 2] as u32) << 8)
                + (data[12 * i + 3] as u32),
        );
        let green = f32::from_bits(
            ((data[12 * i + 4] as u32) << 24)
                + ((data[12 * i + 5] as u32) << 16)
                + ((data[12 * i + 6] as u32) << 8)
                + (data[12 * i + 7] as u32),
        );
        let blue = f32::from_bits(
            ((data[12 * i + 8] as u32) << 24)
                + ((data[12 * i + 9] as u32) << 16)
                + ((data[12 * i + 10] as u32) << 8)
                + (data[12 * i + 11] as u32),
        );
        result.push((255.0 * blue) as u8);
        result.push((255.0 * green) as u8);
        result.push((255.0 * red) as u8);
        result.push(u8::MAX);
    }
    result
}

#[allow(non_snake_case)]
pub fn convert_R32G32B32A32(data: &Vec<u8>) -> Vec<u8> {
    let size = data.len() / 16;
    let mut result = Vec::with_capacity(size * 4);
    for i in 0..size {
        let red = f32::from_bits(
            ((data[16 * i] as u32) << 24)
                + ((data[16 * i + 1] as u32) << 16)
                + ((data[16 * i + 2] as u32) << 8)
                + (data[16 * i + 3] as u32),
        );
        let green = f32::from_bits(
            ((data[16 * i + 4] as u32) << 24)
                + ((data[16 * i + 5] as u32) << 16)
                + ((data[16 * i + 6] as u32) << 8)
                + (data[16 * i + 7] as u32),
        );
        let blue = f32::from_bits(
            ((data[16 * i + 8] as u32) << 24)
                + ((data[16 * i + 9] as u32) << 16)
                + ((data[16 * i + 10] as u32) << 8)
                + (data[16 * i + 11] as u32),
        );
        let alpha = f32::from_bits(
            ((data[16 * i + 12] as u32) << 24)
                + ((data[16 * i + 13] as u32) << 16)
                + ((data[16 * i + 14] as u32) << 8)
                + (data[16 * i + 15] as u32),
        );
        result.push((255.0 * blue) as u8);
        result.push((255.0 * green) as u8);
        result.push((255.0 * red) as u8);
        result.push((255.0 * alpha) as u8);
    }
    result
}
