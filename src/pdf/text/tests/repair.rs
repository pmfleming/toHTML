use super::super::*;

#[test]
fn repairs_hce_shifted_subset_words_after_cmap_decoding() {
    assert_eq!(
        super::super::strings::repair_shifted_subset_words("DQGWHFKQLTXHDVLWUHODWHVWR"),
        "andtechniqueasitrelatesto"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "5RDG %LQ-LDQJ 'LVW +DQJ]KRX &KLQD DQG their UHSUHVHQ"
        ),
        "Road BinJiang Dist Hangzhou China and their represen"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words("0878$/&21),DENTI$/,7<"),
        "MUTUALCONFIDENTIALITY"
    );
    assert_eq!(
        super::super::strings::decode_pdf_text_string(b"0878$/&21),DENTI$/,7<"),
        "MUTUALCONFIDENTIALITY"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "having RIILFHV DW Hillington UHIHUUHGWRDV³"
        ),
        "having offices at Hillington referredtoas³"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "´DUHLQWHUHVWHGLQDSRVVLEOHEXVLQHVVUHODWLRQVKLSUHJDUGLQJ"
        ),
        "´areinterestedinapossiblebusinessrelationshipregarding"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words("19(17521,&6 $1*=+28 1&"),
        "INVENTRONICS HANGZHOU INC"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "AVXVHGLQWKLVMXWXDOCRQILGHQWLDOLWyAJUHHPHQWµ'LVFORVLQ"
        ),
        "AsusedinthisMutualConfidentialityAgreement,'Disclosin"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "AVXVHGLQWKLVMXWXDOCRQILGHQWLDOLWyAJUHHPHQWµDisclosin"
        ),
        "AsusedinthisMutualConfidentialityAgreement,Disclosin"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words("SDUWLHVPD\\H[FKDQJHFHUWDLQF"),
        "partiesmayexchangecertainc"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            r"WKH XQDXWKRUL]HG GLVFORVXUH E\ Receiving Party"
        ),
        "the unauthorized disclosure by Receiving Party"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "UHTXLVLWLRQV SURFHVV LQIRUPDWLRQ LQVWUXFWLRQV WHVW UHVXOWV"
        ),
        "requisitions process information instructions test results"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "EZDV NQRZQ WR ReceiviQJ Party DV HYLGHQFHG E\\ ZULWWHQ UHFRUGV EHIRUH UHFHLSW thereof IURP"
        ),
        "was known to Receiving Party as evidenced by written records before receipt thereof from"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "7+(5()25( LQ FRQVLGHUDWLRQ RI the PXWXDO FRYHQDQW and DJUHHPHQWV FRQWDLQHG herein and DQ\\ Transactions"
        ),
        "THEREFORE in consideration of the mutual covenant and agreements contained herein and any Transactions"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "FRQILGHQWLDO QDWXUH RI VXFK information IXUQLVKHG by the DisclosinJ Party RU LWV directors RIILFHUV HPSOR\\HHV"
        ),
        "confidential nature of such information furnished by the Disclosing Party or its directors officers employees"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "LQFOXGLQJ EXW QRW OLPLWHG WR ELG documents GUDZLQJV VSHFLILFDWLRQV"
        ),
        "including but not limited to bid documents drawings specifications"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "12 27+(5 :$55$17,(6 $5( 0$'( %< (,7+(5 3$57< 81'(5 7+,6"
        ),
        "NO OTHER WARRANTIES ARE MADE BY EITHER PARTY UNDER THIS"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "$*5((0(17$1<,1)250$ 7,21(;&+$1*('81'(5 7+,6$*5((0(17,63"
        ),
        "AGREEMENTANYINFORMA TION EXCHANGED UNDER THISAGREEMENTISP"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "AGREEMENT$1<,1)250$ TIONEXCHANGEDUNDER THISAGREEMENT,63"
        ),
        "AGREEMENT. ANY INFORMA TION EXCHANGED UNDER THIS AGREEMENT IS P"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "DFFRXQWDnts DJHnts GRFXPHnts UHODWLQJ GHVWUR\\HG DYDLODEOH UHPDLQ WKLV agreemeQW"
        ),
        "accountants agents documents relating destroyed available remain this agreement"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "PartySURPSWO\\XSRQUHTXHVW GHSHQGHQWO\\ GHYHORSHG Zithout QVWUXHGLQDFFRUGDQFHwiththe HDFK VHW IRUWK VXUYLYH FRQFOXVLRQ EHWZHHQ RZQ H[SHQVH FRSLHV"
        ),
        "Partypromptlyuponrequest independently developed without construedinaccordancewiththe each set forth survive conclusion between own expense copies"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "eitherPartyLVUHFHLYLQJL HFHLYLQJParty PHDQVRIDGHSRVLWLRQVXESRHQD SHUPLWWHG under applicable ODZ shall FRRSHUDWH HIIRUWV SUHYHQW PartyLQZULWLQJRUE"
        ),
        "eitherPartyisreceivingi ReceivingParty meansofadepositionsubpoena permitted under applicable law shall cooperate efforts prevent Partyinwritingorby"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "HUBBELL &27/$1' ILQ DQFLDO DGYLVRUV lawVUHJXODWLR EOHlaws breachHGbyRecei YLQJParty expenseV own H [SHQVH 5eceiving LYLQJParty"
        ),
        "HUBBELL SCOTLAND financial advisors lawsregulatio blelaws breachedbyReceivingParty expenses own expense Receiving ivingParty"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "AGREEME17 hePaUWLHVKHUHbyagreetothefollowing rkedasconfidentialwithLQ DQFLDO DGYLVRUV GDWD PDQXDOV PDFKLQHV VDPSOHV PDGH deliverHG DZKRQHHGW KHWHUPVRI UHVWULFWL DWKLUG HEHQHIL SURY RIZKL FRS\\VHQWbyHPDLOandsXFKI"
        ),
        "AGREEMENT thePartiesherebyagreetothefollowing markedasconfidentialwithin ancial advisors data manuals machines samples made delivered whoneedt thetermsof restricti athird benefi prov ofwhi copysentbyemailandsuchf"
    );
}

#[test]
fn repairs_iec_toc_shifted_subset_symbols_without_touching_plain_no() {
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "7.4.3 5DWHGSRZHUŁ W and ﬂ 25W..................................................................... 20",
        ),
        "7.4.3 Rated power ≥ 5 W and ≤ 25 W..................................................................... 20",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "5.1 General ................................................................................................................. NO",
        ),
        "5.1 General ................................................................................................................. 12",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "_KNO Conditions d™essai des climatiseurs ....................................................................... 70",
        ),
        "B.12 Conditions d'essai des climatiseurs ....................................................................... 70",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "AnnexeA Circuit de mesure et s ource d'alimentation"
        ),
        "Annexe A Circuit de mesure et source d'alimentation",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "Figure A.1 –Circuit de mesure pour les appareils monophasés",
        ),
        "Figure A.1 – Circuit de mesure pour les appareils monophasés",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "Tableau 1ŒLimites pour les appareils de classe A",
        ),
        "Tableau 1 – Limites pour les appareils de classe A",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words("NO OTHER WARRANTIES"),
        "NO OTHER WARRANTIES",
    );
}

#[test]
fn repairs_iec_prose_spacing_without_touching_plain_codes() {
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "International Standard IEC 6 1000 -3- 2 has been prepared by sub -committee 77A: EMCŒLow frequency phenomena, of IEC technical committee 77: Electromagnetic com patibility.",
        ),
        "International Standard IEC 61000-3-2 has been prepared by sub-committee 77A: EMC – Low frequency phenomena, of IEC technical committee 77: Electromagnetic compatibility.",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "b) the addition of a threshold of 5 W under wh ich no emission limits apply to all lighting",
        ),
        "b) the addition of a threshold of 5 W under which no emission limits apply to all lighting",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "lamps and other types of lighting equipment and with a rated power higher than 100 Wor 200 W",
        ),
        "lamps and other types of lighting equipment and with a rated power higher than 100 W or 200 W",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words("7.4.2 Rated power > 25W"),
        "7.4.2 Rated power > 25 W",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "a) anupdateoftheemissionlimitsforlightingequipmentwitharatedpowerﬂ W to take",
        ),
        "a) an update of the emission limits for lighting equipment with a rated power ≤ 25 W to take",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "Part 3-2: Limits ŒLimits for harmonic current emissions (equipmentinputcurrentﬂ 16 A per phase)",
        ),
        "Part 3-2: Limits – Limits for harmonic current emissions (equipment input current ≤ 16 A per phase)",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "equipmentwithaninputcurrentﬂ A to equipment with a rated input c XUUHQWﬂ A.",
        ),
        "equipment with an input current ≤ 16 A to equipment with a rated input current ≤ 16 A.",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words("powerﬂ W;"),
        "power ≤ 2 W;",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words("IEC 61000 -3-2:2018 © IEC 2018 Œ 5 Œ",),
        "IEC 61000-3-2:2018 © IEC 2018 – 5 –",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words("IEC 61000-3-2:2018 © IEC 2018 Œ19Œ",),
        "IEC 61000-3-2:2018 © IEC 2018 – 19 –",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "Figure1 ŒFlowchart for determining conformity"
        ),
        "Figure 1 – Flowchart for determining conformity",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words("1 Œ"),
        "1 –",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "NOTE Ip(abs) is the higher absolute value of Ip andIp-.",
        ),
        "NOTE Ip(abs) is the higher absolute value of Ip and Ip−.",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "IMPORTANT ŒThe 'colour inside' logo on the cover page",
        ),
        "IMPORTANT – The 'colour inside' logo on the cover page",
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words("x reconfirmed,"),
        "• reconfirmed,",
    );
}

#[test]
fn repairs_iec_prose_after_line_gap_joining() {
    let line = text_lines(&[
        TextSegment::new("IEC 6".to_string(), 10.0, 100.0, 10.0, 24.0),
        TextSegment::new("1000".to_string(), 38.0, 100.0, 10.0, 24.0),
        TextSegment::new("-3-".to_string(), 66.0, 100.0, 10.0, 18.0),
        TextSegment::new("2".to_string(), 88.0, 100.0, 10.0, 6.0),
        TextSegment::new(
            "has been prepared by sub".to_string(),
            105.0,
            100.0,
            10.0,
            120.0,
        ),
        TextSegment::new("-committee".to_string(), 230.0, 100.0, 10.0, 54.0),
    ]);

    assert_eq!(
        line[0].text,
        "IEC 61000-3-2 has been prepared by sub-committee"
    );
}

#[test]
fn keeps_plain_codes_and_parenthesized_words_after_cmap_decoding() {
    assert_eq!(
        super::super::strings::repair_shifted_subset_words("PRG-MUL2 2T21151D000412 TLB01"),
        "PRG-MUL2 2T21151D000412 TLB01"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words("(MAKE ONE BOLD)"),
        "(MAKE ONE BOLD)"
    );
}

#[test]
fn repairs_how_to_program_driver_heading_text() {
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "What Currents Can I set the ariver to without Changing the Total Power"
        ),
        "What Currents Can I set the Driver to without Changing the Total Power"
    );
}

#[test]
fn repairs_dreu_custom_encoded_quote_values() {
    assert_eq!(
        super::super::strings::repair_shifted_subset_words("OMOOJMQJNO"),
        "2022-04-12"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words(
            "VAT: kiURSNOOVUN_MN OMOOJMQJNO aobrOMONMPMPfkMN"
        ),
        "VAT: NL856122981B01 2022-04-12 DREU20210303IN01"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words("rpANVKNM rpANUKQR rpAOOKPN rpAONKRR"),
        "US$19.10 US$18.45 US$22.31 US$21.55"
    );
    assert_eq!(
        super::super::strings::repair_shifted_subset_words("IBAN: kiPN RABO 0311 1323 32"),
        "IBAN: NL31 RABO 0311 1323 32"
    );
}

#[test]
fn repairs_hce_shifted_subset_visual_segments() {
    let stream = b"BT (AVXVHGLQWKLVMXWXDOCRQILGHQWLDOLWyAJUHHPHQW\xb5Disclosin) Tj ET";

    assert_eq!(
        extract_text(stream).as_deref(),
        Some("AsusedinthisMutualConfidentialityAgreement,Disclosin")
    );
}

#[test]
fn decodes_concatenated_shifted_subset_text() {
    let stream = b"BT (7KLV$JUHHPHQWVKDOOEHFRQILGHQWLDO) Tj ET";

    assert_eq!(
        extract_text(stream).as_deref(),
        Some("ThisAgreementshallbeconfidential")
    );
}
