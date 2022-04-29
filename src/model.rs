use crate::CommitCount;
use anyhow::anyhow;
use rust_3d::{io, IsFaceEditableMesh, Mesh3D, Point3D, PointCloud3D, Precision};
use std::fs::File;
use std::path::Path;

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

pub fn build_trophy(heightmap: &[CommitCount], output_path: &Path) -> anyhow::Result<()> {
    let mut mesh: Mesh3D<Point3D, PointCloud3D<Point3D>, Vec<usize>> = Mesh3D::default();
    let top_length = 52.0;
    let top_width = 7.0;
    let plinth_height = 2.0;
    let top_margin = 1.0;
    let bottom_margin = 3.0;
    let maximum_commit_height = 10.0;
    let plinth_faces = plinth(
        &Point3D::new(0.0, 0.0, 0.0),
        top_length + top_margin,
        top_width + top_margin,
        top_length + bottom_margin,
        top_width + bottom_margin,
        plinth_height,
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
                commits as f64 * maximum_commit_height / max_commits as f64;
            let brick_center = Point3D::new(
                x as f64 - top_length * 0.5 + 0.5,
                y as f64 - top_width * 0.5 + 0.5,
                brick_height_normalized as f64 * 0.5 + plinth_height * 0.5,
            );
            let brick = commit_brick(&brick_center, 1.0, 1.0, brick_height_normalized as f64);
            brick_faces.extend_from_slice(brick.as_slice());
        }
    }
    for (p0, p1, p2) in plinth_faces.iter().chain(brick_faces.iter()) {
        mesh.add_face(p0.clone(), p1.clone(), p2.clone());
    }

    let mut buffer = File::create(output_path.with_extension("ply").as_path())?;
    io::save_ply_binary(&mut buffer, &mesh, &Precision::P64).map_err(|e| anyhow!("{:?}", e))?;
    let mut buffer = File::create(output_path.with_extension("stl").as_path())?;
    io::save_stl_ascii(&mut buffer, &mesh).map_err(|e| anyhow!("{:?}", e))?;
    Ok(())
}
