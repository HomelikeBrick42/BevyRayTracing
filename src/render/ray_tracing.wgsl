@group(0)
@binding(0)
var output_texture: texture_storage_2d<rgba8unorm, write>;

struct Camera {
    transform: Motor,
    v_fov: f32,
    min_distance: f32,
    max_distance: f32,
    max_bounces: u32,
}

@group(1)
@binding(0)
var<uniform> camera: Camera;

struct Sphere {
    transform: Motor,
    color: vec3<f32>,
    radius: f32,
}

struct Spheres {
    length: u32,
    data: array<Sphere>,
}

@group(2)
@binding(0)
var<storage, read> spheres: Spheres;

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct Hit {
    hit: bool,
    distance: f32,
    position: vec3<f32>,
    normal: vec3<f32>,
    color: vec3<f32>,
}

fn intersect_sphere(ray: Ray, sphere: Sphere) -> Hit {
    var hit: Hit;
    hit.hit = false;
    hit.color = sphere.color;

    let sphere_position = point_to_vec3(transform_point(vec3_to_point(vec3<f32>(0.0)), sphere.transform));
    let oc = ray.origin - sphere_position;
    let a = dot(ray.direction, ray.direction);
    let half_b = dot(oc, ray.direction);
    let c = dot(oc, oc) - sphere.radius * sphere.radius;
    let discriminant = half_b * half_b - a * c;

    if discriminant < 0.0 {
        return hit;
    }

    let sqrt_discriminant = sqrt(discriminant);
    let t0 = (-half_b - sqrt_discriminant) / a;
    let t1 = (-half_b + sqrt_discriminant) / a;

    if t0 > camera.min_distance {
        hit.distance = t0;
    } else {
        hit.distance = t1;
    }

    if hit.distance < camera.min_distance || camera.max_distance < hit.distance {
        return hit;
    }

    hit.position = ray.origin + ray.direction * hit.distance;
    hit.normal = normalize(hit.position - sphere_position);
    if dot(hit.normal, ray.origin - hit.position) < 0.0 {
        hit.normal *= -1.0;
    }

    hit.hit = true;
    return hit;
}

fn intersect_ray(ray: Ray) -> Hit {
    var closest_hit: Hit;
    closest_hit.hit = false;

    var sphere_index = 0u;
    while sphere_index < spheres.length {
        let hit = intersect_sphere(ray, spheres.data[sphere_index]);
        if hit.hit && (!closest_hit.hit || hit.distance < closest_hit.distance) {
            closest_hit = hit;
        }
        sphere_index += 1u;
    }

    return closest_hit;
}

fn skybox(ray: Ray) -> vec3<f32> {
    let t = ray.direction.y * 0.5 + 0.5;
    let up = vec3<f32>(0.1, 0.2, 0.8);
    let down = vec3<f32>(0.7, 0.7, 0.8);
    return up * t + down * (1.0 - t);
}

fn trace(ray_: Ray) -> vec3<f32> {
    var ray = ray_;

    var color = vec3<f32>(1.0, 1.0, 1.0);
    var incoming_light = vec3<f32>(0.0, 0.0, 0.0);

    var bounce_index = 0u;
    while bounce_index < camera.max_bounces {
        let hit = intersect_ray(ray);
        if hit.hit {
            ray.origin = hit.position + hit.normal * camera.min_distance;
            ray.direction = reflect(ray.direction, hit.normal);

            let emitted_light = vec3<f32>(0.0);
            incoming_light += emitted_light * color;
            color *= hit.color;
        } else {
            incoming_light += skybox(ray) * color;
            break;
        }
        bounce_index += 1u;
    }

    return incoming_light;
}

@compute
@workgroup_size(16, 16)
fn ray_trace(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let size = textureDimensions(output_texture);
    let coords = global_id.xy;

    if coords.x >= size.x || coords.y >= size.y {
        return;
    }

    var ray: Ray;
    ray.origin = point_to_vec3(transform_point(vec3_to_point(vec3<f32>(0.0)), camera.transform));

    let theta = tan(camera.v_fov / 2.0);
    let aspect = f32(size.x) / f32(size.y);
    let normalized_uv = vec2<f32>(f32(coords.x) / f32(size.x), 1.0 - (f32(coords.y) / f32(size.y))) * 2.0 - 1.0;
    ray.direction = vec3<f32>(1.0, normalized_uv.y * theta, normalized_uv.x * aspect * theta);
    ray.direction = normalize(point_to_vec3(transform_point(vec3_to_point(ray.direction), rotation_part_of_motor(camera.transform))));

    let color = trace(ray);
    textureStore(output_texture, coords.xy, vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0));
}

struct Point {
    e012: f32,
    e013: f32,
    e023: f32,
    e123: f32,
}

fn vec3_to_point(v: vec3<f32>) -> Point {
    var result: Point;
    result.e012 = v.z;
    result.e013 = -v.y;
    result.e023 = v.x;
    result.e123 = 1.0;
    return result;
}

fn point_to_vec3(p: Point) -> vec3<f32> {
    return vec3<f32>(
        p.e023 / p.e123,
        -p.e013 / p.e123,
        p.e012 / p.e123,
    );
}

struct Motor {
    s: f32,
    e12: f32,
    e13: f32,
    e23: f32,
    e01: f32,
    e02: f32,
    e03: f32,
    e0123: f32,
}

fn rotation_part_of_motor(motor: Motor) -> Motor {
    var result = motor;
    result.e01 = 0.0;
    result.e02 = 0.0;
    result.e03 = 0.0;
    result.e0123 = 0.0;
    return result;
}

fn transform_point(point: Point, motor: Motor) -> Point {
    let a = motor.s;
    let b = motor.e12;
    let c = motor.e13;
    let d = motor.e23;
    let e = motor.e01;
    let f = motor.e02;
    let g = motor.e03;
    let h = motor.e0123;
    let i = point.e012;
    let j = point.e013;
    let k = point.e023;
    let l = point.e123;

    var result: Point;
    result.e012 = -2.0 * a * d * j + -2.0 * a * g * l + 1.0 * a * a * i + 2.0 * a * c * k + -1.0 * d * d * i + -2.0 * d * f * l + 2.0 * b * d * k + -2.0 * b * h * l + -2.0 * c * e * l + 1.0 * b * b * i + 2.0 * b * c * j + -1.0 * c * c * i;
    result.e013 = -2.0 * a * b * k + -1.0 * b * b * j + 2.0 * b * c * i + 2.0 * b * e * l + 1.0 * a * a * j + 2.0 * a * d * i + 2.0 * a * f * l + -2.0 * c * h * l + -2.0 * d * g * l + -1.0 * d * d * j + 2.0 * c * d * k + 1.0 * c * c * j;
    result.e023 = -2.0 * a * c * i + -2.0 * a * e * l + 1.0 * a * a * k + 2.0 * a * b * j + -1.0 * c * c * k + 2.0 * c * d * j + 2.0 * c * g * l + -2.0 * d * h * l + 2.0 * b * f * l + -1.0 * b * b * k + 2.0 * b * d * i + 1.0 * d * d * k;
    result.e123 = a * a * l + b * b * l + c * c * l + d * d * l;
    return result;
}

fn inverse_motor(motor: Motor) -> Motor {
    var result = motor;
    result.e12 = -motor.e12;
    result.e13 = -motor.e13;
    result.e23 = -motor.e23;
    result.e01 = -motor.e01;
    result.e02 = -motor.e02;
    result.e03 = -motor.e03;
    return result;
}
