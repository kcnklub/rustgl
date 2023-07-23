pub fn generate_terrian_vertices(width: f32, divisions: i32) -> Vec<f32> {
    let img = image::open("my_height_map.png").unwrap().into_luma8();
    let mut output = vec![];
    let triangle_side = width / divisions as f32;
    for row in 0..divisions {
        for col in 0..divisions + 1 {
            // vertex data
            output.push(col as f32 * triangle_side);

            if col >= divisions {
                let pixel = img.get_pixel(col as u32 - 1, row as u32).0[0];
                output.push(pixel as f32 / 15.0); // can we give this height?
            } else {
                let pixel = img.get_pixel(col as u32, row as u32).0[0];
                output.push(pixel as f32 / 15.0); // can we give this height?
            }

            output.push((row as f32 * triangle_side) as f32);

            output.push((col as f32 * triangle_side) / width);
            output.push((row as f32 * triangle_side) / width);
        }
    }

    output
}

pub fn generate_terrian_ebo(divisions: i32) -> Vec<i32> {
    let mut output = vec![];
    for row in 0..divisions - 1 {
        for col in 0..divisions - 1 {
            let index = row * (divisions + 1) + col;

            output.push(index);
            output.push(index + (divisions + 1) + 1);
            output.push(index + (divisions + 1));

            output.push(index);
            output.push(index + 1);
            output.push(index + (divisions + 1) + 1)
        }
    }

    output
}
