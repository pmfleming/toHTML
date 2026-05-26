use super::super::text_repair::repair_visual_text;
use super::super::*;

#[test]
fn repairs_iec_figure_and_page_markers() {
    assert_eq!(
        repair_visual_text("IG SEPA Credit Transferversion 7.0"),
        "IG SEPA Credit Transfer version 7.0"
    );
    assert_eq!(
        repair_visual_text("Figure 1ŒFlowchart for determining conformity"),
        "Figure 1 – Flowchart for determining conformity"
    );
    assert_eq!(
        repair_visual_text("Figure 1 ŒFlowchart for determining conformity"),
        "Figure 1 – Flowchart for determining conformity"
    );
    assert_eq!(repair_visual_text("1 Œ"), "1 –");
    assert_eq!(
        repair_visual_text("Figure2ŒIllustration of the relative phase angle"),
        "Figure 2 – Illustration of the relative phase angle"
    );
    assert_eq!(
        repair_visual_text("IEC 61000-3-2:2018 © IEC 2018 Œ19Œ"),
        "IEC 61000-3-2:2018 © IEC 2018 – 19 –"
    );
    assert_eq!(repair_visual_text("Starthere:"), "Start here:");
    assert_eq!(
        repair_visual_text("_KNO Conditions d™essai des climatiseurs"),
        "B.12 Conditions d'essai des climatiseurs"
    );
    assert_eq!(
        repair_visual_text("AnnexeA Circuit de mesure et s ource d'alimentation"),
        "Annexe A Circuit de mesure et source d'alimentation"
    );
    assert_eq!(
        repair_visual_text("Figure A.1 –Circuit de mesure pour les appareils monophasés"),
        "Figure A.1 – Circuit de mesure pour les appareils monophasés"
    );
    assert_eq!(
        repair_visual_text("Tableau 1ŒLimites pour les appareils de classe A"),
        "Tableau 1 – Limites pour les appareils de classe A"
    );
}

#[test]
fn repairs_hce_legal_spacing_for_visual_lines() {
    assert_eq!(
        repair_visual_text(
            "+ HUBBELL 6SCOTLAND having offices at HillingtonRoad Glasgow *%/ 6FRWOD QG 8. and theiU representatives"
        ),
        "HUBBELL SCOTLAND having offices at Hillington Road, Glasgow, G52 4BL, Scotland, UK, and their representatives"
    );
    assert_eq!(
        repair_visual_text(
            "AsusedinthisMutualConfidentialityAgreementµDisclosin JParty´referstoeitherHubbell or Inventronicsasthe"
        ),
        "As used in this Mutual Confidentiality Agreement, ‘Disclosing Party’ refers to either Hubbell or Inventronics as the"
    );
    assert_eq!(
        repair_visual_text("IfoeceiYLQJPartydecidesnottoproceedwiththeTransacti RQReceivingPartywillpromptlynotifyDisclosingParty"),
        "If Receiving Party decides not to proceed with the Transaction Receiving Party will promptly notify Disclosing Party"
    );
    assert_eq!(
        repair_visual_text("ThisAgreementconstitutestheentireagreementbetweenthe partieswithrespecttothesubjectmatterhereofThis"),
        "This Agreement constitutes the entire agreement between the parties with respect to the subject matter hereof. This"
    );
}

#[test]
fn preserves_fragment_rotation() {
    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(200.0),
        height: Some(300.0),
        segments: vec![
            TextSegment::new("Sideways".to_string(), 20.0, 240.0, 12.0, 60.0).with_rotation(90.0),
        ],
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains("transform:rotate(90.00deg)"));
}

#[test]
fn avoids_expanding_small_diagram_labels() {
    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(200.0),
        height: Some(300.0),
        segments: vec![TextSegment::new(
            "Technical FAE".to_string(),
            20.0,
            240.0,
            8.0,
            90.0,
        )],
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(!html.contains("scaleX("));
}

#[test]
fn renders_shapes_before_text() {
    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(200.0),
        height: Some(300.0),
        segments: vec![TextSegment::new(
            "Cell".to_string(),
            20.0,
            240.0,
            12.0,
            24.0,
        )],
        shapes: vec![RectShape {
            x: 10.0,
            y: 220.0,
            width: 100.0,
            height: 30.0,
            fill: Some("#eeeeee".to_string()),
            stroke: Some("#000000".to_string()),
        }],
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains("class=\"pdf-shape\""));
    assert!(html.find("pdf-shape") < html.find("pdf-text-fragment"));
    assert!(html.contains("background:#eeeeee"));
    assert!(html.contains("border:0.75pt solid #000000"));
}

#[test]
fn renders_images_before_shapes_and_text() {
    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(200.0),
        height: Some(300.0),
        segments: vec![TextSegment::new(
            "Caption".to_string(),
            20.0,
            240.0,
            12.0,
            42.0,
        )],
        shapes: Vec::new(),
        images: vec![VisualImage {
            src: "data:image/jpeg;base64,YWJjZA==".to_string(),
            mask_src: None,
            alt: "PDF image".to_string(),
            x: 10.0,
            y: 20.0,
            width: 50.0,
            height: 40.0,
        }],
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains("class=\"pdf-image\""));
    assert!(html.contains("src=\"data:image/jpeg;base64,YWJjZA==\""));
    assert!(html.contains("left:10.00pt;top:240.00pt;width:50.00pt;height:40.00pt"));
    assert!(html.find("pdf-image") < html.find("pdf-text-fragment"));
}

#[test]
fn renders_page_background_before_embedded_diagram_images() {
    let html = render_pages(&[VisualPage {
        page_number: 2,
        width: Some(200.0),
        height: Some(300.0),
        segments: vec![TextSegment::new(
            "Diagram label".to_string(),
            20.0,
            220.0,
            12.0,
            72.0,
        )],
        shapes: vec![
            RectShape {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 300.0,
                fill: Some("#ffffff".to_string()),
                stroke: None,
            },
            RectShape {
                x: 10.0,
                y: 220.0,
                width: 100.0,
                height: 2.0,
                fill: Some("#000000".to_string()),
                stroke: None,
            },
        ],
        images: vec![VisualImage {
            src: "data:image/png;base64,Ym94".to_string(),
            mask_src: None,
            alt: "PDF image".to_string(),
            x: 15.0,
            y: 195.0,
            width: 80.0,
            height: 44.0,
        }],
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.find("pdf-shape").unwrap() < html.find("pdf-image").unwrap());
    assert!(html.find("pdf-image").unwrap() < html.find("Diagram label").unwrap());
}
