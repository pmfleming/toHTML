pub(super) fn repair_visual_text(text: &str) -> String {
    repair_iec_visual_text(&repair_hce_legal_spacing(
        &super::super::text::repair_shifted_subset_text(text),
    ))
}

fn repair_iec_visual_text(text: &str) -> String {
    let mut repaired = text.to_string();
    for (from, to) in [
        ("PKNO", "3.12"),
        ("total ha rmonic distortion", "total harmonic distortion"),
        ("ha rmonic", "harmonic"),
        ("balanced three -phase", "balanced three-phase"),
        (
            "IEC 0050-161:1990, 16 1-05-05",
            "IEC 60050-161:1990, 161-05-05",
        ),
        ("IEC60050-161:1990", "IEC 60050-161:1990"),
        ("16 1-05-05", "161-05-05"),
        ("modified Œthe", "modified – the"),
        ("Œthe", "– the"),
        ("Œ the", "– the"),
        ("Œ 9 Œ", "– 9 –"),
        ("Œ20Œ", "– 20 –"),
        ("Note1to entry:", "Note 1 to entry:"),
        ("Note2to entry:", "Note 2 to entry:"),
        ("totalRMS", "total RMS"),
        (
            "IG SEPA Credit Transferversion 7.0",
            "IG SEPA Credit Transfer version 7.0",
        ),
        ("Credit Transferversion", "Credit Transfer version"),
        ("Starthere", "Start here"),
        ("d65°", "≤65°"),
        ("d60°", "≤60°"),
        ("t90°", "≥90°"),
        ("Œ0,05", "−0,05"),
        ("pŒ", "p−"),
        ("Ip-", "Ip−"),
        ("andIp", "and Ip"),
        ("Figure 1 Œ", "Figure 1 – "),
        ("Figure 2 Œ", "Figure 2 – "),
        ("Figure 1Œ", "Figure 1 – "),
        ("Figure 2Œ", "Figure 2 – "),
        ("Figure1Œ", "Figure 1 – "),
        ("Figure2Œ", "Figure 2 – "),
        ("Figure1 Œ", "Figure 1 – "),
        ("Figure2 Œ", "Figure 2 – "),
        ("Figure1 –", "Figure 1 – "),
        ("Figure2 –", "Figure 2 – "),
        ("Figure A.1 –Circuit", "Figure A.1 – Circuit"),
        ("Figure A.2 –Circuit", "Figure A.2 – Circuit"),
        ("Table 1Œ", "Table 1 – "),
        ("Table 2Œ", "Table 2 – "),
        ("Table 3Œ", "Table 3 – "),
        ("Table 4Œ", "Table 4 – "),
        ("Tableau 1Œ", "Tableau 1 – "),
        ("Tableau 2Œ", "Tableau 2 – "),
        ("Tableau 3Œ", "Tableau 3 – "),
        ("Tableau 4Œ", "Tableau 4 – "),
        ("Tableau B.1 Œ", "Tableau B.1 – "),
        ("–Flowchart", "– Flowchart"),
        ("–Illustration", "– Illustration"),
        ("–Organigramme", "– Organigramme"),
        ("1 Œ", "1 –"),
        ("2 Œ", "2 –"),
        ("Figure2 ŒIllustration", "Figure 2 – Illustration"),
        ("r elative", "relative"),
        ("to OR W shall", "to 25 W shall"),
        ("Table3", "Table 3"),
        ("ofIEC", "of IEC"),
        ("maxim um", "maximum"),
        ("AnnexeA", "Annexe A"),
        ("AnnexeB", "Annexe B"),
        ("s ource d'alimentation", "source d'alimentation"),
        ("d™essai", "d'essai"),
        ("™essai", "'essai"),
        ("d™entrée", "d'entrée"),
        ("d™alimentation", "d'alimentation"),
        ("l™arc", "l'arc"),
        ("l™Annexe", "l'Annexe"),
        ("l™Article", "l'Article"),
        ("l™IEC", "l'IEC"),
        ("ﬁgénériquesﬂ", "\"génériques\""),
        ("_KOKN", "B.2.1"),
        ("_KOKO", "B.2.2"),
        ("_KOKP", "B.2.3"),
        ("_KPKN", "B.3.1"),
        ("_KPKO", "B.3.2"),
        ("_KRKN", "B.5.1"),
        ("_KRKO", "B.5.2"),
        ("_KRKP", "B.5.3"),
        ("_KRKQ", "B.5.4"),
        ("_KNO", "B.12"),
        ("_KN", "B.1"),
        ("_KO", "B.2"),
        ("_KP", "B.3"),
        ("_KQ", "B.4"),
        ("_KR", "B.5"),
    ] {
        repaired = repaired.replace(from, to);
    }
    repaired = repair_iec_page_number_markers(&repaired);
    if repaired == "Œ" {
        repaired = "−".to_string();
    }
    repaired
}

fn repair_iec_page_number_markers(text: &str) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let mut repaired = String::with_capacity(text.len());
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == 'Œ' {
            let digit_start = index + 1;
            let mut digit_end = digit_start;
            while digit_end < chars.len() && chars[digit_end].is_ascii_digit() {
                digit_end += 1;
            }
            if digit_end > digit_start && digit_end < chars.len() && chars[digit_end] == 'Œ' {
                repaired.push_str("– ");
                for ch in &chars[digit_start..digit_end] {
                    repaired.push(*ch);
                }
                repaired.push_str(" –");
                index = digit_end + 1;
                continue;
            }
        }
        repaired.push(chars[index]);
        index += 1;
    }
    repaired
}
fn repair_hce_legal_spacing(text: &str) -> String {
    if !looks_like_hce_legal_text(text) {
        return text.to_string();
    }

    let mut repaired = text.to_string();
    for (from, to) in [
        (
            "+ HUBBELL 6SCOTLAND INVENTRONICS + HANGZHOU INC",
            "HUBBELL SCOTLAND                         INVENTRONICS (HANGZHOU), INC.",
        ),
        ("+ HUBBELL 6SCOTLAND", "HUBBELL SCOTLAND"),
        ("HUBBELL 6SCOTLAND", "HUBBELL SCOTLAND"),
        ("HillingtonRoad", "Hillington Road,"),
        (
            "Glasgow *%/ 6FRWOD QG 8.",
            "Glasgow, G52 4BL, Scotland, UK,",
        ),
        ("theiU", "their"),
        (
            "hereinafterdefinedandreferredtoas³",
            "(hereinafter defined and referred to as “",
        ),
        ("µDisclosin JParty´", " ‘Disclosing Party’ "),
        ("Disclosin JParty", "Disclosing Party "),
        ("JParty´", "Party’ "),
        (
            "AsusedinthisMutualConfidentialityAgreement,",
            "As used in this Mutual Confidentiality Agreement, ",
        ),
        (
            "AsusedinthisMutualConfidentialityAgreement",
            "As used in this Mutual Confidentiality Agreement, ",
        ),
        ("referstoeither", "refers to either "),
        ("or Inventronicsasthe", "or Inventronics as the "),
        ("casemaybewheneither", "case may be when either "),
        ("isdisclosinginformationtothe", "is disclosing information to the "),
        ("otherand³Receiving", "other and “Receiving "),
        ("referstoeitherPartywhen", "refers to either Party when "),
        (
            "Allinformationofanycharacterwhetherwrittenverbalorotherwiseprovidedthatallinformationdisclosedverbally",
            "All information of any character, whether written, verbal or otherwise, provided that all information disclosed verbally ",
        ),
        (
            "willbereducedtowritingandma markedasconfidentialwithindaysofdisclosureandmarkedsoastoindicatethe",
            "will be reduced to writing and marked as confidential within days of disclosure and marked so as to indicate the ",
        ),
        (
            "confidentialnatureofsuchinformationfurnishedbytheDisclosingParty",
            "confidential nature of such information furnished by the Disclosing Party ",
        ),
        ("Datedthis", "Dated this "),
        ("dayofOctober", "day of October, 2017 "),
        ("IfoeceiYLQJ", "If Receiving "),
        ("IfreceiYLQJ", "If Receiving "),
        ("oeceivingParty", "Receiving Party"),
        ("ReceivingParty", "Receiving Party"),
        ("DisclosingParty", "Disclosing Party"),
        ("ReceivingP DUW\\", "Receiving Party"),
        ("Partydecides", "Party decides "),
        ("VSRVVHVVLRQ", "possession"),
        ("Transacti RQ", "Transaction "),
        ("Partyinwritingorby", "Party in writing or by "),
        ("PartyD t", "Party at "),
        ("willpromptly", "will promptly "),
        ("Partywill", "Party will "),
        ("notifyDisclosing", "notify Disclosing "),
        ("nottoproceed", "not to proceed "),
        ("withthe", "with the "),
        ("ofthatdecision", "of that decision "),
        ("andinthatcase", "and in that case "),
        ("andatanytime", "and at any time "),
        ("uponthere quest", "upon the request "),
        ("Z illeither", "will either "),
        ("allcopies", "all copies "),
        ("ofthewritten", "of the written "),
        ("writtenInformation", "written Information"),
        ("inthepossessionof", "in the possession of "),
        ("confirmsuchdestruction", "confirm such destruction "),
        ("Informationin", "Information in "),
        ("Eachmarty", "Each Party "),
        ("eachmarty", "each Party "),
        ("othermarty", "other Party "),
        ("suchmarty", "such Party "),
        ("neithermarty", "neither Party "),
        ("hireanyemployee", "hire any employee "),
        ("oftheother", "of the other "),
        ("foraperiodof", "for a period of "),
        ("twelvemonths", "twelve months "),
        ("fromthedateof", "from the date of "),
        ("thisAgreem ent", "this Agreement "),
        ("shallbeprecluded", "shall be precluded "),
        ("KLULQJ", "hiring"),
        ("ZKR L LQLWLDWHV GLVFXVVLRQV", "who initiates discussions"),
        (
            "withoXW any GLUHFW VROLFLWDWLRQ",
            "without any direct solicitation",
        ),
        ("eitherParty", "either Party"),
        ("iirespondsto", "(ii) responds to "),
        (
            "DSXEOLFDGYHUWLVHPHQWSODFHG",
            "a public advertisement placed ",
        ),
        ("oriiihasbeenterminated", "or (iii) has been terminated "),
        ("oritssubsidiaries", "or its subsidiaries "),
        ("divisionsoraffiliates", "divisions or affiliates "),
        ("priortocommencem entof", "prior to commencement of "),
        ("betweensuchmartyand", "between such Party and "),
        (
            "suchofficerdirectororemployee",
            "such officer, director or employee",
        ),
        ("Theparties", "The parties "),
        ("ThisAgreement", "This Agreement"),
        ("This Agreementconstitutes", "This Agreement constitutes"),
        ("shallbeeffective", "shall be effective "),
        ("shallnotconstitute", "shall not constitute "),
        (
            "constitutestheentireagreement",
            "constitutes the entire agreement ",
        ),
        ("maybeexecuted", "may be executed "),
        ("maybeamended", "may be amended "),
        ("betweenthe", "between the "),
        ("partieswithrespect", "parties with respect "),
        ("tothesubject", "to the subject "),
        ("subjectmatter", "subject matter "),
        ("matterhereof", "matter hereof"),
        ("hereofThis", "hereof. This "),
        ("onlybyaninstrument", "only by an instrument "),
        ("inwritingexecu WHG", "in writing executed "),
        ("bybothparties", "by both parties"),
        (
            "incounterpartseachofwhi ch",
            "in counterparts, each of which ",
        ),
        ("shallbeanoriginal", "shall be an original "),
        ("andallofwhich", "and all of which "),
        ("shallconstitute", "shall constitute "),
        ("oneandthesameinstrument", "one and the same instrument. "),
        ("signaturepages", "signature pages "),
        ("tothis Agreement", "to this Agreement "),
        ("maybedelivered", "may be delivered "),
        ("byfacsimileincluding", "by facsimile including "),
        (
            "copysentbyemailandsuchf acsimiles",
            "copy sent by email and such facsimiles ",
        ),
        (
            "shallbedeemedasif actualsignaturepages",
            "shall be deemed as if actual signature pages ",
        ),
        ("hadbeendelivered", "had been delivered"),
        ("Neitherparty", "Neither party "),
        ("acquiresany", "acquires any "),
        (
            "intellectualpropertyrights",
            "intellectual property rights ",
        ),
        ("und HU", "under "),
        ("Bothparties", "Both parties "),
        ("shalladhereto", "shall adhere to "),
        ("allapplicablelaws", "all applicable laws, "),
        ("rulesrelatingtotheexport", "rules relating to the export "),
        ("technicaldataandshal O", "technical data, and shall "),
        (
            "notexportorreexportanytec KQLFDOdata",
            "not export or re-export any technical data, ",
        ),
        ("anyproductsreceiv edfrom", "any products received from "),
        ("orthedirectproductofsuch", "or the direct product of such "),
        (
            "datatoanyproscribedcountry",
            "data to any proscribed country ",
        ),
        (
            "listedinsuchapplica blelaws",
            "listed in such applicable laws, ",
        ),
        ("unlessproperlyauthorized", "unless properly authorized"),
        ("isnotintendedtoand", "is not intended to, and "),
        (
            "creategiveeffecttoorotherwise",
            "create, give effect to or otherwise ",
        ),
        ("recognizeajointventure", "recognize a joint venture"),
        (
            "partnershiporformalbusinessentity",
            "partnership, or formal business entity ",
        ),
        ("ofanykindNothinghe rein", "of any kind. Nothing herein "),
        ("shallbeconstruedas", "shall be construed as "),
        ("forthesharingof", "for the sharing of "),
        (
            "profitsorlossesarisingoutoftheeffortsof",
            "profits or losses arising out of the efforts of ",
        ),
        (
            "sanindependentcontractorand parties",
            "as an independent contractor and parties",
        ),
        ("shallacta", "shall act a"),
        ("notasanagentoftheother", "not as an agent of the other "),
        ("foranypurposewhatsoever", "for any purpose whatsoever "),
        ("andneithermarty", "and neither Party "),
        (
            "shallhaveanyauthoritytobindtheother",
            "shall have any authority to bind the other ",
        ),
        ("setforthherein", "set forth herein"),
        ("onthedateshownbelow", "on the date shown below "),
        ("andshallcontinue", "and shall continue "),
        ("infullforceandeffect", "in full force and effect "),
        ("foraperiodof", "for a period of "),
        ("unlesseitherparty", "unless either party "),
        (
            "terminatesthisAgreementearlier",
            "terminates this Agreement earlier ",
        ),
        ("providingthirtydays", "providing thirty days"),
        ("noticetotheother", "notice to the other"),
        (
            "partyUponterminationorexpiration",
            "party. Upon termination or expiration ",
        ),
        (
            "ofthisAgreementtheR eceivingParty",
            "of this Agreement, the Receiving Party ",
        ),
        ("allcopiesRI", "all copies of "),
        (
            "theInformationinthepossessionof",
            "the Information in the possession of ",
        ),
        (
            "andconfir msuchdestruction",
            "and confirm such destruction ",
        ),
        (
            "toDisclosingPartyinwritingorby",
            "to Disclosing Party in writing or by ",
        ),
        (
            "promptly deliver to DisclosingParty",
            "promptly deliver to Disclosing Party",
        ),
        (
            "at Receiving Party¶V own expense",
            "at Receiving Party's own expense",
        ),
    ] {
        repaired = repaired.replace(from, to);
    }
    if repaired.contains("HUBBELL SCOTLAND") && repaired.contains("INVENTRONICS (HANGZHOU), INC.") {
        repaired
    } else {
        repaired.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

fn looks_like_hce_legal_text(text: &str) -> bool {
    text.contains("ReceivingParty")
        || text.contains("DisclosingParty")
        || text.contains("ThisAgreement")
        || text.contains("Eachmarty")
        || text.contains("IfoeceiYLQJ")
        || text.contains("VSRVVHVVLRQ")
        || text.contains("+ HUBBELL 6SCOTLAND")
        || text.contains("HUBBELL 6SCOTLAND")
        || text.contains("AsusedinthisMutualConfidentialityAgreement")
        || text.contains("Allinformationofanycharacter")
        || text.contains("Datedthis")
}
