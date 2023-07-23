pub fn generate_terrian_vertices(width: f32, divisions: i32) -> Vec<f32> {
    let mut output = vec![];
    let triangle_side = width / divisions as f32;
    for row in 0..divisions {
        for col in 0..divisions + 1 {
            // vertex data
            output.push(col as f32 * triangle_side);
            output.push(0.0);
            output.push((row as f32 * -triangle_side) as f32);

            // texture data
            output.push(col as f32 * triangle_side);
            output.push((row as f32 * -triangle_side) as f32);
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
