//! The load cases: built automatically from the model (and editable), and what each one is made of.
//!
//! - SUS: weight and pressure (W + P1), checked against the hot allowable.
//! - OPE1–3: weight, temperature, pressure and imposed displacements of each operating case.
//! - EXP1–3: the range between each operating case and the installed state (OPE − SUS), checked
//!   against the allowable stress range; with more than one operating case, also the range between them.
//! - HYD: weight full of water and the hydrotest pressure, when the settings ask for it.
//! - With wind or static seismic in the settings: OPE1+WIN (OPE1 with the wind) and OCC WIN, the
//!   range OPE1+WIN − OPE1 (the wind's own effect), checked as occasional; the same for seismic,
//!   and for the relief valves' thrust (OPE1+RV, OCC RV).

use crate::model::{CaseKind, LoadCase, Model, Recipe};

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
    let mut ope = [0u32; 3];
    for i in 0..3 {
        if u[i] {
            let n = i as u8 + 1;
            ope[i] = push(&mut out, format!("OPE{n}"), CaseKind::Ope, Recipe::Sum { w: true, water: false, t: n, d: n, p: n, wind: false, seis: false, relief: false });
        }
    }
    let sus = push(&mut out, "SUS".into(), CaseKind::Sus, Recipe::Sum { w: true, water: false, t: 0, d: 0, p: 1, wind: false, seis: false, relief: false });
    // A second or third sustained case where those cases' pressures differ from case 1's.
    for i in 1..3 {
        if u[i] && m.conds.iter().any(|c| c.p[i] != c.p[0]) {
            let n = i as u8 + 1;
            push(&mut out, format!("SUS{n}"), CaseKind::Sus, Recipe::Sum { w: true, water: false, t: 0, d: 0, p: n, wind: false, seis: false, relief: false });
        }
    }
    for i in 0..3 {
        if u[i] {
            push(&mut out, format!("EXP{}", i + 1), CaseKind::Exp, Recipe::Range { a: ope[i], b: sus });
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
    for (on, tag, wind, seis, relief) in [(s.wind_q > 0.0, "WIN", true, false, false), (s.seis.iter().any(|v| *v != 0.0), "SEIS", false, true, false), (!m.reliefs.is_empty(), "RV", false, false, true)] {
        if on {
            let with = push(&mut out, format!("OPE1+{tag}"), CaseKind::Ope, Recipe::Sum { w: true, water: false, t: 1, d: 1, p: 1, wind, seis, relief });
            push(&mut out, format!("OCC {tag}"), CaseKind::Occ, Recipe::Range { a: with, b: ope[0] });
        }
    }
    if m.settings.hydro {
        push(&mut out, "HYD".into(), CaseKind::Hyd, Recipe::Sum { w: true, water: true, t: 0, d: 0, p: 4, wind: false, seis: false, relief: false });
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
}
