use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ExifInfo {
    pub orientation: u16,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub date_time: Option<String>,
    pub exposure: Option<String>,
    pub f_number: Option<String>,
    pub iso: Option<String>,
    pub focal_length: Option<String>,
    pub dimensions: Option<(u32, u32)>,
    pub gps: Option<(f64, f64)>,
}

pub fn read_exif(path: &Path) -> Option<ExifInfo> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let exif_data = exif::Reader::new()
        .read_from_container(&mut reader)
        .ok()?;

    let orientation = exif_data
        .get_field(exif::Tag::Orientation, exif::In::PRIMARY)
        .and_then(|f| f.value.get_uint(0))
        .map(|v| v as u16)
        .unwrap_or(1);

    let camera_make = get_string(&exif_data, exif::Tag::Make);
    let camera_model = get_string(&exif_data, exif::Tag::Model);
    let date_time = get_string(&exif_data, exif::Tag::DateTime);
    let exposure = get_string(&exif_data, exif::Tag::ExposureTime);
    let f_number = get_string(&exif_data, exif::Tag::FNumber);
    let iso = exif_data
        .get_field(exif::Tag::PhotographicSensitivity, exif::In::PRIMARY)
        .map(|f| f.display_value().to_string());
    let focal_length = get_string(&exif_data, exif::Tag::FocalLength);

    let width = exif_data
        .get_field(exif::Tag::PixelXDimension, exif::In::PRIMARY)
        .and_then(|f| f.value.get_uint(0));
    let height = exif_data
        .get_field(exif::Tag::PixelYDimension, exif::In::PRIMARY)
        .and_then(|f| f.value.get_uint(0));
    let dimensions = match (width, height) {
        (Some(w), Some(h)) => Some((w, h)),
        _ => None,
    };

    let gps = read_gps(&exif_data);

    Some(ExifInfo {
        orientation,
        camera_make,
        camera_model,
        date_time,
        exposure,
        f_number,
        iso,
        focal_length,
        dimensions,
        gps,
    })
}

fn get_string(exif_data: &exif::Exif, tag: exif::Tag) -> Option<String> {
    exif_data
        .get_field(tag, exif::In::PRIMARY)
        .map(|f| f.display_value().to_string())
}

fn read_gps(exif_data: &exif::Exif) -> Option<(f64, f64)> {
    let lat = parse_gps_coord(
        exif_data,
        exif::Tag::GPSLatitude,
        exif::Tag::GPSLatitudeRef,
    )?;
    let lon = parse_gps_coord(
        exif_data,
        exif::Tag::GPSLongitude,
        exif::Tag::GPSLongitudeRef,
    )?;
    Some((lat, lon))
}

fn parse_gps_coord(
    exif_data: &exif::Exif,
    coord_tag: exif::Tag,
    ref_tag: exif::Tag,
) -> Option<f64> {
    let field = exif_data.get_field(coord_tag, exif::In::PRIMARY)?;
    let ref_field = exif_data.get_field(ref_tag, exif::In::PRIMARY)?;

    match &field.value {
        exif::Value::Rational(rationals) if rationals.len() >= 3 => {
            let degrees = rationals[0].to_f64();
            let minutes = rationals[1].to_f64();
            let seconds = rationals[2].to_f64();
            let mut coord = degrees + minutes / 60.0 + seconds / 3600.0;

            let ref_str = ref_field.display_value().to_string();
            if ref_str.contains('S') || ref_str.contains('W') {
                coord = -coord;
            }
            Some(coord)
        }
        _ => None,
    }
}

/// Convert EXIF orientation value to (rotation_degrees, flip_h, flip_v)
pub fn exif_orientation_to_rotation(orientation: u16) -> (f64, bool, bool) {
    match orientation {
        1 => (0.0, false, false),   // Normal
        2 => (0.0, true, false),    // Flipped horizontally
        3 => (180.0, false, false), // Rotated 180°
        4 => (0.0, false, true),    // Flipped vertically
        5 => (90.0, true, false),   // Transposed (flip H + rotate 90 CW)
        6 => (90.0, false, false),  // Rotated 90° CW
        7 => (270.0, true, false),  // Transversed (flip H + rotate 270 CW)
        8 => (270.0, false, false), // Rotated 270° CW
        _ => (0.0, false, false),
    }
}

impl ExifInfo {
    pub fn format_summary(&self) -> String {
        let mut lines = Vec::new();

        if let Some(ref make) = self.camera_make {
            lines.push(format!("Camera: {}", make.trim_matches('"')));
        }
        if let Some(ref model) = self.camera_model {
            lines.push(format!("Model: {}", model.trim_matches('"')));
        }
        if let Some(ref dt) = self.date_time {
            lines.push(format!("Date: {}", dt.trim_matches('"')));
        }
        if let Some(ref exp) = self.exposure {
            lines.push(format!("Exposure: {}", exp));
        }
        if let Some(ref f) = self.f_number {
            lines.push(format!("F-Number: {}", f));
        }
        if let Some(ref iso) = self.iso {
            lines.push(format!("ISO: {}", iso));
        }
        if let Some(ref fl) = self.focal_length {
            lines.push(format!("Focal Length: {}", fl));
        }
        if let Some((w, h)) = self.dimensions {
            lines.push(format!("EXIF Dimensions: {}x{}", w, h));
        }
        if let Some((lat, lon)) = self.gps {
            lines.push(format!("GPS: {:.6}, {:.6}", lat, lon));
        }

        lines.join("\n")
    }
}
