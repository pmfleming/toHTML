use super::super::object::PdfValue;
use super::encoding::base64;
use super::encoding::{png_alpha_from_gray_mask, png_from_raw_image};
use super::placement::image_placements;
use super::*;

#[test]
fn encodes_base64_padding() {
    assert_eq!(base64(b"abcd"), "YWJjZA==");
}

#[test]
fn wraps_rgb_pixels_as_png() {
    let mut dictionary = PdfDictionary::new();
    dictionary.insert("Width".to_string(), PdfValue::Integer(1));
    dictionary.insert("Height".to_string(), PdfValue::Integer(1));
    dictionary.insert("BitsPerComponent".to_string(), PdfValue::Integer(8));
    dictionary.insert(
        "ColorSpace".to_string(),
        PdfValue::Name("DeviceRGB".to_string()),
    );

    let png = png_from_raw_image(&dictionary, &[0xff, 0x00, 0x00]).unwrap();

    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert!(png.windows(4).any(|window| window == b"IHDR"));
    assert!(png.windows(4).any(|window| window == b"IDAT"));
    assert!(png.windows(4).any(|window| window == b"IEND"));
}

#[test]
fn infers_rgb_color_from_decode_parameters() {
    let mut dictionary = PdfDictionary::new();
    dictionary.insert("Width".to_string(), PdfValue::Integer(1));
    dictionary.insert("Height".to_string(), PdfValue::Integer(1));
    dictionary.insert("BitsPerComponent".to_string(), PdfValue::Integer(8));
    dictionary.insert(
        "ColorSpace".to_string(),
        PdfValue::Reference(PdfReference {
            object: 54,
            generation: 0,
        }),
    );
    dictionary.insert(
        "DecodeParms".to_string(),
        PdfValue::Dictionary(PdfDictionary::from([(
            "Colors".to_string(),
            PdfValue::Integer(3),
        )])),
    );

    let png = png_from_raw_image(&dictionary, &[0x00, 0x80, 0xff]).unwrap();

    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert_eq!(png[25], 2);
}

#[test]
fn wraps_soft_masks_as_alpha_pngs() {
    let mut dictionary = PdfDictionary::new();
    dictionary.insert("Width".to_string(), PdfValue::Integer(2));
    dictionary.insert("Height".to_string(), PdfValue::Integer(1));
    dictionary.insert("BitsPerComponent".to_string(), PdfValue::Integer(8));
    dictionary.insert(
        "ColorSpace".to_string(),
        PdfValue::Name("DeviceGray".to_string()),
    );

    let png = png_alpha_from_gray_mask(&dictionary, &[0x00, 0xff]).unwrap();

    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert_eq!(png[25], 6);
}

#[test]
fn finds_images_inside_nested_form_xobjects() {
    let source = br#"
1 0 obj
<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length 3 >>
stream
abc
endstream
endobj
2 0 obj
<< /Type /XObject /Subtype /Form /Matrix [100 0 0 40 300 80] /Resources << /XObject << /Im0 1 0 R >> >> /Length 8 >>
stream
/Im0 Do
endstream
endobj
3 0 obj
<< /Type /XObject /Subtype /Form /Resources << /XObject << /Fm1 2 0 R >> >> /Length 8 >>
stream
/Fm1 Do
endstream
endobj
"#;
    let objects = PdfObjects::parse(source);
    let resources = HashMap::from([(
        "Fm0".to_string(),
        PdfReference {
            object: 3,
            generation: 0,
        },
    )]);
    let mut warnings = Vec::new();

    let placements = image_placements(
        source,
        &objects,
        &[b"/Fm0 Do".to_vec()],
        &resources,
        &mut warnings,
    );

    assert!(warnings.is_empty());
    assert_eq!(placements.len(), 1);
    assert_eq!(placements[0].reference.object, 1);
    assert!((placements[0].x - 300.0).abs() < 0.1);
    assert!((placements[0].y - 80.0).abs() < 0.1);
    assert!((placements[0].width - 100.0).abs() < 0.1);
    assert!((placements[0].height - 40.0).abs() < 0.1);
}

#[test]
fn skips_images_fully_clipped_by_rectangular_clip_path() {
    let source = br#"
1 0 obj
<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length 3 >>
stream
abc
endstream
endobj
2 0 obj
<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length 3 >>
stream
def
endstream
endobj
"#;
    let objects = PdfObjects::parse(source);
    let resources = HashMap::from([
        (
            "Im0".to_string(),
            PdfReference {
                object: 1,
                generation: 0,
            },
        ),
        (
            "Im1".to_string(),
            PdfReference {
                object: 2,
                generation: 0,
            },
        ),
    ]);
    let mut warnings = Vec::new();
    let stream = br#"
q
10 100 50 50 re
W
n
q
50 0 0 50 10 100 cm
/Im0 Do
Q
Q
q
10 100 50 50 re
W
n
q
10 50 50 50 re
W
n
q
50 0 0 50 10 50 cm
/Im1 Do
Q
Q
Q
"#;

    let placements = image_placements(
        source,
        &objects,
        &[stream.to_vec()],
        &resources,
        &mut warnings,
    );

    assert!(warnings.is_empty());
    assert_eq!(placements.len(), 1);
    assert_eq!(placements[0].reference.object, 1);
    assert!((placements[0].x - 10.0).abs() < 0.1);
    assert!((placements[0].y - 100.0).abs() < 0.1);
}
