//! The load cases: built automatically from the model (and editable), and what each one is made of.
//!
//! - SUS: weight and pressure (W + P1), checked against the hot allowable.
//! - OPE1–3: weight, temperature, pressure and imposed displacements of each operating case.
//! - EXP1–3: the range between each operating case and the installed state (OPE − SUS), checked
//!   against the allowable stress range; with more than one operating case, also the range between them.
//! - With a cold spring: COLD (W + P1 with the cold spring pulled in, the installed state's loads);
//!   the operating, occasional and hydrotest cases have it too, SUS does not, and EXP is OPE − COLD.
//! - HYD: weight full of water and the hydrotest pressure, when the settings ask for it.
//! - With wind or static seismic in the settings: OPE1+WIN (OPE1 with the wind) and OCC WIN, the
//!   range OPE1+WIN − OPE1 (the wind's own effect), checked as occasional; the same for seismic,
//!   and for the relief valves' thrust (OPE1+RV, OCC RV). With the occasional loads in both
//!   directions, also OPE1−WIN and OCC −WIN (and the same for seismic).

use crate::model::{CaseKind, LoadCase, Model, Recipe};
use std::collections::HashMap;

/// Which operating cases 1–3 the model uses: a temperature away from the installed one, a pressure,
/// or an imposed displacement.
pub fn used(m: &Model) -> [bool; 3] {
    let mut u = [false; 3];
    for i in 0..3 {
        u[i] = m.conds.iter().any(|c| (c.t[i] - m.settings.ambient).abs() > 1e-6 || c.p[i] != 0.0) || m.supports.iter().any(|s| s.disp[i].iter().any(|v| *v != 0.0));
    }
    // Case 1 is always there, so a cold model still has its operating and sustained cases.
    u[0] = true;
    u
}

/// The recommended load cases for the model (ids are filled in by the caller).
pub fn recommend(m: &Model) -> Vec<LoadCase> {
    let u = used(m);
    let mut out = vec![];
    let mut id = 1_000_000;
    let mut push = |out: &mut Vec<LoadCase>, name: String, kind: CaseKind, recipe: Recipe| {
        id += 1;
        out.push(LoadCase { id, name, kind, recipe });
        id
    };
    // With a cold spring it is pulled in for every case but the sustained ones (their stress is
    // weight and pressure only), and the expansion ranges start from COLD, the installed state with it.
    let cs = m.elements.iter().any(|e| e.cold > 0.0);
    let mut ope = [0u32; 3];
    for i in 0..3 {
        if u[i] {
            let n = i as u8 + 1;
            ope[i] = push(&mut out, format!("OPE{n}"), CaseKind::Ope, Recipe::Sum { w: true, water: false, t: n, d: n, p: n, wind: false, seis: false, relief: false, neg: false, cs });
        }
    }
    let sus = push(&mut out, "SUS".into(), CaseKind::Sus, Recipe::Sum { w: true, water: false, t: 0, d: 0, p: 1, wind: false, seis: false, relief: false, neg: false, cs: false });
    // A second or third sustained case where those cases' pressures differ from case 1's.
    for i in 1..3 {
        if u[i] && m.conds.iter().any(|c| c.p[i] != c.p[0]) {
            let n = i as u8 + 1;
            push(&mut out, format!("SUS{n}"), CaseKind::Sus, Recipe::Sum { w: true, water: false, t: 0, d: 0, p: n, wind: false, seis: false, relief: false, neg: false, cs: false });
        }
    }
    let base = if cs { push(&mut out, "COLD".into(), CaseKind::Ope, Recipe::Sum { w: true, water: false, t: 0, d: 0, p: 1, wind: false, seis: false, relief: false, neg: false, cs: true }) } else { sus };
    for i in 0..3 {
        if u[i] {
            push(&mut out, format!("EXP{}", i + 1), CaseKind::Exp, Recipe::Range { a: ope[i], b: base });
        }
    }
    for i in 0..3 {
        for j in i + 1..3 {
            if u[i] && u[j] {
                push(&mut out, format!("EXP{}–{}", i + 1, j + 1), CaseKind::Exp, Recipe::Range { a: ope[i], b: ope[j] });
            }
        }
    }
    // Occasional loads act with operating case 1: the case with them, and the range they add.
    let s = &m.settings;
    for (on, tag, wind, seis, relief) in [(s.has_wind(), "WIN", true, false, false), (s.seis.iter().any(|v| *v != 0.0), "SEIS", false, true, false), (!m.reliefs.is_empty(), "RV", false, false, true)] {
        if on {
            let with = push(&mut out, format!("OPE1+{tag}"), CaseKind::Ope, Recipe::Sum { w: true, water: false, t: 1, d: 1, p: 1, wind, seis, relief, neg: false, cs });
            push(&mut out, format!("OCC {tag}"), CaseKind::Occ, Recipe::Range { a: with, b: ope[0] });
            // Wind and seismic the other way too, when asked (one-way supports and friction make it differ).
            if s.occ_both && !relief {
                let with = push(&mut out, format!("OPE1−{tag}"), CaseKind::Ope, Recipe::Sum { w: true, water: false, t: 1, d: 1, p: 1, wind, seis, relief, neg: true, cs });
                push(&mut out, format!("OCC −{tag}"), CaseKind::Occ, Recipe::Range { a: with, b: ope[0] });
            }
        }
    }
    if m.settings.hydro {
        push(&mut out, "HYD".into(), CaseKind::Hyd, Recipe::Sum { w: true, water: true, t: 0, d: 0, p: 4, wind: false, seis: false, relief: false, neg: false, cs });
    }
    out
}

/// Replaces the model's load cases with the recommended ones, keeping the ids of cases with the
/// same name so results and selections stay put.
pub fn rebuild(m: &mut Model) {
    let mut new = recommend(m);
    let old: std::collections::HashMap<String, u32> = m.cases.iter().map(|c| (c.name.clone(), c.id)).collect();
    let mut map = std::collections::HashMap::new();
    let mut next = m.next.max(m.max_id() + 1);
    let taken: std::collections::HashSet<u32> = m.elements.iter().map(|x| x.id).chain(m.pipes.iter().map(|x| x.id)).chain(m.materials.iter().map(|x| x.id)).chain(m.conds.iter().map(|x| x.id)).chain(m.supports.iter().map(|x| x.id)).chain(m.nozzles.iter().map(|x| x.id)).chain(m.loads.iter().map(|x| x.id)).collect();
    for c in &mut new {
        let id = match old.get(&c.name) {
            Some(&i) if !taken.contains(&i) => i,
            _ => {
                next += 1;
                next - 1
            }
        };
        map.insert(c.id, id);
        c.id = id;
    }
    for c in &mut new {
        if let Recipe::Range { a, b } = &mut c.recipe {
            *a = map[a];
            *b = map[b];
        }
    }
    // Your combinations (0.7.6) are kept, after the cases: each part found again by its case's
    // name; a combination whose cases are all gone goes too.
    let by_name: HashMap<String, u32> = new.iter().map(|c| (c.name.clone(), c.id)).collect();
    let old_names: HashMap<u32, String> = m.cases.iter().map(|c| (c.id, c.name.clone())).collect();
    let mut kept: Vec<LoadCase> = vec![];
    for c in m.cases.iter().filter(|c| matches!(c.recipe, Recipe::Combo { .. })) {
        let Recipe::Combo { rule, parts } = &c.recipe else { continue };
        let parts: Vec<_> = parts.iter().filter_map(|p| {
            let name = old_names.get(&p.case)?;
            by_name.get(name).copied().or_else(|| kept.iter().find(|k| &k.name == name).map(|k| k.id)).map(|case| crate::model::ComboPart { case, factor: p.factor })
        }).collect();
        if parts.is_empty() || by_name.contains_key(&c.name) {
            continue;
        }
        let id = if taken.contains(&c.id) || new.iter().any(|x| x.id == c.id) {
            next += 1;
            next - 1
        } else {
            c.id
        };
        kept.push(LoadCase { id, name: c.name.clone(), kind: c.kind, recipe: Recipe::Combo { rule: *rule, parts } });
    }
    new.extend(kept);
    m.cases = new;
    m.next = next.max(m.max_id() + 1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::K0;

    #[test]
    fn cases_follow_the_operating_conditions() {
        let mut m = Model::new();
        let names = |m: &Model| m.cases.iter().map(|c| c.name.clone()).collect::<Vec<_>>();
        assert_eq!(names(&m), ["OPE1", "SUS", "EXP1"]);
        m.conds[0].t = [K0 + 250.0, K0 + 120.0, K0 + 21.0];
        m.conds[0].p = [10e5, 10e5, 0.0];
        m.settings.hydro = true;
        rebuild(&mut m);
        assert_eq!(names(&m), ["OPE1", "OPE2", "SUS", "EXP1", "EXP2", "EXP1–2", "HYD"]);
        let ids: Vec<u32> = m.cases.iter().map(|c| c.id).collect();
        rebuild(&mut m);
        assert_eq!(ids, m.cases.iter().map(|c| c.id).collect::<Vec<_>>(), "rebuilding keeps ids");
        assert!(m.check().is_ok(), "{:?}", m.check());
    }

    #[test]
    fn combinations_are_kept_checked_and_cascaded() {
        use crate::model::{ComboPart, ComboRule};
        let mut m = Model::new();
        m.conds[0].t = [K0 + 250.0, K0 + 120.0, K0 + 21.0];
        rebuild(&mut m);
        let id = |m: &Model, n: &str| m.cases.iter().find(|c| c.name == n).unwrap().id;
        let (sus, ope2) = (id(&m, "SUS"), id(&m, "OPE2"));
        let combo = LoadCase { id: 900, name: "SUS+OPE2".into(), kind: CaseKind::Ope, recipe: Recipe::Combo { rule: ComboRule::Scalar, parts: vec![ComboPart { case: sus, factor: 1.0 }, ComboPart { case: ope2, factor: 1.5 }] } };
        m.cases.push(combo.clone());
        m.next = m.next.max(901);
        assert!(m.check().is_ok(), "{:?}", m.check());
        // Rebuilding the recommended cases keeps the combination, its parts found by name.
        rebuild(&mut m);
        let c = m.cases.iter().find(|c| c.name == "SUS+OPE2").expect("kept");
        assert_eq!(c.recipe, Recipe::Combo { rule: ComboRule::Scalar, parts: vec![ComboPart { case: id(&m, "SUS"), factor: 1.0 }, ComboPart { case: id(&m, "OPE2"), factor: 1.5 }] });
        // OPE2 no longer used: the combination keeps SUS only.
        m.conds[0].t[1] = m.settings.ambient;
        rebuild(&mut m);
        let c = m.cases.iter().find(|c| c.name == "SUS+OPE2").expect("kept with SUS");
        assert!(matches!(&c.recipe, Recipe::Combo { parts, .. } if parts.len() == 1));
        assert!(m.check().is_ok(), "{:?}", m.check());
        // A combination before its cases, of itself, or with no cases is refused.
        let mut bad = m.clone();
        let c = bad.cases.pop().unwrap();
        bad.cases.insert(0, c);
        assert!(bad.check().unwrap_err().contains("must come after"));
        let mut bad = m.clone();
        if let Recipe::Combo { parts, .. } = &mut bad.cases.last_mut().unwrap().recipe { parts.clear(); }
        assert!(bad.check().is_err());
        // Saved and read back the same.
        let (back, w) = crate::model::from_json(&crate::model::to_json(&m)).unwrap();
        assert!(w.is_empty() && back == m);
        // Deleting the case it is made of takes the combination with it.
        let sus = id(&m, "SUS");
        let mut d = crate::doc::Doc::new(m);
        d.apply(crate::doc::Op::Delete { ids: vec![sus] }).unwrap();
        assert!(!d.m.cases.iter().any(|c| c.name == "SUS+OPE2"), "{:?}", d.m.cases);
        // Adding a combination keeps the cases built automatically; changing a case does not.
        let mut d = crate::doc::Doc::new(Model::new());
        assert!(d.m.settings.auto_cases);
        let mut cs = d.m.cases.clone();
        cs.push(LoadCase { id: 0, name: "2 SUS".into(), kind: CaseKind::Ope, recipe: Recipe::Combo { rule: ComboRule::Algebraic, parts: vec![ComboPart { case: id(&d.m, "SUS"), factor: 2.0 }] } });
        d.apply(crate::doc::Op::SetCases { cases: cs.clone() }).unwrap();
        assert!(d.m.settings.auto_cases && d.m.cases.iter().any(|c| c.name == "2 SUS" && c.id != 0));
        cs[0].name = "OPE".into();
        d.apply(crate::doc::Op::SetCases { cases: cs }).unwrap();
        assert!(!d.m.settings.auto_cases);
    }
}
