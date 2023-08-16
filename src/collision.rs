use glm::{vec2, vec3, Vec2, Vec3};

// return true if objects are colliding.
fn test_collision_2d(
    v1: &[Vec2],
    v2: &[Vec2],
) -> bool
{
    for i in 0..v1.len() - 1
    {
        let start = v1[i];
        let end = v1[i + 1 % (v1.len())];
        let direction = end - start;
        let axis = vec2(-direction.y, direction.x);
        let (min_1, max_1) = get_projected_min_max(v1, &axis);
        let (min_2, max_2) = get_projected_min_max(v2, &axis);
        if min_1 > max_2 || min_2 > max_1
        {
            return false;
        }
    }

    for i in 0..v2.len() - 1
    {
        let start = v2[i];
        let end = v2[i + 1 % (v2.len())];
        let direction = end - start;
        let axis = vec2(-direction.y, direction.x);
        let axis = glm::normalize(axis);
        let (min_1, max_1) = get_projected_min_max(v1, &axis);
        let (min_2, max_2) = get_projected_min_max(v2, &axis);
        if min_1 > max_2 || min_2 > max_1
        {
            return false;
        }
    }
    true
}

// return (min, max)
fn get_projected_min_max(
    vertices: &[Vec2],
    axis: &Vec2,
) -> (f32, f32)
{
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    for vertex in vertices
    {
        let proj = vertex.x * axis.x + vertex.y * axis.y; // this is the dot product
        if proj < min
        {
            min = proj;
        }
        if proj > max
        {
            max = proj;
        }
    }
    (min, max)
}

#[cfg(test)]
mod test_2d
{
    use glm::vec2;

    use super::test_collision_2d;

    #[test]
    fn test_colliding()
    {
        let triangle_one = [vec2(0.0, 0.0), vec2(0.0, 1.0), vec2(1.0, 0.0)];
        let triangle_two = [vec2(0.0, 0.0), vec2(0.0, 1.0), vec2(1.0, 0.0)];

        let is_colliding = test_collision_2d(&triangle_one, &triangle_two);

        assert!(is_colliding)
    }

    #[test]
    fn test_not_colliding()
    {
        let triangle_one = [vec2(0.0, 0.0), vec2(0.0, 1.0), vec2(1.0, 0.0)];
        let triangle_two = [vec2(1.0, 1.0), vec2(1.0, 2.0), vec2(2.0, 2.0)];

        let is_colliding = test_collision_2d(&triangle_one, &triangle_two);

        assert!(!is_colliding)
    }
}

pub fn test_collision_3d(
    v1: &[Vec3],
    stride1: usize,
    v2: &[Vec3],
    stride2: usize,
) -> (bool, Option<Vec3>)
{
    let mut normal_output = vec3(0.0, 0.0, 0.0);
    let mut depth = f32::MAX;
    for i in (0..v1.len()).step_by(stride1)
    {
        let a_to_b = v1[i + 1] - v1[i];
        let a_to_c = v1[i + 2] - v1[i];

        let mut normal = glm::cross(a_to_b, a_to_c);
        normal = glm::normalize(normal);
        let (min_1, max_1) = get_projected_min_max_3d(v1, &normal);
        let (min_2, max_2) = get_projected_min_max_3d(v2, &normal);
        if min_1 > max_2 || min_2 > max_1
        {
            return (false, None);
        }

        let axis_depth = f32::min(max_2 - min_1, max_1 - min_2);
        if depth > axis_depth
        {
            depth = axis_depth;
            normal_output = normal;
        }
    }

    for i in (0..v2.len()).step_by(stride2)
    {
        let a_to_b = v2[i + 1] - v2[i];
        let a_to_c = v2[i + 2] - v2[i];

        let mut normal = glm::cross(a_to_b, a_to_c);
        normal = glm::normalize(normal);
        let (min_1, max_1) = get_projected_min_max_3d(v1, &normal);
        let (min_2, max_2) = get_projected_min_max_3d(v2, &normal);
        if min_1 > max_2 || min_2 > max_1
        {
            return (false, None);
        }

        let axis_depth = f32::min(max_2 - min_1, max_1 - min_2);
        if depth > axis_depth
        {
            depth = axis_depth;
            normal_output = normal;
        }
    }
    (true, Some(normal_output))
}

// return (min, max)
fn get_projected_min_max_3d(
    vertices: &[Vec3],
    axis: &Vec3,
) -> (f32, f32)
{
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    for vertex in vertices
    {
        let proj = vertex.x * axis.x + vertex.y * axis.y + vertex.z * axis.z; // this is the dot product
        if proj < min
        {
            min = proj;
        }
        if proj > max
        {
            max = proj;
        }
    }
    (min, max)
}

#[cfg(test)]
mod test_3d
{
    use glm::vec3;

    use crate::collision::test_collision_3d;

    #[test]
    fn test_colliding()
    {
        let triangle_one = [
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            vec3(1.0, 0.0, 0.0),
        ];
        let triangle_two = [
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            vec3(1.0, 0.0, 0.0),
        ];

        let (is_colliding, _) =
            test_collision_3d(&triangle_one, 3 as usize, &triangle_two, 3 as usize);

        assert!(is_colliding)
    }

    #[test]
    fn test_not_colliding()
    {
        let triangle_one = [
            vec3(0.0, 0.0, 1.0),
            vec3(0.0, 1.0, 1.0),
            vec3(1.0, 0.0, 1.0),
            vec3(0.0, 1.0, 1.0),
            vec3(1.0, 1.0, 1.0),
            vec3(1.0, 0.0, 1.0),
        ];
        let triangle_two = [
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            vec3(1.0, 0.0, 0.0),
        ];

        let (is_colliding, _) =
            test_collision_3d(&triangle_one, 3 as usize, &triangle_two, 3 as usize);

        assert!(!is_colliding)
    }
}
