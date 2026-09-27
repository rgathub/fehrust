use super::*;

#[test]
fn orientation_1() {
    assert_eq!(exif_orientation_to_rotation(1), (0.0, false, false));
}

#[test]
fn orientation_2() {
    assert_eq!(exif_orientation_to_rotation(2), (0.0, true, false));
}

#[test]
fn orientation_3() {
    assert_eq!(exif_orientation_to_rotation(3), (180.0, false, false));
}

#[test]
fn orientation_4() {
    assert_eq!(exif_orientation_to_rotation(4), (0.0, false, true));
}

#[test]
fn orientation_5() {
    assert_eq!(exif_orientation_to_rotation(5), (90.0, true, false));
}

#[test]
fn orientation_6() {
    assert_eq!(exif_orientation_to_rotation(6), (90.0, false, false));
}

#[test]
fn orientation_7() {
    assert_eq!(exif_orientation_to_rotation(7), (270.0, true, false));
}

#[test]
fn orientation_8() {
    assert_eq!(exif_orientation_to_rotation(8), (270.0, false, false));
}

#[test]
fn orientation_unknown() {
    assert_eq!(exif_orientation_to_rotation(99), (0.0, false, false));
}

fn make_exif_info(
    camera_make: Option<&str>,
    camera_model: Option<&str>,
    date_time: Option<&str>,
    exposure: Option<&str>,
    f_number: Option<&str>,
    iso: Option<&str>,
    focal_length: Option<&str>,
    dimensions: Option<(u32, u32)>,
    gps: Option<(f64, f64)>,
) -> ExifInfo {
    ExifInfo {
        orientation: 1,
        camera_make: camera_make.map(String::from),
        camera_model: camera_model.map(String::from),
        date_time: date_time.map(String::from),
        exposure: exposure.map(String::from),
        f_number: f_number.map(String::from),
        iso: iso.map(String::from),
        focal_length: focal_length.map(String::from),
        dimensions,
        gps,
    }
}

#[test]
fn format_summary_all_fields() {
    let info = make_exif_info(
        Some("Canon"),
        Some("EOS R5"),
        Some("2024:01:15 10:30:00"),
        Some("1/250"),
        Some("f/2.8"),
        Some("400"),
        Some("50 mm"),
        Some((6000, 4000)),
        Some((40.7128, -74.0060)),
    );
    let summary = info.format_summary();
    assert!(summary.contains("Camera: Canon"));
    assert!(summary.contains("Model: EOS R5"));
    assert!(summary.contains("Date:"));
    assert!(summary.contains("Exposure: 1/250"));
    assert!(summary.contains("F-Number: f/2.8"));
    assert!(summary.contains("ISO: 400"));
    assert!(summary.contains("Focal Length: 50 mm"));
    assert!(summary.contains("6000x4000"));
    assert!(summary.contains("GPS:"));
}

#[test]
fn format_summary_none_fields() {
    let info = make_exif_info(None, None, None, None, None, None, None, None, None);
    let summary = info.format_summary();
    assert!(summary.is_empty());
}

#[test]
fn format_summary_partial() {
    let info = make_exif_info(
        Some("Nikon"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    let summary = info.format_summary();
    assert!(summary.contains("Camera: Nikon"));
    assert!(!summary.contains("Model:"));
}

#[test]
fn format_summary_gps() {
    let info = make_exif_info(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some((48.8566, 2.3522)),
    );
    let summary = info.format_summary();
    assert!(summary.contains("GPS:"));
    assert!(summary.contains("48.8566"));
}
