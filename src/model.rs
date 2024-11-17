use std::fs::File;
use std::path::Path;

use anyhow::anyhow;
use nalgebra::{Point3, Rotation3, Vector3};
use rust_3d::{io, IsFaceEditableMesh, Mesh3D, Point3D, PointCloud3D, Precision};
use ttf2mesh::{Quality, TTFFile, Value};
use ttf2mesh::Mesh as TTFMesh;
use ttf2mesh::Mesh3d as TTFMesh3d;

use crate::CommitCount;

#[allow(clippy::redundant_clone)]
fn quad3d(points: [Point3D; 8]) -> Vec<(Point3D, Point3D, Point3D)> {
    let [p0, p1, p2, p3, p4, p5, p6, p7] = points;
    vec![
        (p0.clone(), p1.clone(), p2.clone()),
        (p0.clone(), p2.clone(), p3.clone()), // bottom
        (p4.clone(), p5.clone(), p6.clone()),
        (p4.clone(), p6.clone(), p7.clone()), // top
        (p3.clone(), p0.clone(), p4.clone()),
        (p3.clone(), p4.clone(), p7.clone()), // left
        (p1.clone(), p2.clone(), p6.clone()),
        (p1.clone(), p6.clone(), p5.clone()), // right
        (p0.clone(), p1.clone(), p5.clone()),
        (p0.clone(), p5.clone(), p4.clone()), // front
        (p2.clone(), p3.clone(), p7.clone()),
        (p2.clone(), p7.clone(), p6.clone()), // back
    ]
}

fn commit_brick(
    center: &Point3D,
    length: f64,
    width: f64,
    height: f64,
) -> Vec<(Point3D, Point3D, Point3D)> {
    let half_length = length * 0.5;
    let half_width = width * 0.5;
    let half_height = height * 0.5;
    let p0 = Point3D::new(
        center.x - half_length,
        center.y - half_width,
        center.z - half_height,
    );
    let p1 = Point3D::new(
        center.x + half_length,
        center.y - half_width,
        center.z - half_height,
    );
    let p2 = Point3D::new(
        center.x + half_length,
        center.y + half_width,
        center.z - half_height,
    );
    let p3 = Point3D::new(
        center.x - half_length,
        center.y + half_width,
        center.z - half_height,
    );
    let p4 = Point3D::new(
        center.x - half_length,
        center.y - half_width,
        center.z + half_height,
    );
    let p5 = Point3D::new(
        center.x + half_length,
        center.y - half_width,
        center.z + half_height,
    );
    let p6 = Point3D::new(
        center.x + half_length,
        center.y + half_width,
        center.z + half_height,
    );
    let p7 = Point3D::new(
        center.x - half_length,
        center.y + half_width,
        center.z + half_height,
    );
    quad3d([p0, p1, p2, p3, p4, p5, p6, p7])
}

fn plinth(
    center: &Point3D,
    top_length: f64,
    top_width: f64,
    bottom_length: f64,
    bottom_width: f64,
    height: f64,
) -> Vec<(Point3D, Point3D, Point3D)> {
    let half_top_length = top_length * 0.5;
    let half_top_width = top_width * 0.5;
    let half_bottom_length = bottom_length * 0.5;
    let half_bottom_width = bottom_width * 0.5;
    let half_height = height * 0.5;
    let p0 = Point3D::new(
        center.x - half_bottom_length,
        center.y - half_bottom_width,
        center.z - half_height,
    );
    let p1 = Point3D::new(
        center.x + half_bottom_length,
        center.y - half_bottom_width,
        center.z - half_height,
    );
    let p2 = Point3D::new(
        center.x + half_bottom_length,
        center.y + half_bottom_width,
        center.z - half_height,
    );
    let p3 = Point3D::new(
        center.x - half_bottom_length,
        center.y + half_bottom_width,
        center.z - half_height,
    );
    let p4 = Point3D::new(
        center.x - half_top_length,
        center.y - half_top_width,
        center.z + half_height,
    );
    let p5 = Point3D::new(
        center.x + half_top_length,
        center.y - half_top_width,
        center.z + half_height,
    );
    let p6 = Point3D::new(
        center.x + half_top_length,
        center.y + half_top_width,
        center.z + half_height,
    );
    let p7 = Point3D::new(
        center.x - half_top_length,
        center.y + half_top_width,
        center.z + half_height,
    );
    quad3d([p0, p1, p2, p3, p4, p5, p6, p7])
}

fn trophy_text(
    text: String,
    ttf_font_path: &Path,
    depth: f32,
) -> anyhow::Result<Vec<Option<TTFMesh<TTFMesh3d>>>> {
    let mut font = match TTFFile::from_file(ttf_font_path) {
        Ok(f) => Ok(f),
        Err(_) => Err(anyhow!("failed to load font: {:?}", ttf_font_path))
    }?;

    let mut meshes = vec![];

    for char in text.chars() {
        if let Ok(mut glyph) = font.glyph_from_char(char) {
            if let Ok(mesh) = glyph.to_3d_mesh(Quality::Medium, depth) {
                meshes.push(Some(mesh));
                continue;
            }
        }
        meshes.push(None)
    }
    Ok(meshes)
}

pub struct GeometryConfig {
    top_length: f32,
    top_width: f32,
    plinth_height: f32,
    top_margin: f32,
    bottom_margin: f32,
    maximum_commit_height: f32,
    text_depth: f32,
    text_height: f32,
}

impl Default for GeometryConfig {
    fn default() -> Self {
        GeometryConfig {
            top_length: 52.0, // weeks
            top_width: 7.0, // days per week
            plinth_height: 2.0,
            top_margin: 1.0, // 0.5 on both sides of commit
            bottom_margin: 3.0, // > top margin => slope
            maximum_commit_height: 10.0, // limit commit height
            text_depth: 0.2,
            text_height: 2.0, // must be below sqrt(plinth_height^2 + (bottom_margin * 0.5 - top_margin * 0.5)^2)
        }
    }
}

pub fn build_trophy(
    heightmap: &[CommitCount],
    side_text: Option<String>,
    ttf_font_path: Option<&Path>,
    output_path: &Path,
    geometry_config: GeometryConfig,
) -> anyhow::Result<()> {
    let mut mesh: Mesh3D<Point3D, PointCloud3D<Point3D>, Vec<usize>> = Mesh3D::default();
    let plinth_faces = plinth(
        &Point3D::new(0.0, 0.0, 0.0),
        (geometry_config.top_length + geometry_config.top_margin) as f64,
        (geometry_config.top_width + geometry_config.top_margin) as f64,
        (geometry_config.top_length + geometry_config.bottom_margin) as f64,
        (geometry_config.top_width + geometry_config.bottom_margin) as f64,
        geometry_config.plinth_height as f64,
    );
    let mut brick_faces: Vec<(Point3D, Point3D, Point3D)> = vec![];

    let max_commits = *heightmap.iter().max().unwrap_or(&1);

    for x in 0..52 {
        for y in 0..7 {
            let day = x * 7 + y;
            let commits = *heightmap.get(day).unwrap_or(&0);
            if commits == 0 {
                continue;
            }
            let brick_height_normalized =
                commits as f64 * geometry_config.maximum_commit_height as f64 / max_commits as f64;
            let brick_center = Point3D::new(
                x as f64 - geometry_config.top_length as f64 * 0.5 + 0.5,
                y as f64 - geometry_config.top_width as f64 * 0.5 + 0.5,
                brick_height_normalized * 0.5 + geometry_config.plinth_height as f64 * 0.5,
            );
            let brick = commit_brick(&brick_center, 1.0, 1.0, brick_height_normalized as f64);
            brick_faces.extend_from_slice(brick.as_slice());
        }
    }
    for (p0, p1, p2) in plinth_faces.iter().chain(brick_faces.iter()) {
        mesh.add_face(p0.clone(), p1.clone(), p2.clone());
    }

    if let Some(side_text) = side_text {
        let trophy_text_meshes = trophy_text(
            String::from(side_text),
            ttf_font_path.ok_or_else(|| anyhow!("please provide a TTF font path"))?,
            geometry_config.text_depth,
        )?;

        let text_max_z = trophy_text_meshes.iter().filter_map(
            |v|
                v.as_ref().map(
                    |v|
                        v.iter_vertices().map(
                            |v| {
                                let v = v.val();
                                v.1
                            }).into_iter().reduce(f32::max).unwrap()
                )
        ).into_iter().reduce(f32::max).unwrap();

        let text_min_z = trophy_text_meshes.iter().filter_map(
            |v|
                v.as_ref().map(
                    |v|
                        v.iter_vertices().map(
                            |v| {
                                let v = v.val();
                                v.1
                            }).into_iter().reduce(f32::min).unwrap()
                )
        ).into_iter().reduce(f32::min).unwrap();

        let max_text_height= text_max_z - text_min_z;
        let drop = text_min_z.min(0.0).abs();
        let text_scaling = geometry_config.text_height / max_text_height;
        let drop_scaled = drop * text_scaling;

        let x_axis = Vector3::x_axis();
        let x = (geometry_config.bottom_margin * 0.5 - geometry_config.top_margin * 0.5).abs();
        let y = geometry_config.plinth_height;
        let angle_rad= y.atan2(x);
        let rotation = Rotation3::from_axis_angle(&x_axis, angle_rad);
        let hyp = (x * x + y * y).sqrt();
        // center the text on the side
        let dhyp = (hyp - geometry_config.text_height) * 0.5 + drop_scaled;
        let dx = angle_rad.cos() * dhyp;
        let dy = angle_rad.sin() * dhyp;
        // pull text from plinth
        let dx = dx - angle_rad.sin() * geometry_config.text_depth * 0.5 * text_scaling;
        let dy = dy + angle_rad.cos() * geometry_config.text_depth * 0.5 * text_scaling;

        for (glyph_i, glyph_mesh) in trophy_text_meshes.iter().enumerate() {
            if let Some(glyph_mesh) = glyph_mesh {
                let vertices = glyph_mesh.iter_vertices().map(|v| v.val()).collect::<Vec<(f32, f32, f32)>>();
                for face in glyph_mesh.iter_faces() {
                    let (v1, v2, v3) = face.val();
                    let p1 = vertices.get(v1 as usize);
                    let p2 = vertices.get(v2 as usize);
                    let p3 = vertices.get(v3 as usize);
                    match (p1, p2, p3) {
                        (Some(p1), Some(p2), Some(p3)) => {
                            let translation_after_rotation = Vector3::new(
                                (glyph_i as f32 * text_scaling * 0.5) - geometry_config.top_length * 0.5,
                                - (geometry_config.top_width + geometry_config.bottom_margin) * 0.5 + dx,
                                - geometry_config.plinth_height * 0.5 + dy,
                            );
                            let p1 = (rotation * Point3::new(p1.0, p1.1, p1.2) * text_scaling) + translation_after_rotation;
                            let p2 = (rotation * Point3::new(p2.0, p2.1, p2.2) * text_scaling) + translation_after_rotation;
                            let p3 = (rotation * Point3::new(p3.0, p3.1, p3.2) * text_scaling) + translation_after_rotation;
                            mesh.add_face(
                                Point3D::new(p1.x as f64, p1.y as f64, p1.z as f64),
                                Point3D::new(p2.x as f64, p2.y as f64, p2.z as f64),
                                Point3D::new(p3.x as f64, p3.y as f64, p3.z as f64),
                            );
                        }
                        _ => {},
                    }
                }
            }
        }
    }

    let mut buffer = File::create(output_path.with_extension("ply").as_path())?;
    io::save_ply_binary(&mut buffer, &mesh, &Precision::P64).map_err(|e| anyhow!("{:?}", e))?;
    let mut buffer = File::create(output_path.with_extension("stl").as_path())?;
    io::save_stl_ascii(&mut buffer, &mesh).map_err(|e| anyhow!("{:?}", e))?;
    Ok(())
}
