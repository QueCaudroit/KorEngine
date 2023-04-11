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
